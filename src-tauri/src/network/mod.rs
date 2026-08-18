//! Network radar backend: polling handler that turns raw socket tables into
//! the `NetSnapshot` shipped to the UI, with ESTATS-first traffic counting.

pub mod estats;
pub mod firewall;
pub mod interfaces;
pub mod process;
pub mod resolver;
pub mod sockets;

use crate::core::Timeblock;
use crate::network::estats::TrafficTracker;
use crate::network::resolver::HostCache;
use crate::trust::models::{ConnectionRow, NetSnapshot, ProcessAgg};
use std::cmp::Reverse;
use std::collections::HashMap;

pub struct Tracker {
    pub traffic: TrafficTracker,
    pub hosts: HostCache,
    pid_cache: HashMap<u32, (String, Option<String>)>,
}

impl Default for Tracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Tracker {
    pub fn new() -> Self {
        Tracker {
            traffic: TrafficTracker::default(),
            hosts: HostCache::new(),
            pid_cache: HashMap::new(),
        }
    }

    pub fn snapshot(&mut self, interval_ms: u32, reverse_dns: bool) -> NetSnapshot {
        let mut raw = sockets::tcp_rows();
        raw.extend(sockets::udp_rows());

        let budget = Timeblock::new(40);
        let mut connections = Vec::with_capacity(raw.len());
        let mut agg: HashMap<u32, (String, Option<String>, u64, u64, u32)> = HashMap::new();
        let mut total_out: u64 = 0;
        let mut total_in: u64 = 0;

        for conn in &raw {
            let (name, path) = match self.pid_cache.get(&conn.pid) {
                Some(v) => v.clone(),
                None => {
                    let resolved = process::resolve_pid(conn.pid);
                    let entry = resolved
                        .map(|r| (r.name, r.path))
                        .unwrap_or_else(|| (format!("[{}]", conn.pid), None));
                    self.pid_cache.insert(conn.pid, entry.clone());
                    entry
                }
            };

            let (bytes_out, bytes_in, track) = self.traffic.sample(conn);

            let remote_host = if reverse_dns && conn.zone == "internet" && !budget.exhausted() {
                self.hosts.resolve(conn, budget.remaining())
            } else {
                None
            };

            total_out += bytes_out;
            total_in += bytes_in;

            let entry = agg
                .entry(conn.pid)
                .or_insert_with(|| (name.clone(), path.clone(), 0u64, 0u64, 0u32));
            entry.2 += bytes_out;
            entry.3 += bytes_in;
            entry.4 += 1;

            connections.push(ConnectionRow {
                pid: conn.pid,
                process: name,
                process_path: path,
                proto: conn.proto.to_string(),
                state: conn.state.clone(),
                local_addr: conn.local_addr.clone(),
                local_port: conn.local_port as u32,
                remote_addr: conn.remote_addr.clone(),
                remote_host,
                remote_port: conn.remote_port.map(|p| p as u32),
                zone: conn.zone.to_string(),
                bytes_out,
                bytes_in,
                track: track.to_string(),
            });
        }

        let mut aggregates: Vec<ProcessAgg> = agg
            .into_iter()
            .map(
                |(pid, (process, process_path, bytes_out, bytes_in, conns))| ProcessAgg {
                    pid,
                    process,
                    process_path,
                    bytes_out,
                    bytes_in,
                    conns,
                },
            )
            .collect();
        aggregates.sort_by_key(|a| Reverse(a.bytes_in + a.bytes_out));

        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0);

        NetSnapshot {
            ts,
            interval_ms,
            connections,
            aggregates,
            total_out,
            total_in,
        }
    }
}
