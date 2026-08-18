//! Reverse-DNS via the local resolver only (`GetNameInfoW`), with a bounded
//! in-memory cache so radar polls never hammer the resolver.

use crate::network::sockets::RawConn;
use std::time::{Duration, Instant};
use windows::Win32::Networking::WinSock::{
    socklen_t, GetNameInfoW, AF_INET, AF_INET6, IN6_ADDR, IN6_ADDR_0, IN_ADDR, IN_ADDR_0,
    NI_NAMEREQD, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6,
};

const TTL: Duration = Duration::from_secs(300);

pub struct HostCache {
    map: std::collections::HashMap<String, (String, Instant)>,
}

impl Default for HostCache {
    fn default() -> Self {
        Self::new()
    }
}

impl HostCache {
    pub fn new() -> Self {
        HostCache {
            map: std::collections::HashMap::new(),
        }
    }

    /// Standalone reverse lookup for a bare IP string (the `resolve_host`
    /// command path). Less strict than the polling path.
    pub fn resolve_ip_string(&mut self, ip: &str) -> Option<String> {
        let v6 = ip.contains(':');
        let bytes = if v6 {
            ip.parse::<std::net::Ipv6Addr>().ok().map(|a| a.octets())
        } else {
            ip.parse::<std::net::Ipv4Addr>().ok().map(|a| {
                let o = a.octets();
                let mut b = [0u8; 16];
                b[..4].copy_from_slice(&o);
                b
            })
        }?;
        let conn = RawConn {
            pid: 0,
            proto: "tcp",
            state: String::new(),
            local_addr: String::new(),
            local_port: 0,
            remote_addr: Some(ip.to_string()),
            remote_port: Some(0),
            zone: "internet",
            key: String::new(),
            v6,
            local_bytes: None,
            remote_bytes: Some(bytes),
            scope: 0,
            net_local_addr: 0,
            net_remote_addr: 0,
            net_local_port: 0,
            net_remote_port: 0,
        };
        self.resolve(&conn, Duration::from_millis(200))
    }

    /// Returns a cached or freshly resolved hostname, or None. Budget caps
    /// the total time spent across one poll.
    pub fn resolve(&mut self, conn: &RawConn, budget: Duration) -> Option<String> {
        let addr = conn.remote_addr.as_ref()?;
        if let Some((host, at)) = self.map.get(addr) {
            if at.elapsed() < TTL {
                return Some(host.clone());
            }
        }
        let t0 = Instant::now();
        let resolved = reverse_lookup(conn);
        if t0.elapsed() > budget {
            return resolved;
        }
        if let Some(h) = resolved {
            self.map.insert(addr.clone(), (h.clone(), Instant::now()));
            return Some(h);
        }
        None
    }
}

/// Command-level helper (no persistent cache — each call is bounded).
pub fn resolve_ip(ip: &str) -> crate::core::Result<Option<String>> {
    let mut cache = HostCache::new();
    Ok(cache.resolve_ip_string(ip))
}

fn reverse_lookup(conn: &RawConn) -> Option<String> {
    let mut nodebuf = [0u16; 256];
    unsafe {
        let rc = if conn.v6 {
            let bytes = conn.remote_bytes?;
            let sa = SOCKADDR_IN6 {
                sin6_family: AF_INET6,
                sin6_port: 0,
                sin6_flowinfo: 0,
                sin6_addr: IN6_ADDR {
                    u: IN6_ADDR_0 { Byte: bytes },
                },
                Anonymous: windows::Win32::Networking::WinSock::SOCKADDR_IN6_0 {
                    sin6_scope_id: conn.scope,
                },
            };
            GetNameInfoW(
                &sa as *const SOCKADDR_IN6 as *const SOCKADDR,
                socklen_t(std::mem::size_of::<SOCKADDR_IN6>() as i32),
                Some(&mut nodebuf),
                None,
                NI_NAMEREQD as i32,
            )
        } else {
            let bytes = conn.remote_bytes?;
            let sa = SOCKADDR_IN {
                sin_family: AF_INET,
                sin_port: 0,
                sin_addr: IN_ADDR {
                    S_un: IN_ADDR_0 {
                        S_addr: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    },
                },
                sin_zero: [0; 8],
            };
            GetNameInfoW(
                &sa as *const SOCKADDR_IN as *const SOCKADDR,
                socklen_t(std::mem::size_of::<SOCKADDR_IN>() as i32),
                Some(&mut nodebuf),
                None,
                NI_NAMEREQD as i32,
            )
        };
        if rc != 0 {
            return None;
        }
        let end = nodebuf.iter().position(|&c| c == 0).unwrap_or(256);
        if end == 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&nodebuf[..end]))
    }
}
