//! Telemetry & bloatware catalog: a curated list of registry/task changes with
//! honest tiering. Reads are unprivileged; writes are journaled, restore-pointed,
//! and executed through the elevation runner. Guarded/info items are
//! informational only — never actionable.

use crate::core::ZtError;
use crate::elevate::ops::{ElevOp, ElevResponse};
use crate::trust::journal;
use crate::trust::models::{CatalogItem, Category, ChangeOutcome, JournalEntry, RegOldState, Tier};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

const HKCU: &str = "HKCU";
const HKLM: &str = "HKLM";

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
enum TargetKind {
    Dword { target: u32 },
    Str { target: &'static str },
    Delete,
    Task,
    Info,
}

#[derive(Debug)]
struct Spec {
    id: &'static str,
    category: Category,
    title: &'static str,
    description: &'static str,
    tier: Tier,
    guarded: bool,
    hive: &'static str,
    key: &'static str,
    value: &'static str,
    kind: TargetKind,
}

const SPECS: &[Spec] = &[
    // ---------------------------------------------------------------- telemetry
    Spec {
        id: "telemetry.allow",
        category: Category::Telemetry,
        title: "Telemetry opt-out (AllowTelemetry)",
        description: "Writes the enterprise telemetry opt-out policy value. Reduces diagnostic data collection to the minimum.",
        tier: Tier::Recommended,
        guarded: false,
        hive: HKLM,
        key: r"SOFTWARE\Policies\Microsoft\Windows\DataCollection",
        value: "AllowTelemetry",
        kind: TargetKind::Dword { target: 0 },
    },
    Spec {
        id: "telemetry.appraiser",
        category: Category::Telemetry,
        title: "Compatibility Appraiser task",
        description: "Microsoft Compatibility Appraiser uploads device inventories and app compatibility telemetry. Disabling stops the scheduled upload.",
        tier: Tier::Recommended,
        guarded: false,
        hive: "TASK",
        key: r"\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser",
        value: "",
        kind: TargetKind::Task,
    },
    Spec {
        id: "telemetry.programdata",
        category: Category::Telemetry,
        title: "ProgramDataUpdater task",
        description: "Scheduled task that sends telemetry during maintenance windows.",
        tier: Tier::Recommended,
        guarded: false,
        hive: "TASK",
        key: r"\Microsoft\Windows\Application Experience\ProgramDataUpdater",
        value: "",
        kind: TargetKind::Task,
    },
    Spec {
        id: "telemetry.census",
        category: Category::Telemetry,
        title: "MergeAppCacheUpdate task",
        description: "Census-style cache update task tied to device telemetry.",
        tier: Tier::Limited,
        guarded: false,
        hive: "TASK",
        key: r"\Microsoft\Windows\Application Experience\StartupAppTask",
        value: "",
        kind: TargetKind::Task,
    },
    Spec {
        id: "telemetry.wers",
        category: Category::Telemetry,
        title: "Windows Error Reporting queue",
        description: "QueueReporting task forwards crash telemetry to Microsoft.",
        tier: Tier::Limited,
        guarded: false,
        hive: "TASK",
        key: r"\Microsoft\Windows\Windows Error Reporting\QueueReporting",
        value: "",
        kind: TargetKind::Task,
    },
    // ------------------------------------------------------------------ privacy
    Spec {
        id: "privacy.activity",
        category: Category::Privacy,
        title: "Share activity history",
        description: "PublishActivities controls cross-device activity sharing to Microsoft.",
        tier: Tier::Recommended,
        guarded: false,
        hive: HKCU,
        key: r"Software\Microsoft\Windows\CurrentVersion\Privacy\ActivityHistory",
        value: "PublishActivities",
        kind: TargetKind::Dword { target: 0 },
    },
    Spec {
        id: "privacy.timeline",
        category: Category::Privacy,
        title: "Enable activity feed",
        description: "EnableActivityFeed powers the Timeline feature which stores local activity.",
        tier: Tier::Recommended,
        guarded: false,
        hive: HKCU,
        key: r"Software\Microsoft\Windows\CurrentVersion\Privacy\ActivityHistory",
        value: "EnableActivityFeed",
        kind: TargetKind::Dword { target: 0 },
    },
    // ---------------------------------------------------------------------- ads
    Spec {
        id: "ads.advertising-id",
        category: Category::Ads,
        title: "Advertising ID",
        description: "Enabled (1) lets apps use the per-user advertising ID for ad targeting.",
        tier: Tier::Recommended,
        guarded: false,
        hive: HKCU,
        key: r"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo",
        value: "Enabled",
        kind: TargetKind::Dword { target: 0 },
    },
    Spec {
        id: "ads.tailored",
        category: Category::Ads,
        title: "Tailored experiences",
        description: "Disables using diagnostic data to customize ads and suggestions.",
        tier: Tier::Recommended,
        guarded: false,
        hive: HKLM,
        key: r"SOFTWARE\Policies\Microsoft\Windows\CloudContent",
        value: "DisableTailoredExperiencesWithDiagnosticData",
        kind: TargetKind::Dword { target: 1 },
    },
    Spec {
        id: "ads.suggestions",
        category: Category::Ads,
        title: "Settings suggestions",
        description: "Suppresses promotional suggestions inside the Settings app.",
        tier: Tier::Limited,
        guarded: false,
        hive: HKCU,
        key: r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
        value: "SystemPaneSuggestionsEnabled",
        kind: TargetKind::Dword { target: 0 },
    },
    // ----------------------------------------------------------------------- ai
    Spec {
        id: "ai.copilot",
        category: Category::Ai,
        title: "Windows Copilot",
        description: "Turns off the Copilot policy switch on supported builds.",
        tier: Tier::Recommended,
        guarded: false,
        hive: HKLM,
        key: r"SOFTWARE\Policies\Microsoft\Windows\WindowsCopilot",
        value: "TurnOffWindowsCopilot",
        kind: TargetKind::Dword { target: 1 },
    },
    Spec {
        id: "ai.recall",
        category: Category::Ai,
        title: "Recall (AI activity history)",
        description: "Disables Recall's periodic local AI analysis on Copilot+ hardware.",
        tier: Tier::Recommended,
        guarded: false,
        hive: HKLM,
        key: r"SOFTWARE\Policies\Microsoft\Windows\WindowsAI",
        value: "DisableAIDataAnalysis",
        kind: TargetKind::Dword { target: 1 },
    },
    // ------------------------------------------------------------------ cortana
    Spec {
        id: "cortana.allow",
        category: Category::Cortana,
        title: "Allow Cortana",
        description: "Enterprise policy that disables Cortana entirely.",
        tier: Tier::Recommended,
        guarded: false,
        hive: HKLM,
        key: r"SOFTWARE\Policies\Microsoft\Windows\Windows Search",
        value: "AllowCortana",
        kind: TargetKind::Dword { target: 0 },
    },
    Spec {
        id: "cortana.search-location",
        category: Category::Cortana,
        title: "Search uses location",
        description: "Stops web/personalized search from using your location.",
        tier: Tier::Limited,
        guarded: false,
        hive: HKLM,
        key: r"SOFTWARE\Policies\Microsoft\Windows\Windows Search",
        value: "AllowSearchToUseLocation",
        kind: TargetKind::Dword { target: 0 },
    },
    Spec {
        id: "cortana.websearch",
        category: Category::Cortana,
        title: "Web search in Start",
        description: "Disables web results in the Start menu search.",
        tier: Tier::Limited,
        guarded: false,
        hive: HKLM,
        key: r"SOFTWARE\Policies\Microsoft\Windows\Windows Search",
        value: "DisableWebSearch",
        kind: TargetKind::Dword { target: 1 },
    },
    // ----------------------------------------------------------------- location
    Spec {
        id: "location.master",
        category: Category::Location,
        title: "Location services",
        description: "Denies all apps location access via the consent store master switch.",
        tier: Tier::Recommended,
        guarded: false,
        hive: HKLM,
        key: r"SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\location",
        value: "Value",
        kind: TargetKind::Str { target: "Deny" },
    },
    // ------------------------------------------------------------- app privacy
    Spec {
        id: "app.mic",
        category: Category::AppPrivacy,
        title: "Microphone access",
        description: "Denies all apps microphone access via the consent store master switch.",
        tier: Tier::Recommended,
        guarded: false,
        hive: HKLM,
        key: r"SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone",
        value: "Value",
        kind: TargetKind::Str { target: "Deny" },
    },
    Spec {
        id: "app.camera",
        category: Category::AppPrivacy,
        title: "Webcam access",
        description: "Denies all apps camera access via the consent store master switch.",
        tier: Tier::Recommended,
        guarded: false,
        hive: HKLM,
        key: r"SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\webcam",
        value: "Value",
        kind: TargetKind::Str { target: "Deny" },
    },
    Spec {
        id: "app.notifications",
        category: Category::AppPrivacy,
        title: "Notifications access",
        description: "Denies apps the ability to send notifications system-wide.",
        tier: Tier::Limited,
        guarded: false,
        hive: HKLM,
        key: r"SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\notifications",
        value: "Value",
        kind: TargetKind::Str { target: "Deny" },
    },
    // ------------------------------------------------------------------- system
    Spec {
        id: "system.smart-screen",
        category: Category::System,
        title: "SmartScreen",
        description: "Disabling SmartScreen removes a phishing-safety layer and is not recommended. Shown for visibility only.",
        tier: Tier::NotRecommended,
        guarded: true,
        hive: HKCU,
        key: r"Software\Microsoft\Windows\CurrentVersion\Explorer",
        value: "SmartScreenEnabled",
        kind: TargetKind::Str { target: "Off" },
    },
    Spec {
        id: "system.gamebar",
        category: Category::System,
        title: "Xbox Game Bar auto",
        description: "Disables Game Bar's automated launch behavior.",
        tier: Tier::Limited,
        guarded: false,
        hive: HKCU,
        key: r"Software\Microsoft\GameBar",
        value: "AutoGameModeEnabled",
        kind: TargetKind::Dword { target: 0 },
    },
    Spec {
        id: "system.maps-update",
        category: Category::System,
        title: "Maps auto-update task",
        description: "Scheduled download of offline maps data in the background.",
        tier: Tier::Limited,
        guarded: false,
        hive: "TASK",
        key: r"\Microsoft\Windows\Maps\MapsUpdateTask",
        value: "",
        kind: TargetKind::Task,
    },
    Spec {
        id: "system.windows-update",
        category: Category::System,
        title: "Windows Update",
        description: "Windows Update is a system-critical service. Informational only; ZeroTrace never disables it.",
        tier: Tier::Info,
        guarded: true,
        hive: "INFO",
        key: "",
        value: "",
        kind: TargetKind::Info,
    },
    Spec {
        id: "system.defender",
        category: Category::System,
        title: "Microsoft Defender",
        description: "Defender is your real-time AV. Informational only; ZeroTrace never disables it.",
        tier: Tier::Info,
        guarded: true,
        hive: "INFO",
        key: "",
        value: "",
        kind: TargetKind::Info,
    },
    Spec {
        id: "system.activation",
        category: Category::System,
        title: "Windows activation",
        description: "Activation state is protected system state. Informational only.",
        tier: Tier::Info,
        guarded: true,
        hive: "INFO",
        key: "",
        value: "",
        kind: TargetKind::Info,
    },
];

pub fn catalog() -> Vec<CatalogItem> {
    SPECS
        .iter()
        .map(|s| {
            let (current, applied) = current_state(s);
            let (desired, kind) = match s.kind {
                TargetKind::Dword { target } => (format!("{target}"), "dword".to_string()),
                TargetKind::Str { target } => (target.to_string(), "string".to_string()),
                TargetKind::Delete => ("value removed".to_string(), "delete".to_string()),
                TargetKind::Task => ("task disabled".to_string(), "task".to_string()),
                TargetKind::Info => ("—".to_string(), "info".to_string()),
            };
            CatalogItem {
                id: s.id.to_string(),
                category: s.category,
                title: s.title.to_string(),
                description: s.description.to_string(),
                tier: s.tier,
                guarded: s.guarded,
                kind,
                target_display: display_target(s),
                current_display: current,
                desired_display: desired,
                applied,
            }
        })
        .collect()
}

fn display_target(s: &Spec) -> String {
    match s.hive {
        "TASK" => format!("scheduled task {}", s.key),
        "INFO" => s.title.to_string(),
        hive => format!("{hive}\\{}\\{}", s.key, s.value),
    }
}

fn current_state(s: &Spec) -> (String, bool) {
    match s.hive {
        "TASK" => {
            let enabled = match crate::elevate::ops::list_tasks() {
                Ok(tasks) => tasks
                    .iter()
                    .find(|t| t.name == s.key.trim_start_matches('\\'))
                    .map(|t| t.enabled),
                Err(_) => None,
            };
            match enabled {
                Some(true) => ("enabled".to_string(), false),
                Some(false) => ("disabled".to_string(), true),
                None => ("not found".to_string(), false),
            }
        }
        "INFO" => ("—".to_string(), false),
        _ => {
            let (existed, value_type, data) = read_raw(s.hive, s.key, s.value);
            if !existed {
                let desired = match s.kind {
                    TargetKind::Dword { target } => target == 0,
                    TargetKind::Str { target } => target == "Allow",
                    TargetKind::Delete => true,
                    _ => false,
                };
                return ("not set".to_string(), desired);
            }
            match s.kind {
                TargetKind::Dword { target } => {
                    let val = dword_from_data(&data).unwrap_or(0);
                    (format!("{} (dword)", val), val == target)
                }
                TargetKind::Str { target } => {
                    let val = str_from_data(&data);
                    (format!("{val:?}"), val.trim_end_matches('\0') == target)
                }
                TargetKind::Delete => ("present".to_string(), false),
                _ => (format!("(type {value_type})"), false),
            }
        }
    }
}

fn read_raw(hive: &str, key_path: &str, value: &str) -> (bool, u32, Vec<u8>) {
    let root = match hive {
        "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
        "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
        _ => return (false, 0, Vec::new()),
    };
    match root.open_subkey_with_flags(key_path, KEY_READ) {
        Ok(key) => match key.get_raw_value(value) {
            Ok(rv) => (true, rv.vtype as u32, rv.bytes),
            Err(_) => (false, 0, Vec::new()),
        },
        Err(_) => (false, 0, Vec::new()),
    }
}

fn dword_from_data(data: &[u8]) -> Option<u32> {
    if data.len() != 4 {
        return None;
    }
    Some(u32::from_ne_bytes([data[0], data[1], data[2], data[3]]))
}

fn str_from_data(data: &[u8]) -> String {
    let mut u16s: Vec<u16> = data
        .chunks(2)
        .map(|c| u16::from_ne_bytes([c[0], c[1]]))
        .collect();
    let end = u16s.iter().position(|&c| c == 0).unwrap_or(u16s.len());
    u16s.truncate(end);
    String::from_utf16_lossy(&u16s)
}

/// Encode REG_SZ / REG_DWORD / etc. payload bytes for a target state.
fn encode(value_type: u32, s: &Spec) -> Vec<u8> {
    match value_type {
        1 | 2 => {
            let target_str = match s.kind {
                TargetKind::Str { target } => target,
                _ => "",
            };
            let mut v: Vec<u16> = target_str.encode_utf16().collect();
            v.push(0);
            v.iter().flat_map(|c| c.to_ne_bytes()).collect()
        }
        4 => match s.kind {
            TargetKind::Dword { target } => target.to_ne_bytes().to_vec(),
            _ => 0u32.to_ne_bytes().to_vec(),
        },
        _ => Vec::new(),
    }
}

// -- apply / revert -----------------------------------------------------------

pub async fn apply(ids: Vec<String>) -> Result<ChangeOutcome, ZtError> {
    let specs: Vec<&Spec> = ids
        .iter()
        .filter_map(|id| SPECS.iter().find(|s| s.id == id))
        .collect();
    if specs.is_empty() {
        return Err(ZtError::new("no recognized items"));
    }
    if specs
        .iter()
        .any(|s| s.guarded || matches!(s.kind, TargetKind::Info))
    {
        return Err(ZtError::new(
            "refusing to act on guarded/informational items",
        ));
    }

    // A restore point is attempted before any change.
    run_op(ElevOp::RestorePoint {
        description: format!("ZeroTrace: {} change(s)", specs.len()),
    })
    .await?;

    for spec in &specs {
        let _ = apply_one(spec).await?;
    }
    Ok(ChangeOutcome {
        ok: true,
        detail: format!("applied {} item(s)", specs.len()),
        journal_id: None,
    })
}

async fn apply_one(spec: &Spec) -> Result<ChangeOutcome, ZtError> {
    let (existed, old_type) = match spec.hive {
        "TASK" | "INFO" => (false, 0u32),
        _ => {
            let (existed, value_type, _) = read_raw(spec.hive, spec.key, spec.value);
            (existed, value_type)
        }
    };

    let old = if spec.hive != "TASK" && spec.hive != "INFO" && existed {
        let (_, _, data) = read_raw(spec.hive, spec.key, spec.value);
        Some(RegOldState {
            existed: true,
            value_type: old_type,
            data,
        })
    } else {
        None
    };

    let op = match spec.kind {
        TargetKind::Task => ElevOp::TaskSetEnabled {
            task_name: spec.key.trim_start_matches('\\').to_string(),
            enabled: false,
        },
        TargetKind::Dword { .. } | TargetKind::Str { .. } => ElevOp::RegWrite {
            hive: spec.hive.to_string(),
            key_path: spec.key.to_string(),
            value_name: spec.value.to_string(),
            value_type: type_of(spec.kind),
            data: encode(type_of(spec.kind), spec),
        },
        TargetKind::Delete => ElevOp::RegDelete {
            hive: spec.hive.to_string(),
            key_path: spec.key.to_string(),
            value_name: spec.value.to_string(),
        },
        TargetKind::Info => {
            return Err(ZtError::new("information item is not actionable"));
        }
    };

    let resp = run_op(op).await?;
    if !resp.ok {
        return Err(ZtError::new(resp.detail));
    }

    let mut entry = JournalEntry {
        id: String::new(),
        ts: String::new(),
        kind: if spec.hive == "TASK" {
            "task".into()
        } else {
            "registry".into()
        },
        target: display_target(spec),
        old_display: current_display_short(spec, &old),
        new_display: desired_display_short(spec).to_string(),
        result: resp.detail.clone(),
        restored: false,
        item_id: Some(spec.id.to_string()),
        reg_hive: (spec.hive != "TASK" && spec.hive != "INFO").then(|| spec.hive.to_string()),
        reg_key: (spec.hive != "TASK" && spec.hive != "INFO").then(|| spec.key.to_string()),
        reg_value: (spec.hive != "TASK" && spec.hive != "INFO").then(|| spec.value.to_string()),
        reg_old: old.clone(),
        reg_apply_type: (spec.hive != "TASK" && spec.hive != "INFO").then(|| type_of(spec.kind)),
        reg_apply_data: (spec.hive != "TASK"
            && spec.hive != "INFO"
            && !matches!(spec.kind, TargetKind::Delete))
        .then(|| encode(type_of(spec.kind), spec)),
        task_path: (spec.hive == "TASK").then(|| spec.key.trim_start_matches('\\').to_string()),
        rule_name: None,
        fw_program: None,
        fw_host: None,
        startup_mode: None,
        file_from: None,
        file_to: None,
    };
    let id = journal::append(&mut entry)?;

    Ok(ChangeOutcome {
        ok: true,
        detail: resp.detail,
        journal_id: Some(id),
    })
}

fn current_display_short(spec: &Spec, old: &Option<RegOldState>) -> String {
    match spec.hive {
        "TASK" => "enabled".to_string(),
        _ => match old {
            Some(o) if o.existed => match spec.kind {
                TargetKind::Dword { .. } => {
                    format!("{} (dword)", dword_from_data(&o.data).unwrap_or(0))
                }
                TargetKind::Str { .. } => format!("{:?}", str_from_data(&o.data)),
                _ => "present".to_string(),
            },
            _ => "not set".to_string(),
        },
    }
}

fn desired_display_short(spec: &Spec) -> &'static str {
    match spec.kind {
        TargetKind::Task => "disabled",
        TargetKind::Dword { .. } => "target dword",
        TargetKind::Str { .. } => "target string",
        TargetKind::Delete => "removed",
        TargetKind::Info => "—",
    }
}

