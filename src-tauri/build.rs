fn main() {
    // Bake build provenance into the binary via env vars set by the build.
    let commit = std::env::var("GITHUB_SHA").ok().or_else(|| {
        std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    });
    if let Some(c) = commit {
        println!("cargo:rustc-env=ZT_COMMIT={c}");
    }
    let tag = std::env::var("GITHUB_REF").ok().and_then(|r| {
        r.strip_prefix("refs/tags/")
            .map(|t| t.to_string())
            .or_else(|| r.split('/').next_back().map(|t| t.to_string()))
    });
    if let Some(t) = tag {
        println!("cargo:rustc-env=ZT_TAG={t}");
    }
    println!("cargo:rustc-env=ZT_BUILT_AT={}", rfc3339_now());
    if let Ok(path) = std::env::var("ZT_DEP_AUDIT_JSON") {
        println!("cargo:rustc-env=ZT_DEP_AUDIT_JSON={path}");
    }
    tauri_build::build()
}

/// RFC3339 UTC timestamp, no chrono dependency (build script).
fn rfc3339_now() -> String {
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

/// Howard Hinnant `civil_from_days`.
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
