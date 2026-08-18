import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  BuildInfo,
  CatalogItem,
  ChangeOutcome,
  DepAudit,
  FirewallRule,
  JournalEntry,
  LocalInterface,
  NetSnapshot,
  ProcessDetail,
  RestorePointResult,
  ServiceRow,
  StartupEntry,
} from "./types";

export function getSnapshot(): Promise<NetSnapshot> {
  return invoke<NetSnapshot>("get_snapshot");
}

export function setPollInterval(ms: number): Promise<void> {
  return invoke("set_poll_interval", { ms });
}

export function resolveHost(ip: string): Promise<string | null> {
  return invoke<string | null>("resolve_host", { ip });
}

export function getLocalInterfaces(): Promise<LocalInterface[]> {
  return invoke<LocalInterface[]>("get_local_interfaces");
}

export function getProcessInfo(pid: number): Promise<ProcessDetail> {
  return invoke<ProcessDetail>("get_process_info", { pid });
}

export function signatureStatus(path: string): Promise<{
  signed: boolean | null;
  publisher: string | null;
}> {
  return invoke("signature_status", { path });
}

export function blockProcessOutbound(
  path: string,
  displayName: string,
): Promise<ChangeOutcome> {
  return invoke<ChangeOutcome>("block_process_outbound", {
    path,
    displayName,
  });
}

export function unblockProcessOutbound(ruleName: string): Promise<ChangeOutcome> {
  return invoke<ChangeOutcome>("unblock_process_outbound", { ruleName });
}

export function blockHost(host: string): Promise<ChangeOutcome> {
  return invoke<ChangeOutcome>("block_host", { host });
}

export function unblockHost(host: string): Promise<ChangeOutcome> {
  return invoke<ChangeOutcome>("unblock_host", { host });
}

export function listZeroTraceRules(): Promise<FirewallRule[]> {
  return invoke<FirewallRule[]>("list_zerotrace_rules");
}

export function isRuleActive(name: string): Promise<boolean> {
  return invoke<boolean>("is_rule_active", { name });
}

export function getTelemetryCatalog(): Promise<CatalogItem[]> {
  return invoke<CatalogItem[]>("get_telemetry_catalog");
}

export function applyTelemetryChanges(
  ids: string[],
): Promise<{ ok: boolean; detail: string }> {
  return invoke("apply_telemetry_changes", { ids });
}

export function undoChange(journalId: string): Promise<ChangeOutcome> {
  return invoke<ChangeOutcome>("undo_change", { journalId });
}

export function getUndoCenter(): Promise<JournalEntry[]> {
  return invoke<JournalEntry[]>("get_undo_center");
}

export function createRestorePoint(): Promise<RestorePointResult> {
  return invoke<RestorePointResult>("create_restore_point");
}

export function getStartupEntries(): Promise<StartupEntry[]> {
  return invoke<StartupEntry[]>("get_startup_entries");
}

export function disableStartupEntry(id: string): Promise<ChangeOutcome> {
  return invoke<ChangeOutcome>("disable_startup_entry", { id });
}

export function enableStartupEntry(id: string): Promise<ChangeOutcome> {
  return invoke<ChangeOutcome>("enable_startup_entry", { id });
}

export function getServiceXRay(): Promise<ServiceRow[]> {
  return invoke<ServiceRow[]>("get_service_xray");
}

export function getBuildInfo(): Promise<BuildInfo> {
  return invoke<BuildInfo>("get_build_info");
}

export function getDependencyAudit(): Promise<DepAudit> {
  return invoke<DepAudit>("get_dependency_audit");
}

export function isElevated(): Promise<boolean> {
  return invoke<boolean>("is_elevated");
}

export function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export function setSettings(settings: AppSettings): Promise<void> {
  return invoke("set_settings", { settings });
}

export function exportJournal(): Promise<string> {
  return invoke<string>("export_journal");
}

export function openRepo(): Promise<void> {
  return invoke("open_repo");
}
