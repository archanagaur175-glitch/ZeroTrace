//! Raw socket enumeration (TCP/UDP, v4/v6, owner PID) via GetExtendedTcpTable
//! / GetExtendedUdpTable. Rows carry the raw network-order address bytes so
//! display, reverse-DNS, and ESTATS sampling all consume one canonical shape.

use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCP6TABLE_OWNER_PID,
    MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID, MIB_TCP_STATE, MIB_UDP6ROW_OWNER_PID,
    MIB_UDP6TABLE_OWNER_PID, MIB_UDPROW_OWNER_PID, MIB_UDPTABLE_OWNER_PID, TCP_TABLE_OWNER_PID_ALL,
    UDP_TABLE_OWNER_PID,
};

pub const AF_INET_V4: u32 = 2;
pub const AF_INET_V6: u32 = 23;

#[derive(Debug, Clone)]
pub struct RawConn {
    pub pid: u32,
    pub proto: &'static str, // "tcp" | "udp"
    pub state: String,       // "LISTEN"/"ESTABLISHED"/… or "UDP"
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: Option<String>,
    pub remote_port: Option<u16>,
    pub zone: &'static str,
    pub key: String,
    pub v6: bool,
    /// Raw network-order address bytes (4 for v4, 16 for v6).
    pub local_bytes: Option<[u8; 16]>,
    pub remote_bytes: Option<[u8; 16]>,
    /// v6 scope id.
    pub scope: u32,
    /// Net-order u32 form used by ESTATS (v4 TCP only).
    pub net_local_addr: u32,
    pub net_remote_addr: u32,
    /// Net-order port values as read from the table (ESTATS uses these).
    pub net_local_port: u16,
    pub net_remote_port: u16,
}

impl RawConn {
    fn key_for(proto: &str, local: &str, lp: u16, remote: &str, rp: u16) -> String {
        format!("{proto}|{local}:{lp}|{remote}:{rp}")
    }
}

pub fn tcp_rows() -> Vec<RawConn> {
    let mut out = Vec::new();
    for row in tcp4() {
        let local = fmt_ipv4(row.dwLocalAddr);
        let remote = fmt_ipv4(row.dwRemoteAddr);
        let state = tcp_state_name(MIB_TCP_STATE(row.dwState as i32));
        let (r, rp) = normalize_remote(&remote, row.dwRemotePort, &state);
        let key = RawConn::key_for("tcp", &local, ntohs(row.dwLocalPort), &r, rp);
        out.push(RawConn {
            pid: row.dwOwningPid,
            proto: "tcp",
            state,
            zone: zone_for(Some(&r), false),
            local_addr: local,
            local_port: ntohs(row.dwLocalPort),
            remote_addr: if r.is_empty() { None } else { Some(r.clone()) },
            remote_port: (rp != 0).then_some(rp),
            key,
            v6: false,
            local_bytes: Some(u32_to_bytes(row.dwLocalAddr)),
            remote_bytes: (row.dwRemoteAddr != 0).then(|| u32_to_bytes(row.dwRemoteAddr)),
            scope: 0,
            net_local_addr: row.dwLocalAddr,
            net_remote_addr: row.dwRemoteAddr,
            net_local_port: row.dwLocalPort as u16,
            net_remote_port: row.dwRemotePort as u16,
        });
    }
    for row in tcp6() {
        let local = fmt_ipv6(&row.ucLocalAddr);
        let remote = fmt_ipv6(&row.ucRemoteAddr);
        let state = tcp_state_name(MIB_TCP_STATE(row.dwState as i32));
        let (r, rp) = normalize_remote(&remote, row.dwRemotePort, &state);
        let key = RawConn::key_for("tcp", &local, ntohs(row.dwLocalPort), &r, rp);
        let empty = [0u8; 16];
        out.push(RawConn {
            pid: row.dwOwningPid,
            proto: "tcp",
            state,
            zone: zone_for(Some(&r), true),
            local_addr: local,
            local_port: ntohs(row.dwLocalPort),
            remote_addr: if r.is_empty() { None } else { Some(r.clone()) },
            remote_port: (rp != 0).then_some(rp),
            key,
            v6: true,
            local_bytes: Some(row.ucLocalAddr),
            remote_bytes: (row.ucRemoteAddr != empty).then_some(row.ucRemoteAddr),
            scope: row.dwLocalScopeId,
            net_local_addr: 0,
            net_remote_addr: 0,
            net_local_port: row.dwLocalPort as u16,
            net_remote_port: 0,
        });
    }
    out
}

