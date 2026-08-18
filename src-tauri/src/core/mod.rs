//! Shared foundation for all backend modules: error type, result alias,
//! monotonic time budgeting, and the event-payload marker trait.

pub mod traits;

use serde::Serialize;
use std::fmt;
use std::time::{Duration, Instant};

/// Error type carried across Tauri command boundaries. Serde-serializable so
/// frontend `invoke` failures render as `{ message }`.
#[derive(Debug, Serialize)]
pub struct ZtError {
    pub message: String,
}

impl ZtError {
    pub fn new(message: impl Into<String>) -> Self {
        ZtError {
            message: message.into(),
        }
    }
}

impl fmt::Display for ZtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ZtError {}

impl From<windows::core::Error> for ZtError {
    fn from(e: windows::core::Error) -> Self {
        ZtError::new(e.to_string())
    }
}

impl From<std::io::Error> for ZtError {
    fn from(e: std::io::Error) -> Self {
        ZtError::new(e.to_string())
    }
}

impl From<serde_json::Error> for ZtError {
    fn from(e: serde_json::Error) -> Self {
        ZtError::new(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ZtError>;

/// A monotonic time budget, used to cap how long a single poll may spend on
/// best-effort work (e.g. reverse-DNS lookups) before falling back.
pub struct Timeblock {
    start: Instant,
    budget: Duration,
}

impl Timeblock {
    pub fn new(budget_ms: u64) -> Self {
        Timeblock {
            start: Instant::now(),
            budget: Duration::from_millis(budget_ms),
        }
    }

    pub fn remaining(&self) -> Duration {
        self.budget.saturating_sub(self.start.elapsed())
    }

    pub fn exhausted(&self) -> bool {
        self.start.elapsed() >= self.budget
    }
}
