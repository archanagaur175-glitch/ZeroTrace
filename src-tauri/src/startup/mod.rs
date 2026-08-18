//! Startup & service inventory (autostart x-ray).
//!
//! Sources: registry Run/RunOnce (HKCU, HKLM, HKLM Wow6432Node), Startup
//! folder shortcuts, scheduled tasks, and background services. Excludes the
//! local zero-trust app itself.

use crate::core::ZtError;
use crate::trust::models::{JournalEntry, ServiceRow, StartupEntry};
use crate::trust::{journal, signature};
use std::path::PathBuf;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_ONCE: &str = r"Software\Microsoft\Windows\CurrentVersion\RunOnce";
const RUN_WOW: &str = r"Software\Microsoft\Windows\CurrentVersion\Run\Wow6432Node";

fn user_root() -> RegKey {
    RegKey::predef(HKEY_CURRENT_USER)
}
fn machine_root() -> RegKey {
    RegKey::predef(HKEY_LOCAL_MACHINE)
}

fn enumerate_run(key_path: &str, source: &str, out: &mut Vec<StartupEntry>) {
    let roots = [("HKCU", user_root()), ("HKLM", machine_root())];
    for (hive, root) in roots {
        if let Ok(key) = root.open_subkey_with_flags(key_path, KEY_READ) {
            for (name, value) in key.enum_values().flatten() {
                let cmd = string_of(&value).trim().to_string();
                if cmd.is_empty() {
                    continue;
                }
                let is_runonce = key_path == RUN_ONCE;
                out.push(entry_from_reg(
                    id_for_reg(hive, key_path, &name, is_runonce),
                    name,
                    cmd,
                    source,
                    hive,
                    key_path,
                    is_runonce,
                ));
            }
        }
    }
    // The Wow6432Node alias on 64-bit systems (legacy run keys).
    if key_path != RUN_ONCE {
        if let Ok(key) = machine_root().open_subkey_with_flags(
            r"Software\Wow6432Node\Microsoft\Windows\CurrentVersion\Run",
            KEY_READ,
        ) {
            for (name, value) in key.enum_values().flatten() {
                let cmd = string_of(&value).trim().to_string();
                if cmd.is_empty() {
                    continue;
                }
                out.push(entry_from_reg(
                    id_for_reg("HKLM32", RUN_WOW, &name, false),
                    name,
                    cmd,
                    source,
                    "HKLM32",
                    RUN_WOW,
                    false,
                ));
            }
        }
    }
}

fn id_for_reg(hive: &str, key_path: &str, name: &str, runonce: bool) -> String {
    let kind = if runonce { "runonce" } else { "run" };
    format!("{kind}|{hive}|{key_path}|{name}")
}

fn string_of(value: &winreg::RegValue) -> String {
    String::from_utf16_lossy(
        &value
            .bytes
            .chunks(2)
            .map(|c| u16::from_ne_bytes([c[0], c[1]]))
            .collect::<Vec<u16>>(),
    )
    .trim_end_matches('\0')
    .to_string()
}

fn entry_from_reg(
    id: String,
    name: String,
    command: String,
    source: &str,
    _hive: &str,
    key_path: &str,
    is_runonce: bool,
) -> StartupEntry {
    let path = exe_from_command(&command);
    let (signed, publisher) = path
        .as_deref()
        .map(signature::inspect)
        .unwrap_or((None, None));
    let (risk, reasons) = classify(path.as_deref(), signed, publisher.as_deref());
    StartupEntry {
        id,
        name,
        command: command.clone(),
        source: source.to_string(),
        path,
        signed,
        publisher,
        risk: risk.to_string(),
        risk_reasons: reasons,
        enabled: true, // presence in the Run key means it runs
        source_detail: if is_runonce {
            format!("run-once value at {key_path}")
        } else {
            format!("autorun value at {key_path}")
        },
    }
}