pub fn udp_rows() -> Vec<RawConn> {
    let mut out = Vec::new();
    for row in udp4() {
        let local = fmt_ipv4(row.dwLocalAddr);
        let key = RawConn::key_for("udp", &local, ntohs(row.dwLocalPort), "", 0);
        out.push(RawConn {
            pid: row.dwOwningPid,
            proto: "udp",
            state: "UDP".to_string(),
            zone: zone_for(None, false),
            local_addr: local,
            local_port: ntohs(row.dwLocalPort),
            remote_addr: None,
            remote_port: None,
            key,
            v6: false,
            local_bytes: Some(u32_to_bytes(row.dwLocalAddr)),
            remote_bytes: None,
            scope: 0,
            net_local_addr: row.dwLocalAddr,
            net_remote_addr: 0,
            net_local_port: row.dwLocalPort as u16,
            net_remote_port: 0,
        });
    }
    for row in udp6() {
        let local = fmt_ipv6(&row.ucLocalAddr);
        let key = RawConn::key_for("udp", &local, ntohs(row.dwLocalPort), "", 0);
        out.push(RawConn {
            pid: row.dwOwningPid,
            proto: "udp",
            state: "UDP".to_string(),
            zone: zone_for(None, true),
            local_addr: local,
            local_port: ntohs(row.dwLocalPort),
            remote_addr: None,
            remote_port: None,
            key,
            v6: true,
            local_bytes: Some(row.ucLocalAddr),
            remote_bytes: None,
            scope: row.dwLocalScopeId,
            net_local_addr: 0,
            net_remote_addr: 0,
            net_local_port: row.dwLocalPort as u16,
            net_remote_port: 0,
        });
    }
    out
}

// -- table fetchers ----------------------------------------------------------

