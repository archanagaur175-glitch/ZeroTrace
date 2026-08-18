//! Global application handle + settings storage, set once at startup.

use crate::core::ZtError;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager};

use crate::trust::models::AppSettings;

static APP: OnceLock<AppHandle> = OnceLock::new();
static SETTINGS: Mutex<Option<AppSettings>> = Mutex::new(None);

pub fn init(handle: &AppHandle) {
    let _ = APP.set(handle.clone());
    let loaded = load_settings();
    let mut guard = SETTINGS.lock().unwrap_or_else(|p| p.into_inner());
    *guard = Some(loaded);
}

pub fn app() -> Result<AppHandle, ZtError> {
    APP.get()
        .cloned()
        .ok_or_else(|| ZtError::new("application state not initialized"))
}

pub fn app_handle() -> Option<AppHandle> {
    APP.get().cloned()
}

pub fn settings() -> AppSettings {
    SETTINGS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .as_ref()
        .cloned()
        .unwrap_or_default()
}

pub fn update_settings(mut f: impl FnMut(&mut AppSettings)) -> Result<AppSettings, ZtError> {
    let mut guard = SETTINGS.lock().unwrap_or_else(|p| p.into_inner());
    let settings = guard.get_or_insert_with(AppSettings::default);
    f(settings);
    let clone = settings.clone();
    save_settings(&clone)?;
    Ok(clone)
}

pub fn settings_dir() -> Result<std::path::PathBuf, ZtError> {
    let app = app()?;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| ZtError::new(e.to_string()))?;
    std::fs::create_dir_all(&dir).map_err(|e| ZtError::new(e.to_string()))?;
    Ok(dir)
}

const SETTINGS_FILE: &str = "settings.json";

fn load_settings() -> AppSettings {
    let Ok(dir) = settings_dir() else {
        return AppSettings::default();
    };
    let path = dir.join(SETTINGS_FILE);
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &AppSettings) -> Result<(), ZtError> {
    let dir = settings_dir()?;
    let path = dir.join(SETTINGS_FILE);
    let json = serde_json::to_string_pretty(settings).map_err(|e| ZtError::new(e.to_string()))?;
    std::fs::write(path, json).map_err(|e| ZtError::new(e.to_string()))
}