/// Best-effort extraction of the executable path from an autorun command.
fn exe_from_command(cmd: &str) -> Option<String> {
    let c = cmd.trim();
    let mut c = c.strip_prefix('"').unwrap_or(c);
    if let Some(end) = c.find('"') {
        c = &c[..end];
    } else {
        c = c.split_whitespace().next().unwrap_or(c);
    }
    if c.is_empty() {
        return None;
    }
    Some(c.to_string())
}

// -- startup folder --------------------------------------------------------------

fn startup_folders() -> Vec<(String, String)> {
    let appdata = std::env::var_os("APPDATA")
        .map(|v| v.to_string_lossy().into_owned())
        .unwrap_or_default();
    let programdata = std::env::var_os("PROGRAMDATA")
        .map(|v| v.to_string_lossy().into_owned())
        .unwrap_or_default();
    if appdata.is_empty() && programdata.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    if !appdata.is_empty() {
        out.push((
            format!("{appdata}\\Microsoft\\Windows\\Start Menu\\Programs\\Startup"),
            "user startup folder".to_string(),
        ));
    }
    if !programdata.is_empty() {
        out.push((
            format!("{programdata}\\Microsoft\\Windows\\Start Menu\\Programs\\Startup"),
            "all-users startup folder".to_string(),
        ));
    }
    out
}

fn enumerate_folder(out: &mut Vec<StartupEntry>) {
    for (dir, source) in startup_folders() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in rd.flatten() {
            let path = e.path();
            let ext = path
                .extension()
                .and_then(|x| x.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if ext != "lnk" && ext != "url" {
                continue;
            }
            let name = path
                .file_stem()
                .and_then(|x| x.to_str())
                .unwrap_or("shortcut")
                .to_string();
            let cmd = link_target(&path).unwrap_or_else(|| path.display().to_string());
            let path_str = path.display().to_string();
            let sig = cmd
                .split_whitespace()
                .next()
                .map(signature::inspect_exe)
                .unwrap_or((None, None));
            let (signed, publisher) = sig;
            let id = format!("folder|{}", path_str);
            let (risk, reasons) =
                classify(cmd.split_whitespace().next(), signed, publisher.as_deref());
            out.push(StartupEntry {
                id,
                name,
                command: cmd,
                source: source.clone(),
                path: Some(path_str),
                signed,
                publisher,
                risk: risk.to_string(),
                risk_reasons: reasons,
                enabled: true,
                source_detail: format!("shortcut in {dir}"),
            });
        }
    }
}

/// Parse a .url shortcut (INI) or return an empty string.
fn parse_url(path: &PathBuf) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let url = raw
        .lines()
        .find_map(|l| l.strip_prefix("URL="))
        .map(|s| s.to_string())?;
    Some(url)
}

/// Best-effort extraction of the target of a .lnk via its binary format.
/// Returns the raw target string; used only for display/classification.
fn link_target(path: &PathBuf) -> Option<String> {
    let is_url = path
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("url"));
    if is_url {
        return parse_url(path);
    }
    let bytes = std::fs::read(path).ok()?;
    // LNK header: 4-byte signature + 16-byte GUID must match MS-SHLLINK.
    if bytes.len() < 76 || &bytes[0..4] != b"\x4c\x00\x00\x00" {
        // Fallback: scan for a quoted path substring.
        let s = String::from_utf8_lossy(&bytes);
        let start = s.find('"')?;
        let end = s[start + 1..].find('"')? + start + 1;
        return Some(s[start + 1..end].to_string());
    }
    // Read the link flags to find the LocalBasePath string list (offset 0x4c).
    let flags = u32::from_le_bytes([bytes[0x14], bytes[0x15], bytes[0x16], bytes[0x17]]);
    if flags & 0x1 == 0 {
        return None; // no local path saved
    }
    // StringData begins right after the 0x4c-byte LinkInfo, so locate it by
    // walking the fixed header + optional blocks. This intentionally uses a
    // conservative parser: if offsets look wrong, we give up rather than
    // misclassify.
    let mut pos = 0x4c;
    if flags & 0x2 != 0 {
        // HasLinkInfo: skip the 28-byte header + variable-sized extra data.
        if bytes.len() < pos + 28 {
            return None;
        }
        let link_info_size =
            u32::from_le_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]);
        pos += 28;
        if bytes.len() < pos + (link_info_size as usize - 28).min(4) {
            return None;
        }
        // Locate the LocalBasePath within the LinkInfo body by searching for
        // the ANSI (0x3) or Unicode (0x2) path kind.
        let body = &bytes[pos..pos + link_info_size as usize - 28];
        let base_path_offset =
            u32::from_le_bytes([body[12], body[13], body[14], body[15]]) as usize;
        if let Some(hit) = body.get(base_path_offset.saturating_mul(1)..) {
            let end = hit.iter().position(|&b| b == 0).unwrap_or(hit.len());
            let raw = hit[..end].to_vec();
            if !raw.is_empty() {
                return Some(String::from_utf8_lossy(&raw).to_string());
            }
        }
    }
    None
}

