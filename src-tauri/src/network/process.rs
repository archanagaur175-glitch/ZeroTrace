//! Process identity (name + path) from a PID, unprivileged.

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};

pub struct ProcessInfo {
    pub name: String,
    pub path: Option<String>,
}

pub fn resolve_pid(pid: u32) -> Option<ProcessInfo> {
    if pid == 0 {
        return Some(ProcessInfo {
            name: "System Idle Process".into(),
            path: None,
        });
    }
    if pid == 4 {
        return Some(ProcessInfo {
            name: "System".into(),
            path: Some("System".into()),
        });
    }
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = vec![0u16; 2048];
        let mut size: u32 = 2048;
        let path = match QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        ) {
            Ok(()) if size > 0 => {
                let s = String::from_utf16_lossy(&buf[..size as usize]);
                Some(s)
            }
            _ => None,
        };
        let _ = CloseHandle(handle);
        let name = path
            .as_ref()
            .and_then(|p| {
                std::path::Path::new(p)
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| format!("[{pid}]"));
        Some(ProcessInfo { name, path })
    }
}
