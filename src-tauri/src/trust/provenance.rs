//! Build provenance (baked via build.rs) and the embedded dependency audit.

use crate::core::ZtError;
use crate::trust::models::{BuildInfo, DepAudit, DepAuditFileScan};
use windows::Win32::UI::Shell::SHGetFolderPathW;

pub const REPO: &str = "github.com/archanagaur175-glitch/ZeroTrace";

pub fn rfc3339_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, m, d) = civil_from_days((secs / 86400) as i64);
    let rem = secs % 86400;
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// Howard Hinnant `civil_from_days` — days since 1970-01-01 to Y/M/D.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

pub fn build_info() -> BuildInfo {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let commit = std::env::var("ZT_COMMIT").unwrap_or_else(|_| "unknown".into());
    let tag = std::env::var("ZT_TAG").ok().filter(|t| !t.is_empty());
    let built_at = std::env::var("ZT_BUILT_AT").unwrap_or_else(|_| "unknown".into());
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    BuildInfo {
        version,
        commit,
        tag,
        repo: REPO.to_string(),
        built_at,
        profile: profile.to_string(),
    }
}

static EMPTY_AUDIT: &str =
    r#"{"generated_at":"unknown","files":[],"total_deps":0,"hits":[],"zero_telemetry":true}"#;

pub fn dependency_audit() -> DepAudit {
    let raw = std::env::var("ZT_DEP_AUDIT_JSON")
        .map(|p| std::fs::read_to_string(&p).unwrap_or_else(|_| EMPTY_AUDIT.to_string()))
        .unwrap_or_else(|_| EMPTY_AUDIT.to_string());
    serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::from_str(EMPTY_AUDIT).unwrap())
}

/// Resolve a shell known folder (CSIDL) to an absolute path.
pub fn known_folder(csidl: i32) -> crate::core::Result<std::path::PathBuf> {
    let mut buf = [0u16; 260];
    unsafe { SHGetFolderPathW(None, csidl, None, 0, &mut buf) }.map_err(ZtError::from)?;
    let end = buf.iter().position(|&c| c == 0).unwrap_or(260);
    let s = String::from_utf16_lossy(&buf[..end]);
    if s.is_empty() {
        Err(ZtError::new("known folder resolved to empty path"))
    } else {
        Ok(std::path::PathBuf::from(s))
    }
}

// Re-exported so other modules can reuse the field-free parse shape without
// giving the UI extra surface.
pub type AuditFileScan = DepAuditFileScan;