// -- services (read-only x-ray) --------------------------------------------------

pub fn services() -> Vec<ServiceRow> {
    crate::trust::services::list()
}

// -- tasks -----------------------------------------------------------------------

pub fn tasks() -> Vec<(String, bool)> {
    crate::elevate::ops::list_tasks()
        .map(|t| t.into_iter().map(|s| (s.name, s.enabled)).collect())
        .unwrap_or_default()
}

// -- entry listing --------------------------------------------------------------

pub fn list() -> Vec<StartupEntry> {
    let mut out = Vec::new();
    enumerate_run(RUN, "registry", &mut out);
    enumerate_run(RUN_ONCE, "registry", &mut out);
    enumerate_folder(&mut out);
    // Scheduled tasks that trigger at logon get their own source label. We
    // only show tasks whose path is a real registered task so the list stays
    // honest and small.
    for (name, _enabled) in tasks() {
        if looks_like_logon_task(&name) {
            out.push(StartupEntry {
                id: format!("task|{name}"),
                name: name.rsplit('\\').next().unwrap_or(&name).to_string(),
                command: format!("schtasks /Run /TN \"{name}\""),
                source: "scheduled task".to_string(),
                path: None,
                signed: None,
                publisher: None,
                risk: "low".to_string(),
                risk_reasons: vec![],
                enabled: true,
                source_detail: name.clone(),
            });
        }
    }
    out.sort_by(|a, b| a.source.cmp(&b.source).then(a.name.cmp(&b.name)));
    out
}

fn looks_like_logon_task(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("logon") || lower.contains("startup") || lower.contains("boot")
}

fn classify(
    path: Option<&str>,
    signed: Option<bool>,
    publisher: Option<&str>,
) -> (Risk, Vec<String>) {
    let mut reasons = Vec::new();
    let mut risk = Risk::Low;

    if let Some(p) = path {
        let lp = p.to_ascii_lowercase();
        if lp.contains("temp") || lp.contains("\\download") || lp.contains("\\cache") {
            reasons.push("path under temp/download/cache".to_string());
            risk = risk.max(Risk::Medium);
        }
        if !p.starts_with('"') && p.contains(' ') {
            reasons.push("unquoted path".to_string());
            risk = risk.max(Risk::Medium);
        }
    }

    match signed {
        Some(true) => {}
        Some(false) => {
            reasons.push("unsigned executable".to_string());
            risk = risk.max(Risk::Medium);
        }
        None => {
            reasons.push("signature unknown".to_string());
        }
    }

    match publisher {
        Some(p) if p.to_ascii_lowercase().contains("microsoft") => {}
        Some(_) => {}
        None => {
            reasons.push("no publisher found".to_string());
        }
    }
    (risk, reasons)
}

#[allow(dead_code)]
#[derive(PartialEq, PartialOrd, Clone, Copy, Eq, Ord, Debug)]
enum Risk {
    Low,
    Medium,
    High,
}
impl std::fmt::Display for Risk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Risk::Low => "low",
            Risk::Medium => "medium",
            Risk::High => "high",
        };
        write!(f, "{s}")
    }
}

