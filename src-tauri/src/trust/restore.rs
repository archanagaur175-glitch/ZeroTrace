//! Restore point creation via the Windows System Restore API.

use crate::core::ZtError;
use crate::trust::models::RestorePointResult;
use windows::Win32::System::Restore::{
    SRSetRestorePointW, BEGIN_SYSTEM_CHANGE, END_SYSTEM_CHANGE, MODIFY_SETTINGS, RESTOREPOINTINFOW,
    RESTOREPOINTINFO_EVENT_TYPE, RESTOREPOINTINFO_TYPE, STATEMGRSTATUS,
};

const BEGIN: RESTOREPOINTINFO_EVENT_TYPE = BEGIN_SYSTEM_CHANGE;
const TYPE: RESTOREPOINTINFO_TYPE = MODIFY_SETTINGS;

fn wide(s: &str, cap: usize) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_utf16().take(cap.saturating_sub(1)).collect();
    v.resize(cap, 0);
    v
}

fn arr_from(src: &[u16]) -> [u16; 256] {
    let mut arr = [0u16; 256];
    let n = src.len().min(256);
    arr[..n].copy_from_slice(&src[..n]);
    arr
}

/// Create a restore point. Returns Ok(false) when System Restore is disabled
/// or not available (e.g. unsupported disk layout), which callers may treat
/// as non-fatal.
pub fn create(description: &str) -> Result<RestorePointResult, ZtError> {
    let desc = wide(description, 256);
    let info = RESTOREPOINTINFOW {
        dwEventType: BEGIN,
        dwRestorePtType: TYPE,
        llSequenceNumber: 0,
        szDescription: arr_from(&desc),
    };

    let mut status = STATEMGRSTATUS {
        nStatus: Default::default(),
        llSequenceNumber: 0,
    };

    let begin_ok = unsafe { SRSetRestorePointW(&info, &mut status) }.as_bool();

    if !begin_ok {
        let nstatus = status.nStatus.0;
        return Ok(RestorePointResult {
            ok: false,
            detail: format!("system restore unavailable on this volume (status {nstatus})"),
        });
    }

    let end = RESTOREPOINTINFOW {
        dwEventType: END_SYSTEM_CHANGE,
        dwRestorePtType: TYPE,
        llSequenceNumber: status.llSequenceNumber,
        szDescription: arr_from(&desc),
    };
    let mut end_status = STATEMGRSTATUS {
        nStatus: Default::default(),
        llSequenceNumber: 0,
    };
    let _ = unsafe { SRSetRestorePointW(&end, &mut end_status) };

    Ok(RestorePointResult {
        ok: true,
        detail: "restore point created".to_string(),
    })
}