fn type_of(kind: TargetKind) -> u32 {
    match kind {
        TargetKind::Str { .. } => 1,   // REG_SZ
        TargetKind::Dword { .. } => 4, // REG_DWORD
        _ => 0,
    }
}

/// Revert one journaled change: fire the inverse elevated op and mark
/// the entry restored.
pub async fn revert(journal_id: String) -> Result<ChangeOutcome, ZtError> {
    let entries = journal::read_all()?;
    let entry = entries
        .iter()
        .find(|e| e.id == journal_id && !e.restored)
        .ok_or_else(|| ZtError::new("no unrestored journal entry with this id"))?;

    let op = reverse_op(entry)?;
    let resp = run_op(op).await?;
    if !resp.ok {
        return Ok(ChangeOutcome {
            ok: false,
            detail: resp.detail,
            journal_id: Some(journal_id.clone()),
        });
    }
    let updated = journal::mark_restored(&journal_id)?;
    Ok(ChangeOutcome {
        ok: true,
        detail: format!("reverted {} (was {})", updated.target, updated.new_display),
        journal_id: Some(journal_id),
    })
}

fn reverse_op(entry: &JournalEntry) -> Result<ElevOp, ZtError> {
    match entry.kind.as_str() {
        "registry" => {
            let hive = entry
                .reg_hive
                .as_deref()
                .ok_or_else(|| ZtError::new("journal missing registry hive"))?;
            let key = entry
                .reg_key
                .as_deref()
                .ok_or_else(|| ZtError::new("journal missing registry key"))?;
            let value = entry
                .reg_value
                .as_deref()
                .ok_or_else(|| ZtError::new("journal missing registry value"))?;
            match &entry.reg_old {
                Some(old) if old.existed => Ok(ElevOp::RegWrite {
                    hive: hive.to_string(),
                    key_path: key.to_string(),
                    value_name: value.to_string(),
                    value_type: old.value_type,
                    data: old.data.clone(),
                }),
                _ => Ok(ElevOp::RegDelete {
                    hive: hive.to_string(),
                    key_path: key.to_string(),
                    value_name: value.to_string(),
                }),
            }
        }
        "task" => {
            let name = entry
                .task_path
                .as_deref()
                .ok_or_else(|| ZtError::new("journal missing task path"))?;
            Ok(ElevOp::TaskSetEnabled {
                task_name: name.to_string(),
                enabled: true,
            })
        }
        "firewall" => {
            let name = entry
                .rule_name
                .as_deref()
                .ok_or_else(|| ZtError::new("journal missing rule name"))?;
            Ok(ElevOp::FirewallRemove {
                name: name.to_string(),
            })
        }
        "startup" => {
            if entry.startup_mode.as_deref() == Some("folder") {
                let from = entry
                    .file_to
                    .as_deref()
                    .ok_or_else(|| ZtError::new("journal missing startup file target"))?;
                let to = entry
                    .file_from
                    .as_deref()
                    .ok_or_else(|| ZtError::new("journal missing startup file origin"))?;
                Ok(ElevOp::FileMove {
                    from: from.to_string(),
                    to: to.to_string(),
                })
            } else {
                let hive = entry
                    .reg_hive
                    .as_deref()
                    .ok_or_else(|| ZtError::new("journal missing startup hive"))?;
                let key = entry
                    .reg_key
                    .as_deref()
                    .ok_or_else(|| ZtError::new("journal missing startup key"))?;
                let value = entry
                    .reg_value
                    .as_deref()
                    .ok_or_else(|| ZtError::new("journal missing startup value"))?;
                match &entry.reg_old {
                    Some(old) if old.existed => Ok(ElevOp::RegWrite {
                        hive: hive.to_string(),
                        key_path: key.to_string(),
                        value_name: value.to_string(),
                        value_type: old.value_type,
                        data: old.data.clone(),
                    }),
                    _ => Ok(ElevOp::RegDelete {
                        hive: hive.to_string(),
                        key_path: key.to_string(),
                        value_name: value.to_string(),
                    }),
                }
            }
        }
        other => Err(ZtError::new(format!("no revert for journal kind {other}"))),
    }
}

/// Run an op: in-process when already elevated, otherwise through the helper.
pub async fn run_op(op: ElevOp) -> Result<ElevResponse, ZtError> {
    let app = crate::state::app_handle()
        .ok_or_else(|| ZtError::new("application state not available"))?;
    crate::elevate::runner::run_with_elevation(&app, &op)
}
