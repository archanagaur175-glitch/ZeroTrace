//! ZeroTrace Tauri backend entry point.

pub mod core;
pub mod elevate;
pub mod network;
pub mod startup;
pub mod state;
pub mod telemetry;
pub mod trust;

use tauri::{AppHandle, Emitter, State};
use trust::models::*;

use crate::core::ZtError;
use crate::network::Tracker;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

struct AppState {
    tracker: std::sync::Mutex<Tracker>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            tracker: std::sync::Mutex::new(Tracker::new()),
        }
    }
}

// ---------------------------------------------------------------------------
// Commands: radar
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_snapshot(app_state: State<'_, AppState>) -> Result<NetSnapshot, ZtError> {
    let settings = state::settings();
    let snap = app_state
        .tracker
        .lock()
        .map_err(|_| ZtError::new("tracker lock poisoned"))?
        .snapshot(settings.poll_interval_ms, settings.reverse_dns_enabled);
    Ok(snap)
}

#[tauri::command]
fn set_poll_interval(ms: u32) -> Result<(), ZtError> {
    state::update_settings(|s| s.poll_interval_ms = ms.max(250))?;
    Ok(())
}

#[tauri::command]
fn resolve_host(ip: String) -> Result<Option<String>, ZtError> {
    let host = crate::network::resolver::resolve_ip(&ip)?;
    Ok(host)
}

#[tauri::command]
fn get_local_interfaces() -> Result<Vec<LocalInterface>, ZtError> {
    crate::network::interfaces::local_interfaces()
}

#[tauri::command]
fn get_process_info(pid: u32) -> Result<ProcessDetail, ZtError> {
    Ok(match crate::network::process::resolve_pid(pid) {
        Some(info) => {
            let (signed, publisher) = info
                .path
                .as_deref()
                .map(crate::trust::signature::inspect)
                .unwrap_or((None, None));
            ProcessDetail {
                pid,
                name: info.name,
                path: info.path,
                publisher,
                signed,
            }
        }
        None => ProcessDetail {
            pid,
            name: format!("[{pid}]"),
            path: None,
            publisher: None,
            signed: None,
        },
    })
}

#[tauri::command]
fn signature_status(path: String) -> Result<SignatureStatus, ZtError> {
    Ok(crate::trust::signature::signature_status(&path))
}

// ---------------------------------------------------------------------------
// Commands: firewall
// ---------------------------------------------------------------------------

#[tauri::command]
async fn block_process_outbound(
    app: AppHandle,
    path: String,
    display_name: String,
) -> Result<ChangeOutcome, ZtError> {
    let name = crate::network::firewall::rule_name_for_program(&display_name);
    let op = crate::elevate::ops::ElevOp::FirewallAdd {
        name: name.clone(),
        program: Some(path.clone()),
        host: None,
    };
    let resp = crate::elevate::runner::run_with_elevation(&app, &op)?;
    if !resp.ok {
        return Ok(ChangeOutcome {
            ok: false,
            detail: resp.detail,
            journal_id: None,
        });
    }
    let mut entry = journal_entry(
        "firewall",
        name.clone(),
        "no rule",
        "block outbound",
        resp.detail.clone(),
    );
    entry.rule_name = Some(name.clone());
    entry.fw_program = Some(path);
    let id = crate::trust::journal::append(&mut entry)?;
    Ok(ChangeOutcome {
        ok: true,
        detail: resp.detail,
        journal_id: Some(id),
    })
}

#[tauri::command]
async fn unblock_process_outbound(
    app: AppHandle,
    rule_name: String,
) -> Result<ChangeOutcome, ZtError> {
    let op = crate::elevate::ops::ElevOp::FirewallRemove {
        name: rule_name.clone(),
    };
    let resp = crate::elevate::runner::run_with_elevation(&app, &op)?;
    Ok(ChangeOutcome {
        ok: resp.ok,
        detail: resp.detail,
        journal_id: None,
    })
}

#[tauri::command]
async fn block_host(app: AppHandle, host: String) -> Result<ChangeOutcome, ZtError> {
    let name = crate::network::firewall::rule_name_for_host(&host);
    let op = crate::elevate::ops::ElevOp::FirewallAdd {
        name: name.clone(),
        program: None,
        host: Some(host.clone()),
    };
    let resp = crate::elevate::runner::run_with_elevation(&app, &op)?;
    if !resp.ok {
        return Ok(ChangeOutcome {
            ok: false,
            detail: resp.detail,
            journal_id: None,
        });
    }
    let mut entry = journal_entry(
        "firewall",
        name.clone(),
        "no rule",
        "block host",
        resp.detail.clone(),
    );
    entry.rule_name = Some(name.clone());
    entry.fw_host = Some(host);
    let id = crate::trust::journal::append(&mut entry)?;
    Ok(ChangeOutcome {
        ok: true,
        detail: resp.detail,
        journal_id: Some(id),
    })
}

#[tauri::command]
async fn unblock_host(app: AppHandle, host: String) -> Result<ChangeOutcome, ZtError> {
    let name = crate::network::firewall::rule_name_for_host(&host);
    let op = crate::elevate::ops::ElevOp::FirewallRemove { name };
    let resp = crate::elevate::runner::run_with_elevation(&app, &op)?;
    Ok(ChangeOutcome {
        ok: resp.ok,
        detail: resp.detail,
        journal_id: None,
    })
}

