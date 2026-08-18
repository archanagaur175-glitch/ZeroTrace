//! ZeroTrace elevated helper.
//!
//! Launched by the main process (via UAC) with one argument: the name of a
//! local named pipe. It reads a single `ElevOp` frame, executes it, writes the
//! `ElevResponse` frame, then exits. One request per process, by design.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use windows::Win32::Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_MODE, OPEN_EXISTING,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        std::process::exit(101);
    }

    let pipe_name = &args[1];
    let handle = unsafe {
        match CreateFileW(
            windows::core::PCWSTR(
                pipe_name
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect::<Vec<u16>>()
                    .as_ptr(),
            ),
            GENERIC_READ.0 | GENERIC_WRITE.0,
            FILE_SHARE_MODE(0),
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        ) {
            Ok(h) => h,
            Err(_) => std::process::exit(102),
        }
    };

    // Exactly one framed request.
    let raw = match zerotrace::elevate::pipe::read_frame(handle, 2 << 20) {
        Ok(r) => r,
        Err(_) => {
            let _ = unsafe { CloseHandle(handle) };
            std::process::exit(103);
        }
    };
    let op: zerotrace::elevate::ops::ElevOp = match serde_json::from_slice(&raw) {
        Ok(op) => op,
        Err(_) => {
            let _ = unsafe { CloseHandle(handle) };
            std::process::exit(104);
        }
    };

    let resp = zerotrace::elevate::ops::execute(&op);
    let _ = serde_json::to_vec(&resp)
        .ok()
        .and_then(|bytes| zerotrace::elevate::pipe::write_frame(handle, &bytes).ok());

    let _ = unsafe { CloseHandle(handle) };
    std::process::exit(0);
}
