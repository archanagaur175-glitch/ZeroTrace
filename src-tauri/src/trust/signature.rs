//! Authenticode verification via WinVerifyTrust + publisher via the
//! file version resource. Read-only; never requires elevation.

use crate::trust::models::SignatureStatus;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::Security::WinTrust::{
    WinVerifyTrust, WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_DATA_0,
    WINTRUST_FILE_INFO, WTD_CHOICE_FILE, WTD_REVOKE_NONE, WTD_STATEACTION_CLOSE,
    WTD_STATEACTION_VERIFY, WTD_UI_NONE,
};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};

pub fn signature_status(path: &str) -> SignatureStatus {
    let (wide, ptr) = widestr(path);
    let trusted = verify_authenticode(ptr);
    let publisher = if trusted { company_name(&wide) } else { None };
    SignatureStatus {
        signed: Some(trusted),
        publisher,
    }
}

/// Convenience: `(signed, publisher)` tuple used by startup/services x-rays.
/// Returns `(None, None)` when the path isn't a real file.
pub fn inspect(path: &str) -> (Option<bool>, Option<String>) {
    let st = signature_status(path);
    (st.signed, st.publisher)
}

/// Same as `inspect` but tolerates `"{one.exe} args"` / plain command strings
/// by extracting the leading executable token.
pub fn inspect_exe(cmd: &str) -> (Option<bool>, Option<String>) {
    let trimmed = cmd.trim();
    let token = if let Some(rest) = trimmed.strip_prefix('"') {
        rest.split('"')
            .next()
            .map(|t| t.to_string())
            .unwrap_or_default()
    } else {
        trimmed.split_whitespace().next().unwrap_or("").to_string()
    };
    if token.is_empty() {
        return (None, None);
    }
    let path = std::path::Path::new(&token);
    if !path.is_file() {
        return (None, None);
    }
    inspect(&token)
}

pub fn status_for_file(path: &std::path::Path) -> SignatureStatus {
    if !path.is_file() {
        return SignatureStatus {
            signed: None,
            publisher: None,
        };
    }
    signature_status(&path.to_string_lossy())
}

/// NUL-terminated wide buffer + a PCWSTR into it (buffer outlives use).
fn widestr(s: &str) -> (Vec<u16>, PCWSTR) {
    let mut v: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
    let ptr = PCWSTR(v.as_mut_ptr());
    (v, ptr)
}

fn verify_authenticode(wpath: PCWSTR) -> bool {
    unsafe {
        let mut file_info = WINTRUST_FILE_INFO {
            cbStruct: std::mem::size_of::<WINTRUST_FILE_INFO>() as u32,
            pcwszFilePath: wpath,
            hFile: HANDLE(std::ptr::null_mut()),
            pgKnownSubject: std::ptr::null_mut(),
        };

        let mut state = WINTRUST_DATA {
            cbStruct: std::mem::size_of::<WINTRUST_DATA>() as u32,
            pPolicyCallbackData: std::ptr::null_mut(),
            pSIPClientData: std::ptr::null_mut(),
            dwUIChoice: WTD_UI_NONE,
            fdwRevocationChecks: WTD_REVOKE_NONE,
            dwUnionChoice: WTD_CHOICE_FILE,
            Anonymous: WINTRUST_DATA_0 {
                pFile: std::ptr::addr_of_mut!(file_info),
            },
            dwStateAction: WTD_STATEACTION_VERIFY,
            ..Default::default()
        };

        let mut action = WINTRUST_ACTION_GENERIC_VERIFY_V2;
        let trusted = {
            let rc = WinVerifyTrust(
                HWND(std::ptr::null_mut()),
                std::ptr::addr_of_mut!(action),
                std::ptr::addr_of_mut!(state) as *mut _,
            );
            rc == 0
        };
        state.dwStateAction = WTD_STATEACTION_CLOSE;
        let _ = WinVerifyTrust(
            HWND(std::ptr::null_mut()),
            std::ptr::addr_of_mut!(action),
            std::ptr::addr_of_mut!(state) as *mut _,
        );
        trusted
    }
}

fn company_name(wpath: &[u16]) -> Option<String> {
    let size = unsafe { GetFileVersionInfoSizeW(PCWSTR(wpath.as_ptr()), None) };
    if size == 0 {
        return None;
    }
    let mut buf = vec![0u8; size as usize];
    let ok = unsafe {
        GetFileVersionInfoW(
            PCWSTR(wpath.as_ptr()),
            None,
            size,
            buf.as_mut_ptr() as *mut _,
        )
        .is_ok()
    };
    if !ok {
        return None;
    }
    let mut ptr: *mut core::ffi::c_void = std::ptr::null_mut();
    let mut len: u32 = 0;
    let key = windows::core::w!("\\StringFileInfo\\*\\CompanyName");
    let found = unsafe { VerQueryValueW(buf.as_ptr() as *const _, key, &mut ptr, &mut len) };
    if !found.as_bool() || ptr.is_null() || len == 0 {
        return None;
    }
    let slice = unsafe { core::slice::from_raw_parts(ptr as *const u16, len as usize / 2) };
    let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
    let name = String::from_utf16_lossy(&slice[..end]).trim().to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}
