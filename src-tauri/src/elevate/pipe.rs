//! Frame transport for the elevation pipe: 4-byte LE length prefix + payload.

use crate::core::ZtError;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};

pub fn write_frame(handle: HANDLE, bytes: &[u8]) -> crate::core::Result<()> {
    let len = (bytes.len() as u32).to_le_bytes();
    write_all(handle, &len)?;
    write_all(handle, bytes)?;
    Ok(())
}

pub fn read_frame(handle: HANDLE, max: usize) -> crate::core::Result<Vec<u8>> {
    let mut lenbuf = [0u8; 4];
    read_exact(handle, &mut lenbuf)?;
    let len = u32::from_le_bytes(lenbuf) as usize;
    if len == 0 || len > max {
        return Err(ZtError::new(format!(
            "bad pipe frame length {len} (max {max})"
        )));
    }
    let mut buf = vec![0u8; len];
    read_exact(handle, &mut buf)?;
    Ok(buf)
}

fn write_all(handle: HANDLE, bytes: &[u8]) -> crate::core::Result<()> {
    unsafe {
        let mut off = 0usize;
        while off < bytes.len() {
            let mut written: u32 = 0;
            WriteFile(handle, Some(&bytes[off..]), Some(&mut written), None)?;
            if written == 0 {
                return Err(ZtError::new("pipe write stalled"));
            }
            off += written as usize;
        }
        Ok(())
    }
}

fn read_exact(handle: HANDLE, buf: &mut [u8]) -> crate::core::Result<()> {
    unsafe {
        let mut off = 0usize;
        while off < buf.len() {
            let mut got: u32 = 0;
            ReadFile(handle, Some(&mut buf[off..]), Some(&mut got), None)?;
            if got == 0 {
                return Err(ZtError::new("pipe read stalled"));
            }
            off += got as usize;
        }
        Ok(())
    }
}
