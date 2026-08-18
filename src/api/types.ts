export type Tier = "recommended" | "limited" | "notrecommended" | "info";

export type Category =
  | "telemetry"
  | "privacy"
  | "cortana"
  | "location"
  | "app_privacy"
  | "ai"
  | "ads"
  | "system";

export interface ConnectionRow {
  pid: number;
  process: string;
  process_path: string | null;
  proto: string;
  state: string;
  local_addr: string;
  local_port: number;
  remote_addr: string | null;
  remote_host: string | null;
  remote_port: number | null;
  zone: "local" | "lan" | "internet" | "unknown";
  bytes_out: number;
  bytes_in: number;
  track: "estats" | "churn";
}

export interface ProcessAgg {
  pid: number;
  process: string;
  process_path: string | null;
  bytes_out: number;
  bytes_in: number;
  conns: number;
}

export interface NetSnapshot {
  ts: number;
  interval_ms: number;
  connections: ConnectionRow[];
  aggregates: ProcessAgg[];
  total_out: number;
  total_in: number;
}

export interface CatalogItem {
  id: string;
  category: Category;
  title: string;
  description: string;
  tier: Tier;
  guarded: boolean;
  kind: "dword" | "string" | "delete" | "task" | "info";
  target_display: string;
  current_display: string;
  desired_display: string;
  applied: boolean;
}

export interface ChangeOutcome {
  ok: boolean;
  detail: string;
  journal_id: string | null;
}

export interface FirewallRule {
  name: string;
  program: string | null;
  remote_addresses: string | null;
  direction: string;
  action: string;
  enabled: boolean;
}

export interface StartupEntry {
  id: string;
  name: string;
  command: string;
  source: string;
  path: string | null;
  signed: boolean | null;
  publisher: string | null;
  risk: "low" | "medium" | "high";
  risk_reasons: string[];
  enabled: boolean;
  source_detail: string;
}

export interface ServiceRow {
  name: string;
  display_name: string;
  state: string;
  start_type: string;
  path: string;
  signed: boolean | null;
  publisher: string | null;
  risk: "low" | "medium" | "high";
  risk_reasons: string[];
}

export interface JournalEntry {
  id: string;
  ts: string;
  kind: "registry" | "task" | "firewall" | "restore_point";
  target: string;
  old_display: string;
  new_display: string;
  result: string;
  restored: boolean;
  item_id: string | null;
}

export interface RestorePointResult {
  ok: boolean;
  detail: string;
}

export interface BuildInfo {
  version: string;
  commit: string;
  tag: string | null;
  repo: string;
  built_at: string;
  profile: string;
}

export interface DepAuditFileScan {
  file: string;
  dep_count: number;
}

export interface DepAudit {
  generated_at: string;
  files: DepAuditFileScan[];
  total_deps: number;
  hits: { file: string; name: string; reason: string }[];
  zero_telemetry: boolean;
}

export interface AppSettings {
  poll_interval_ms: number;
  reverse_dns_enabled: boolean;
}

export interface LocalInterface {
  name: string;
  description: string;
  addresses: string[];
}

export interface ProcessDetail {
  pid: number;
  name: string;
  path: string | null;
  publisher: string | null;
  signed: boolean | null;
}
