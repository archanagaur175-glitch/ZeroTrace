//! Serde models matching `src/api/types.ts`. Field names are the wire
//! contract — do not rename without updating the frontend.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Tier / Category enums (frontend: `Tier`, `Category`)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Recommended,
    Limited,
    NotRecommended,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Telemetry,
    Privacy,
    Cortana,
    Location,
    AppPrivacy,
    Ai,
    Ads,
    System,
}

// ---------------------------------------------------------------------------
// Radar
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionRow {
    pub pid: u32,
    pub process: String,
    pub process_path: Option<String>,
    pub proto: String,
    pub state: String,
    pub local_addr: String,
    pub local_port: u32,
    pub remote_addr: Option<String>,
    pub remote_host: Option<String>,
    pub remote_port: Option<u32>,
    pub zone: String,
    pub bytes_out: u64,
    pub bytes_in: u64,
    pub track: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessAgg {
    pub pid: u32,
    pub process: String,
    pub process_path: Option<String>,
    pub bytes_out: u64,
    pub bytes_in: u64,
    pub conns: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetSnapshot {
    pub ts: f64,
    pub interval_ms: u32,
    pub connections: Vec<ConnectionRow>,
    pub aggregates: Vec<ProcessAgg>,
    pub total_out: u64,
    pub total_in: u64,
}

// ---------------------------------------------------------------------------
// Telemetry catalog / outcomes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct CatalogItem {
    pub id: String,
    pub category: Category,
    pub title: String,
    pub description: String,
    pub tier: Tier,
    pub guarded: bool,
    pub kind: String, // "dword" | "string" | "delete" | "task" | "info"
    pub target_display: String,
    pub current_display: String,
    pub desired_display: String,
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangeOutcome {
    pub ok: bool,
    pub detail: String,
    pub journal_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Firewall
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct FirewallRule {
    pub name: String,
    pub program: Option<String>,
    pub remote_addresses: Option<String>,
    pub direction: String,
    pub action: String,
    pub enabled: bool,
}

// ---------------------------------------------------------------------------
// Startup & services
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct StartupEntry {
    pub id: String,
    pub name: String,
    pub command: String,
    pub source: String,
    pub path: Option<String>,
    pub signed: Option<bool>,
    pub publisher: Option<String>,
    pub risk: String,
    pub risk_reasons: Vec<String>,
    pub enabled: bool,
    pub source_detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceRow {
    pub name: String,
    pub display_name: String,
    pub state: String,
    pub start_type: String,
    pub path: String,
    pub signed: Option<bool>,
    pub publisher: Option<String>,
    pub risk: String,
    pub risk_reasons: Vec<String>,
}

// ---------------------------------------------------------------------------
// Journal
// ---------------------------------------------------------------------------

/// Raw previous state of a registry value, used to reverse a change exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegOldState {
    pub existed: bool,
    pub value_type: u32,
    pub data: Vec<u8>,
}

/// One immutable journal line. Internal fields are serialized to the JSONL
/// ledger so undo can reverse exactly, but hidden from the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: String,
    pub ts: String,
    pub kind: String, // "registry" | "task" | "firewall" | "restore_point"
    pub target: String,
    pub old_display: String,
    pub new_display: String,
    pub result: String,
    pub restored: bool,
    pub item_id: Option<String>,

    // Revert payload (never sent to the frontend).
    #[serde(skip_serializing, default)]
    pub reg_hive: Option<String>,
    #[serde(skip_serializing, default)]
    pub reg_key: Option<String>,
    #[serde(skip_serializing, default)]
    pub reg_value: Option<String>,
    #[serde(skip_serializing, default)]
    pub reg_old: Option<RegOldState>,
    #[serde(skip_serializing, default)]
    pub reg_apply_type: Option<u32>,
    #[serde(skip_serializing, default)]
    pub reg_apply_data: Option<Vec<u8>>,
    #[serde(skip_serializing, default)]
    pub task_path: Option<String>,
    #[serde(skip_serializing, default)]
    pub rule_name: Option<String>,
    #[serde(skip_serializing, default)]
    pub fw_program: Option<String>,
    #[serde(skip_serializing, default)]
    pub fw_host: Option<String>,
    #[serde(skip_serializing, default)]
    pub startup_mode: Option<String>,
    #[serde(skip_serializing, default)]
    pub file_from: Option<String>,
    #[serde(skip_serializing, default)]
    pub file_to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RestorePointResult {
    pub ok: bool,
    pub detail: String,
}

// ---------------------------------------------------------------------------
// Trust & provenance
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct BuildInfo {
    pub version: String,
    pub commit: String,
    pub tag: Option<String>,
    pub repo: String,
    pub built_at: String,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepAuditFileScan {
    pub file: String,
    pub dep_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepAudit {
    pub generated_at: String,
    pub files: Vec<DepAuditFileScan>,
    pub total_deps: usize,
    pub hits: Vec<DepHit>,
    pub zero_telemetry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepHit {
    pub file: String,
    pub name: String,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Settings & misc
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub poll_interval_ms: u32,
    pub reverse_dns_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            poll_interval_ms: 1000,
            reverse_dns_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalInterface {
    pub name: String,
    pub description: String,
    pub addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessDetail {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub publisher: Option<String>,
    pub signed: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignatureStatus {
    pub signed: Option<bool>,
    pub publisher: Option<String>,
}
