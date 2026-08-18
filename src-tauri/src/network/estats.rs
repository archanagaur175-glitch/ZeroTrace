//! Per-connection traffic measurement.
//!
//! Primary path: kernel TCP ESTATS (`GetPerTcpConnectionEStats`), which
//! exposes true cumulative byte counters per established connection. Fallback
//! (short-lived / ESTATS-empty rows, and all UDP): the row is flagged
//! `track: "churn"` and reported with 0 bytes rather than a fabricated value.

use crate::network::sockets::RawConn;
use std::collections::HashMap;
use windows::Win32::NetworkManagement::IpHelper::{
    GetPerTcpConnectionEStats, SetPerTcpConnectionEStats, TCP_ESTATS_DATA_ROD_v0,
    TCP_ESTATS_DATA_RW_v0, TcpConnectionEstatsData, MIB_TCPROW_LH, MIB_TCPROW_LH_0,
    MIB_TCP_STATE_ESTAB,
};

const ESTATS_ENABLED: &str = "estats";
const ESTATS_CHURN: &str = "churn";

#[derive(Default)]
pub struct TrafficTracker {
    prev: HashMap<String, (u64, u64)>,
}

impl TrafficTracker {
    /// Returns (bytes_out, bytes_in, track) for this poll.
    pub fn sample(&mut self, conn: &RawConn) -> (u64, u64, &'static str) {
        match estats_sample(conn) {
            Some((out, inn)) => {
                let (dout, din) = match self.prev.get(&conn.key) {
                    Some(&(pout, pin)) => (out.saturating_sub(pout), inn.saturating_sub(pin)),
                    None => (0, 0),
                };
                self.prev.insert(conn.key.clone(), (out, inn));
                (dout, din, ESTATS_ENABLED)
            }
            None => (0, 0, ESTATS_CHURN),
        }
    }
}

/// Cumulative ESTATS bytes for an established IPv4 TCP connection, if the
/// kernel exposes them. `SetPerTcpConnectionEStats` (enable) then
/// `GetPerTcpConnectionEStats` (read). Any failure → None → churn.
fn estats_sample(conn: &RawConn) -> Option<(u64, u64)> {
    if conn.proto != "tcp"
        || conn.v6
        || conn.state != "ESTABLISHED"
        || conn.remote_addr.is_none()
        || conn.net_remote_addr == 0
    {
        return None;
    }

    let row = MIB_TCPROW_LH {
        Anonymous: MIB_TCPROW_LH_0 {
            dwState: MIB_TCP_STATE_ESTAB.0 as u32,
        },
        dwLocalAddr: conn.net_local_addr,
        dwLocalPort: conn.net_local_port as u32,
        dwRemoteAddr: conn.net_remote_addr,
        dwRemotePort: conn.net_remote_port as u32,
    };

    unsafe {
        // Enable collection for the connection (idempotent, best effort).
        let rw = TCP_ESTATS_DATA_RW_v0 {
            EnableCollection: true,
        };
        let rw_bytes = core::slice::from_raw_parts(
            (&rw as *const TCP_ESTATS_DATA_RW_v0) as *const u8,
            core::mem::size_of::<TCP_ESTATS_DATA_RW_v0>(),
        );
        let _ = SetPerTcpConnectionEStats(&row, TcpConnectionEstatsData, rw_bytes, 0, 0);

        // Read the counters.
        let mut rod = TCP_ESTATS_DATA_ROD_v0::default();
        let rod_bytes = core::slice::from_raw_parts_mut(
            (&mut rod as *mut TCP_ESTATS_DATA_ROD_v0) as *mut u8,
            core::mem::size_of::<TCP_ESTATS_DATA_ROD_v0>(),
        );
        let rc = GetPerTcpConnectionEStats(
            &row,
            TcpConnectionEstatsData,
            None,
            0,
            None,
            0,
            Some(rod_bytes),
            0,
        );
        if rc != 0 {
            return None;
        }
        Some((rod.DataBytesOut, rod.DataBytesIn))
    }
}
