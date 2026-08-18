//! Background service inventory via the SCM (read-only x-ray).

use crate::trust::models::ServiceRow;
use crate::trust::signature;
use std::mem::size_of;
use windows::core::PCWSTR;
use windows::Win32::System::Services::{
    CloseServiceHandle, EnumServicesStatusExW, OpenSCManagerW, OpenServiceW, QueryServiceConfigW,
    QueryServiceStatusEx, ENUM_SERVICE_STATUS_PROCESSW, QUERY_SERVICE_CONFIGW,
    SC_ENUM_PROCESS_INFO, SC_MANAGER_CONNECT, SC_MANAGER_ENUMERATE_SERVICE, SC_STATUS_PROCESS_INFO,
    SERVICE_AUTO_START, SERVICE_BOOT_START, SERVICE_DEMAND_START, SERVICE_DISABLED,
    SERVICE_KERNEL_DRIVER, SERVICE_QUERY_CONFIG, SERVICE_QUERY_STATUS, SERVICE_START_TYPE,
    SERVICE_STATE_ALL, SERVICE_SYSTEM_START, SERVICE_WIN32,
};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn list() -> Vec<ServiceRow> {
    let manager = match unsafe {
        OpenSCManagerW(
            PCWSTR::null(),
            PCWSTR::null(),
            SC_MANAGER_CONNECT | SC_MANAGER_ENUMERATE_SERVICE,
        )
    } {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };

    let mut out: Vec<ServiceRow> = Vec::new();

    let mut needed: u32 = 0;
    let mut returned: u32 = 0;
    let mut resume: u32 = 0;

    // size probe
    let _ = unsafe {
        EnumServicesStatusExW(
            manager,
            SC_ENUM_PROCESS_INFO,
            SERVICE_WIN32 | SERVICE_KERNEL_DRIVER,
            SERVICE_STATE_ALL,
            None,
            &mut needed,
            &mut returned,
            Some(&mut resume),
            PCWSTR::null(),
        )
    };
    if needed == 0 {
        unsafe {
            let _ = CloseServiceHandle(manager);
        }
        return Vec::new();
    }

    let mut buf: Vec<u8> = vec![0u8; needed as usize];
    let result = unsafe {
        EnumServicesStatusExW(
            manager,
            SC_ENUM_PROCESS_INFO,
            SERVICE_WIN32 | SERVICE_KERNEL_DRIVER,
            SERVICE_STATE_ALL,
            Some(&mut buf),
            &mut needed,
            &mut returned,
            Some(&mut resume),
            PCWSTR::null(),
        )
    };
    if result.is_err() || returned == 0 {
        unsafe {
            let _ = CloseServiceHandle(manager);
        }
        return Vec::new();
    }

    let count = returned as usize;
    let entries: &[ENUM_SERVICE_STATUS_PROCESSW] =
        unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const _, count) };

    for e in entries {
        let name = to_string_opt(PCWSTR(e.lpServiceName.as_ptr())).unwrap_or_default();
        let display = to_string_opt(PCWSTR(e.lpDisplayName.as_ptr())).unwrap_or_default();
        let state: u32 = e.ServiceStatusProcess.dwCurrentState.0;
        out.push(ServiceRow {
            name,
            display_name: display,
            state: state_name(state),
            start_type: "–".to_string(),
            path: String::new(),
            signed: None,
            publisher: None,
            risk: "low".to_string(),
            risk_reasons: Vec::new(),
        });
    }
    unsafe {
        let _ = CloseServiceHandle(manager);
    }

    // second pass: per-service config (start type + binary path) + signing
    if let Ok(manager) =
        unsafe { OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT) }
    {
        for row in &mut out {
            enrich(manager, row);
        }
        unsafe {
            let _ = CloseServiceHandle(manager);
        }
    }

    out
}

fn enrich(manager: windows::Win32::System::Services::SC_HANDLE, row: &mut ServiceRow) {
    let name = wide(&row.name);
    let handle = match unsafe {
        OpenServiceW(
            manager,
            PCWSTR(name.as_ptr()),
            SERVICE_QUERY_CONFIG | SERVICE_QUERY_STATUS,
        )
    } {
        Ok(h) => h,
        Err(_) => return,
    };

    let mut cfg_raw: Vec<u8> = vec![0u8; 8192];
    let mut sz: u32 = 0;
    let ok = unsafe {
        QueryServiceConfigW(
            handle,
            Some(cfg_raw.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW),
            cfg_raw.len() as u32,
            &mut sz,
        )
    }
    .is_ok();
    if ok {
        let cfg = unsafe { &*(cfg_raw.as_ptr() as *const QUERY_SERVICE_CONFIGW) };
        row.start_type = start_type_name(cfg.dwStartType);
        row.path = to_string_opt(PCWSTR(cfg.lpBinaryPathName.as_ptr())).unwrap_or_default();
    }

    let mut status_raw: Vec<u8> =
        vec![0u8; size_of::<windows::Win32::System::Services::SERVICE_STATUS_PROCESS>()];
    let mut sb: u32 = 0;
    let status_ok = unsafe {
        QueryServiceStatusEx(
            handle,
            SC_STATUS_PROCESS_INFO,
            Some(&mut status_raw),
            &mut sb,
        )
    }
    .is_ok();
    if status_ok {
        let status: &windows::Win32::System::Services::SERVICE_STATUS_PROCESS =
            unsafe { &*(status_raw.as_ptr() as *const _) };
        row.state = state_name(status.dwCurrentState.0);
    }

    unsafe {
        let _ = CloseServiceHandle(handle);
    }

    if !row.path.is_empty() {
        let (signed, publisher) = signature::inspect_exe(&row.path);
        row.signed = signed;
        row.publisher = publisher;
        let mut reasons = Vec::new();
        let lp = row.path.to_ascii_lowercase();
        if lp.contains("temp") || lp.contains("\\download") {
            reasons.push("path under temp/download".to_string());
        }
        if signed == Some(false) {
            reasons.push("unsigned binary".to_string());
        }
        if row.name.to_ascii_lowercase().contains("update") && !reasons.is_empty() {
            reasons.insert(0, "auto-update service".to_string());
        }
        row.risk = if (!reasons.is_empty()) || signed == Some(false) {
            "medium".to_string()
        } else {
            "low".to_string()
        };
        row.risk_reasons = reasons;
    }
}

fn to_string_opt(p: PCWSTR) -> Option<String> {
    if p.is_null() {
        return None;
    }
    let mut len = 0usize;
    let mut ptr = p.as_ptr();
    while unsafe { *ptr } != 0 {
        len += 1;
        ptr = unsafe { ptr.add(1) };
    }
    if len == 0 {
        return None;
    }
    Some(unsafe { String::from_utf16_lossy(std::slice::from_raw_parts(p.as_ptr(), len)) })
}

fn state_name(state: u32) -> String {
    match state {
        1 => "stopped",
        2 => "start pending",
        3 => "stop pending",
        4 => "running",
        5 => "continue pending",
        6 => "pause pending",
        7 => "paused",
        _ => "unknown",
    }
    .to_string()
}

fn start_type_name(tp: SERVICE_START_TYPE) -> String {
    match tp {
        SERVICE_BOOT_START => "boot",
        SERVICE_SYSTEM_START => "system",
        SERVICE_AUTO_START => "auto",
        SERVICE_DEMAND_START => "manual",
        SERVICE_DISABLED => "disabled",
        _ => "unknown",
    }
    .to_string()
}

#[allow(dead_code)]
fn _unused() {
    let _ = SERVICE_START_TYPE::default();
}
