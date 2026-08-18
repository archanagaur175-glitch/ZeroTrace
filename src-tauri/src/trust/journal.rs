//! JSONL journal at `%LOCALAPPDATA%\ZeroTrace\journal.jsonl`.
//!
//! Every privileged change is appended as one line; `undo` reads the line,
//! builds the reverse operation, and re-writes the ledger with the entry
//! flagged `restored` (keyed by id) so the frontend can reflect it.

use crate::core::ZtError;
use crate::trust::models::JournalEntry;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub fn journal_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into()))
        });
    base.join("ZeroTrace").join("journal.jsonl")
}

fn rfc3339_now() -> String {
    crate::trust::provenance::rfc3339_now()
}

/// RFC3339 UTC timestamp string (public helper for callers building entries).
pub fn now() -> String {
    rfc3339_now()
}

pub fn append(entry: &mut JournalEntry) -> crate::core::Result<String> {
    let path = journal_path();
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if entry.id.is_empty() {
        entry.id = crate::elevate::ops::new_id();
    }
    if entry.ts.is_empty() {
        entry.ts = rfc3339_now();
    }
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| ZtError::new(format!("cannot open journal {}: {e}", path.display())))?;
    let line = serde_json::to_string(entry).map_err(ZtError::from)?;
    writeln!(&mut f, "{line}").map_err(ZtError::from)?;
    f.flush().map_err(ZtError::from)?;
    Ok(entry.id.clone())
}

pub fn read_all() -> crate::core::Result<Vec<JournalEntry>> {
    let path = journal_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let f = File::open(&path).map_err(ZtError::from)?;
    let reader = BufReader::new(f);
    let mut out = Vec::new();
    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<JournalEntry>(&line) {
            out.push(entry);
        }
    }
    Ok(out)
}

/// Mark an entry as restored. Returns the entry that was rewritten.
pub fn mark_restored(id: &str) -> crate::core::Result<JournalEntry> {
    let path = journal_path();
    let mut entries = read_all()?;
    let mut hit: Option<JournalEntry> = None;
    for e in entries.iter_mut() {
        if e.id == id {
            e.restored = true;
            hit = Some(e.clone());
            break;
        }
    }
    let Some(entry) = hit else {
        return Err(ZtError::new(format!("journal id not found: {id}")));
    };
    let mut out = String::new();
    for e in &entries {
        let line = serde_json::to_string(e).map_err(ZtError::from)?;
        out.push_str(&line);
        out.push('\n');
    }
    fs::write(&path, out).map_err(ZtError::from)?;
    Ok(entry)
}

/// Export a copy of the ledger to the user's Downloads folder.
pub fn export(csidl_downloads: i32) -> crate::core::Result<String> {
    let downloads = crate::trust::provenance::known_folder(csidl_downloads)?;
    let ts = rfc3339_now().replace([':', '-'], "").replace('Z', "");
    let dest = downloads.join(format!("zerotrace-journal-{ts}.jsonl"));
    fs::copy(journal_path(), &dest).map_err(ZtError::from)?;
    Ok(dest.to_string_lossy().into_owned())
}