fn tcp4() -> Vec<MIB_TCPROW_OWNER_PID> {
    unsafe {
        let mut size: u32 = 0;
        let rc = GetExtendedTcpTable(
            None,
            &mut size,
            false,
            AF_INET_V4,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
        if rc != 0 || size == 0 {
            return Vec::new();
        }
        let mut buf: Vec<u8> = vec![0u8; size as usize];
        let rc = GetExtendedTcpTable(
            Some(buf.as_mut_ptr() as *mut core::ffi::c_void),
            &mut size,
            false,
            AF_INET_V4,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
        if rc != 0 {
            return Vec::new();
        }
        let table = buf.as_ptr() as *const MIB_TCPTABLE_OWNER_PID;
        let n = (*table).dwNumEntries as usize;
        core::slice::from_raw_parts((*table).table.as_ptr(), n).to_vec()
    }
}

fn tcp6() -> Vec<MIB_TCP6ROW_OWNER_PID> {
    unsafe {
        let mut size: u32 = 0;
        let rc = GetExtendedTcpTable(
            None,
            &mut size,
            false,
            AF_INET_V6,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
        if rc != 0 || size == 0 {
            return Vec::new();
        }
        let mut buf: Vec<u8> = vec![0u8; size as usize];
        let rc = GetExtendedTcpTable(
            Some(buf.as_mut_ptr() as *mut core::ffi::c_void),
            &mut size,
            false,
            AF_INET_V6,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
        if rc != 0 {
            return Vec::new();
        }
        let table = buf.as_ptr() as *const MIB_TCP6TABLE_OWNER_PID;
        let n = (*table).dwNumEntries as usize;
        core::slice::from_raw_parts((*table).table.as_ptr(), n).to_vec()
    }
}

fn udp4() -> Vec<MIB_UDPROW_OWNER_PID> {
    unsafe {
        let mut size: u32 = 0;
        let rc = GetExtendedUdpTable(None, &mut size, false, AF_INET_V4, UDP_TABLE_OWNER_PID, 0);
        if rc != 0 || size == 0 {
            return Vec::new();
        }
        let mut buf: Vec<u8> = vec![0u8; size as usize];
        let rc = GetExtendedUdpTable(
            Some(buf.as_mut_ptr() as *mut core::ffi::c_void),
            &mut size,
            false,
            AF_INET_V4,
            UDP_TABLE_OWNER_PID,
            0,
        );
        if rc != 0 {
            return Vec::new();
        }
        let table = buf.as_ptr() as *const MIB_UDPTABLE_OWNER_PID;
        let n = (*table).dwNumEntries as usize;
        core::slice::from_raw_parts((*table).table.as_ptr(), n).to_vec()
    }
}

fn udp6() -> Vec<MIB_UDP6ROW_OWNER_PID> {
    unsafe {
        let mut size: u32 = 0;
        let rc = GetExtendedUdpTable(None, &mut size, false, AF_INET_V6, UDP_TABLE_OWNER_PID, 0);
        if rc != 0 || size == 0 {
            return Vec::new();
        }
        let mut buf: Vec<u8> = vec![0u8; size as usize];
        let rc = GetExtendedUdpTable(
            Some(buf.as_mut_ptr() as *mut core::ffi::c_void),
            &mut size,
            false,
            AF_INET_V6,
            UDP_TABLE_OWNER_PID,
            0,
        );
        if rc != 0 {
            return Vec::new();
        }
        let table = buf.as_ptr() as *const MIB_UDP6TABLE_OWNER_PID;
        let n = (*table).dwNumEntries as usize;
        core::slice::from_raw_parts((*table).table.as_ptr(), n).to_vec()
    }
}

// -- helpers -----------------------------------------------------------------

fn ntohs(port: u32) -> u16 {
    u16::from_be(port as u16)
}

fn u32_to_bytes(v: u32) -> [u8; 16] {
    let b = v.to_le_bytes();
    let mut out = [0u8; 16];
    out[..4].copy_from_slice(&b);
    out
}

fn normalize_remote(remote: &str, port: u32, state: &str) -> (String, u16) {
    let empty = remote.is_empty() || remote == "0.0.0.0" || remote == "::";
    if empty {
        return (String::new(), 0);
    }
    // LISTEN rows carry the remote of the listener (usually all-zero).
    if state == "LISTEN" {
        return (String::new(), 0);
    }
    (remote.to_string(), ntohs(port))
}

pub fn fmt_ipv4(v: u32) -> String {
    let b = v.to_le_bytes();
    format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3])
}

pub fn fmt_ipv6(bytes: &[u8; 16]) -> String {
    let groups: Vec<u16> = bytes
        .chunks(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();

    // Longest run of zero groups (length >= 2) becomes "::".
    let mut run_start: Option<usize> = None;
    let mut run_len = 0usize;
    let mut best_start: Option<usize> = None;
    let mut best_len = 0usize;
    for (i, &g) in groups.iter().enumerate() {
        if g == 0 {
            if run_start.is_none() {
                run_start = Some(i);
            }
            run_len += 1;
        } else {
            if run_len >= 2 && run_len > best_len {
                best_len = run_len;
                best_start = run_start;
            }
            run_start = None;
            run_len = 0;
        }
    }
    if run_len >= 2 && run_len > best_len {
        best_len = run_len;
        best_start = run_start;
    }

    match best_start {
        Some(start) => {
            let mut s = String::new();
            for (i, g) in groups[..start].iter().enumerate() {
                if i > 0 {
                    s.push(':');
                }
                s.push_str(&format!("{g:x}"));
            }
            s.push_str("::");
            let end = start + best_len;
            for (idx, g) in groups[end..].iter().enumerate() {
                if idx > 0 {
                    s.push(':');
                }
                s.push_str(&format!("{g:x}"));
            }
            if start == 0 && end == 8 {
                // all-zero address
                s
            } else {
                s
            }
        }
        None => groups
            .iter()
            .map(|g| format!("{g:x}"))
            .collect::<Vec<_>>()
            .join(":"),
    }
}

pub fn tcp_state_name(state: MIB_TCP_STATE) -> String {
    let name = match state.0 {
        1 => "CLOSED",
        2 => "LISTEN",
        3 => "SYN_SENT",
        4 => "SYN_RCVD",
        5 => "ESTABLISHED",
        6 => "FIN_WAIT1",
        7 => "FIN_WAIT2",
        8 => "CLOSE_WAIT",
        9 => "CLOSING",
        10 => "LAST_ACK",
        11 => "TIME_WAIT",
        12 => "DELETE_TCB",
        _ => "UNKNOWN",
    };
    name.to_string()
}

pub fn zone_for(remote: Option<&str>, v6: bool) -> &'static str {
    let Some(a) = remote else {
        return "unknown";
    };
    if a.is_empty() {
        return "unknown";
    }
    if v6 {
        if a == "::1" {
            return "local";
        }
        if a.starts_with("fe80") || a.starts_with("fc") || a.starts_with("fd") {
            return "lan";
        }
        if a.starts_with("ff0") {
            return "lan";
        }
        return "internet";
    }
    if a.starts_with("127.") || a.starts_with("0.") {
        return "local";
    }
    if a.starts_with("10.") || a.starts_with("192.168.") || a.starts_with("169.254.") {
        return "lan";
    }
    if let Some(rest) = a.strip_prefix("172.") {
        if let Some(first) = rest.split('.').next().and_then(|p| p.parse::<u8>().ok()) {
            if (16..=31).contains(&first) {
                return "lan";
            }
        }
    }
    if a.starts_with("224.") || a.starts_with("239.") {
        return "lan";
    }
    "internet"
}
