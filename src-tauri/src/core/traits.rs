//! Marker trait for every payload that crosses a Tauri boundary
//! (command return values and `net::snapshot` events). All such types are
//! serializable, accessible from any command thread, and `'static`.
//!
//! Modules that emit or accept these payloads can bound their generic APIs on
//! this trait so the audit stays honest about what we ship across the IPC seam.

pub trait EventPayload: serde::Serialize + Send + 'static {}

impl<T> EventPayload for T where T: serde::Serialize + Send + 'static {}