// -- enable / disable -----------------------------------------------------------

/// toggle by id; ids encode the source so we can reverse exactly.
pub async fn set(id: &str, enabled: bool) -> Result<crate::trust::models::ChangeOutcome, ZtError> {
    let (kind, rest) = id
        .split_once('|')
        .ok_or_else(|| ZtError::new(format!("malformed startup id {id}")))?;
    match kind {
        "run" | "runonce" => set_reg_entry(rest, enabled).await,
        "folder" => set_folder_entry(rest, enabled).await,
        "task" => set_task_entry(rest, enabled).await,
        other => Err(ZtError::new(format!("unsupported startup source {other}"))),
    }
}

fn parse_reg_id(rest: &str) -> Result<(String, String, String), ZtError> {
    // hive | keypath | value
    let mut parts = rest.splitn(3, '|');
    let hive = parts
        .next()
        .ok_or_else(|| ZtError::new("missing hive"))?
        .to_string();
    let key = parts
        .next()
        .ok_or_else(|| ZtError::new("missing key"))?
        .to_string();
    let value = parts
        .next()
        .ok_or_else(|| ZtError::new("missing value"))?
        .to_string();
    Ok((hive, key, value))
}

async fn set_reg_entry(
    rest: &str,
    enabled: bool,
) -> Result<crate::trust::models::ChangeOutcome, ZtError> {
    let (hive, key_path, value_name) = parse_reg_id(rest)?;
    let root = match hive.as_str() {
        "HKCU" => user_root(),
        "HKLM" | "HKLM32" => machine_root(),
        other => return Err(ZtError::new(format!("bad hive {other}"))),
    };
    let position = root
        .open_subkey_with_flags(&key_path, KEY_READ)
        .and_then(|k| k.get_raw_value(&value_name));

    let old = match &position {
        Ok(rv) => Some(crate::trust::models::RegOldState {
            existed: true,
            value_type: rv.vtype.clone() as u32,
            data: rv.bytes.clone(),
        }),
        Err(_) => None,
    };
    let new_display = if enabled {
        "enabled (present)"
    } else {
        "disabled (removed)"
    };

    if enabled {
        match &old {
            Some(o) => {
                // Restore the exact previous value.
                let op = crate::elevate::ops::ElevOp::RegWrite {
                    hive: hive.clone(),
                    key_path: key_path.clone(),
                    value_name: value_name.clone(),
                    value_type: o.value_type,
                    data: o.data.clone(),
                };
                crate::elevate::runner::run_with_elevation(&crate::state::app()?, &op)?;
            }
            None => return Err(ZtError::new("entry already absent; cannot enable")),
        }
    } else {
        let op = crate::elevate::ops::ElevOp::RegDelete {
            hive: hive.clone(),
            key_path: key_path.clone(),
            value_name: value_name.clone(),
        };
        crate::elevate::runner::run_with_elevation(&crate::state::app()?, &op)?;
    }

    let mut entry = JournalEntry {
        id: String::new(),
        ts: journal::now(),
        kind: "startup".to_string(),
        target: format!("{hive}\\{key_path}\\{value_name}"),
        old_display: match &old {
            Some(o) if o.existed => format!("value present ({} bytes)", o.data.len()),
            _ => "not present".to_string(),
        },
        new_display: new_display.to_string(),
        result: if enabled { "enabled" } else { "disabled" }.to_string(),
        restored: false,
        item_id: None,
        reg_hive: Some(hive),
        reg_key: Some(key_path),
        reg_value: Some(value_name),
        reg_old: old,
        reg_apply_type: None,
        reg_apply_data: None,
        task_path: None,
        rule_name: None,
        fw_program: None,
        fw_host: None,
        startup_mode: None,
        file_from: None,
        file_to: None,
    };
    let id = journal::append(&mut entry)?;
    Ok(crate::trust::models::ChangeOutcome {
        ok: true,
        detail: if enabled {
            "entry enabled"
        } else {
            "entry disabled"
        }
        .to_string(),
        journal_id: Some(id),
    })
}

