//! Elevation runner: executes an `ElevOp` either in-process (when the app is
//! already elevated) or by spawning the scoped helper over a UAC-created named
//! pipe. The helper performs exactly one request, then exits.

use crate::core::ZtError;
use crate::elevate::is_elevated;
use crate::elevate::ops::{ElevOp, ElevResponse};
#[cfg(not(debug_assertions))]
use tauri::Manager;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, NAMED_PIPE_MODE, PIPE_UNLIMITED_INSTANCES,
};

const PIPE_TIMEOUT_MS: u32 = 120_000;
const FRAME_MAX: usize = 2 << 20;

pub fn run_with_elevation(
    app: &tauri::AppHandle,
    op: &ElevOp,
) -> crate::core::Result<ElevResponse> {
    if is_elevated() {
        return Ok(execute_now(op));
    }

    let pipe_name = format!(r"\\.\pipe\ZeroTrace\elev-{}", uuid::Uuid::new_v4());
    let exe = helper_path(app)?;

    unsafe {
        let server = CreateNamedPipeW(
            &windows::core::HSTRING::from(pipe_name.as_str()),
            PIPE_ACCESS_DUPLEX,
            NAMED_PIPE_MODE(0),
            PIPE_UNLIMITED_INSTANCES,
            65536,
            65536,
            PIPE_TIMEOUT_MS,
            None,
        );
        if server.is_invalid() {
            return Err(ZtError::new("failed to create elevation pipe"));
        }

        let mut child = std::process::Command::new(&exe)
            .arg(&pipe_name)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| {
                ZtError::new(format!(
                    "failed to spawn elevated helper {}: {e}",
                    exe.display()
                ))
            })?;

        // Helpers connects via CreateFileW once spawned; the race is benign.
        let _ = ConnectNamedPipe(server, None);

        let payload = serde_json::to_vec(op)?;
        crate::elevate::pipe::write_frame(server, &payload)?;
        let raw = crate::elevate::pipe::read_frame(server, FRAME_MAX)?;
        let resp: ElevResponse = serde_json::from_slice(&raw)?;

        let _ = child.wait();
        let _ = CloseHandle(server);
        Ok(resp)
    }
}

/// Direct in-process execution (main app already elevated).
pub fn execute_now(op: &ElevOp) -> ElevResponse {
    crate::elevate::ops::execute(op)
}

#[cfg_attr(debug_assertions, allow(unused_variables))]
fn helper_path(app: &tauri::AppHandle) -> crate::core::Result<std::path::PathBuf> {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    #[cfg(not(debug_assertions))]
    {
        if let Ok(dir) = app.path().resource_dir() {
            candidates.push(dir.join("bin").join("zerotrace-elevated.exe"));
        }
    }
    #[cfg(debug_assertions)]
    {
        let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("target");
        candidates.push(base.join("release").join("zerotrace-elevated.exe"));
        candidates.push(base.join("debug").join("zerotrace-elevated.exe"));
    }
    candidates
        .into_iter()
        .find(|p| p.is_file())
        .ok_or_else(|| ZtError::new("elevated helper binary not found"))
}
