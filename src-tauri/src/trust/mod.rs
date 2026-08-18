//! Central data model: every struct that crosses the IPC seam, plus the
//! journal (undo ledger) and build provenance / dependency audit.

pub mod journal;
pub mod models;
pub mod provenance;
pub mod restore;
pub mod services;
pub mod signature;

pub use models::*;