#[tauri::command]
fn list_zerotrace_rules() -> Result<Vec<FirewallRule>, ZtError> {
    let rules = crate::network::firewall::list_rules()?;
    Ok(rules
        .into_iter()
        .filter(|r| r.name.starts_with(crate::network::firewall::ZT_PREFIX))
        .collect())
}

#[tauri::command]
fn is_rule_active(name: String) -> Result<bool, ZtError> {
    Ok(crate::network::firewall::rule_exists(&name))
}

// ---------------------------------------------------------------------------
// Commands: telemetry / undo
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_telemetry_catalog() -> Result<Vec<CatalogItem>, ZtError> {
    Ok(crate::telemetry::catalog())
}

#[tauri::command]
async fn apply_telemetry_changes(ids: Vec<String>) -> Result<ChangeOutcome, ZtError> {
    crate::telemetry::apply(ids).await
}

#[tauri::command]
async fn undo_change(journal_id: String) -> Result<ChangeOutcome, ZtError> {
    crate::telemetry::revert(journal_id).await
}

#[tauri::command]
fn get_undo_center() -> Result<Vec<JournalEntry>, ZtError> {
    crate::trust::journal::read_all()
}

#[tauri::command]
fn create_restore_point() -> Result<RestorePointResult, ZtError> {
    crate::trust::restore::create("ZeroTrace pre-change restore point")
}

#[tauri::command]
fn export_journal() -> Result<String, ZtError> {
    crate::trust::journal::export(5)
}

#[tauri::command]
fn open_repo() -> Result<(), ZtError> {
    let _ = crate::trust::provenance::REPO;
    let url = format!("https://{}", crate::trust::provenance::REPO);
    open_url(&url)
}

// ---------------------------------------------------------------------------
// Commands: startup / services / trust
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_startup_entries() -> Result<Vec<StartupEntry>, ZtError> {
    Ok(crate::startup::list())
}

#[tauri::command]
async fn disable_startup_entry(id: String) -> Result<ChangeOutcome, ZtError> {
    crate::startup::set(&id, false).await
}

#[tauri::command]
async fn enable_startup_entry(id: String) -> Result<ChangeOutcome, ZtError> {
    crate::startup::set(&id, true).await
}

#[tauri::command]
fn get_service_xray() -> Result<Vec<ServiceRow>, ZtError> {
    Ok(crate::trust::services::list())
}

#[tauri::command]
fn get_build_info() -> Result<BuildInfo, ZtError> {
    Ok(crate::trust::provenance::build_info())
}

#[tauri::command]
fn get_dependency_audit() -> Result<DepAudit, ZtError> {
    Ok(crate::trust::provenance::dependency_audit())
}

#[tauri::command]
fn is_elevated() -> Result<bool, ZtError> {
    Ok(crate::elevate::is_elevated())
}

// ---------------------------------------------------------------------------
// Commands: settings
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_settings() -> Result<AppSettings, ZtError> {
    Ok(state::settings())
}

#[tauri::command]
fn set_settings(settings: AppSettings) -> Result<(), ZtError> {
    state::update_settings(|s| {
        s.poll_interval_ms = settings.poll_interval_ms.clamp(250, 60_000);
        s.reverse_dns_enabled = settings.reverse_dns_enabled;
    })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn journal_entry(kind: &str, target: String, old: &str, new: &str, result: String) -> JournalEntry {
    JournalEntry {
        id: String::new(),
        ts: String::new(),
        kind: kind.to_string(),
        target,
        old_display: old.to_string(),
        new_display: new.to_string(),
        result,
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
        startup_mode: None,
        file_from: None,
        file_to: None,
    }
}

fn open_url(url: &str) -> Result<(), ZtError> {
    std::process::Command::new("rundll32.exe")
        .arg("url.dll,FileProtocolHandler")
        .arg(url)
        .spawn()
        .map_err(|e| ZtError::new(e.to_string()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Setup: background radar loop
// ---------------------------------------------------------------------------

fn spawn_radar_loop(app: AppHandle) {
    std::thread::Builder::new()
        .name("zerotrace-radar".into())
        .spawn(move || {
            let mut tracker = Tracker::new();
            loop {
                let settings = state::settings();
                let snap =
                    tracker.snapshot(settings.poll_interval_ms, settings.reverse_dns_enabled);
                let _ = app.emit("net::snapshot", &snap);
                std::thread::sleep(std::time::Duration::from_millis(
                    settings.poll_interval_ms as u64,
                ));
            }
        })
        .expect("failed to spawn radar loop");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|app| {
            state::init(app.handle());
            spawn_radar_loop(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            set_poll_interval,
            resolve_host,
            get_local_interfaces,
            get_process_info,
            signature_status,
            block_process_outbound,
            unblock_process_outbound,
            block_host,
            unblock_host,
            list_zerotrace_rules,
            is_rule_active,
            get_telemetry_catalog,
            apply_telemetry_changes,
            undo_change,
            get_undo_center,
            create_restore_point,
            export_journal,
            open_repo,
            get_startup_entries,
            disable_startup_entry,
            enable_startup_entry,
            get_service_xray,
            get_build_info,
            get_dependency_audit,
            is_elevated,
            get_settings,
            set_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ZeroTrace");
}