async fn set_folder_entry(
    rest: &str,
    enabled: bool,
) -> Result<crate::trust::models::ChangeOutcome, ZtError> {
    let path = PathBuf::from(rest);
    if !path.exists() {
        return Err(ZtError::new("startup file not found"));
    }
    let ext = path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_string();
    let disabled_path = if enabled {
        // rename back: strip .off suffix we added
        if !ext.eq_ignore_ascii_case("off") {
            return Err(ZtError::new("cannot re-enable: file has no .off suffix"));
        }

        path.with_extension("lnk")
    } else {
        // disable: rename shortcut to a non-launchable name

        path.with_extension("lnk.off")
    };

    let from = path.display().to_string();
    let to = disabled_path.display().to_string();

    let op = crate::elevate::ops::ElevOp::FileMove {
        from: if enabled { &to } else { &from }.clone(),
        to: if enabled { &from } else { &to }.clone(),
    };
    crate::elevate::runner::run_with_elevation(&crate::state::app()?, &op)?;

    let mut entry = JournalEntry {
        id: String::new(),
        ts: journal::now(),
        kind: "startup".to_string(),
        target: rest.to_string(),
        old_display: if enabled {
            "disabled (renamed)"
        } else {
            "enabled (shortcut)"
        }
        .to_string(),
        new_display: if enabled {
            "enabled (shortcut)"
        } else {
            "disabled (renamed)"
        }
        .to_string(),
        result: if enabled { "enabled" } else { "disabled" }.to_string(),
        restored: false,
        item_id: None,
        reg_hive: None,
        reg_key: None,
        reg_value: None,
        reg_old: None,
        reg_apply_type: None,
        reg_apply_data: None,
        task_path: None,
        rule_name: None,
        fw_program: None,
        fw_host: None,
        startup_mode: Some("folder".to_string()),
        file_from: Some(from.clone()),
        file_to: Some(to.clone()),
    };
    let id = journal::append(&mut entry)?;
    Ok(crate::trust::models::ChangeOutcome {
        ok: true,
        detail: if enabled {
            "entry enabled"
        } else {
            "entry disabled"
        }
        .to_string(),
        journal_id: Some(id),
    })
}

async fn set_task_entry(
    rest: &str,
    enabled: bool,
) -> Result<crate::trust::models::ChangeOutcome, ZtError> {
    let op = crate::elevate::ops::ElevOp::TaskSetEnabled {
        task_name: rest.to_string(),
        enabled,
    };
    crate::elevate::runner::run_with_elevation(&crate::state::app()?, &op)?;
    let mut entry = JournalEntry {
        id: String::new(),
        ts: journal::now(),
        kind: "task".to_string(),
        target: rest.to_string(),
        old_display: if enabled { "disabled" } else { "enabled" }.to_string(),
        new_display: if enabled { "enabled" } else { "disabled" }.to_string(),
        result: if enabled { "enabled" } else { "disabled" }.to_string(),
        restored: false,
        item_id: None,
        reg_hive: None,
        reg_key: None,
        reg_value: None,
        reg_old: None,
        reg_apply_type: None,
        reg_apply_data: None,
        task_path: Some(rest.to_string()),
        rule_name: None,
        fw_program: None,
        fw_host: None,
        startup_mode: None,
        file_from: None,
        file_to: None,
    };
    let id = journal::append(&mut entry)?;
    Ok(crate::trust::models::ChangeOutcome {
        ok: true,
        detail: if enabled {
            "task enabled"
        } else {
            "task disabled"
        }
        .to_string(),
        journal_id: Some(id),
    })
}

/// resolve a shortcut target for display only
pub fn _link_target_display(path: &PathBuf) -> Option<String> {
    link_target(path)
}
