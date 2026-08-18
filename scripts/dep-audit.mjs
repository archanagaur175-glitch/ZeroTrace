#!/usr/bin/env node
// Dependency deny-list audit for ZeroTrace.
//
// Scans the committed lockfiles (npm + cargo) for any package/crate that is a
// known telemetry, analytics, or crash-reporting dependency. ZeroTrace is a
// 100% local privacy tool and must never link or ship such a component — not
// even in a way that is "currently unused".
//
// Emits a JSON report matching the Rust `DepAudit` model (see
// src-tauri/src/trust/models.rs) to the path given by $ZT_DEP_AUDIT_JSON (or
// target/dep-audit.json), and exits non-zero when a banned dependency is
// found so CI can gate on it.

import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

// npm / node package names. Prefix entries end with `*` and match any package
// whose name starts with that prefix; other entries match exactly.
const NPM_DENY = [
  "@sentry/*",
  "@bugsnag/*",
  "@amplitude/*",
  "@mixpanel/*",
  "@segment/*",
  "@segmentio/*",
  "@fullstory/*",
  "@posthog/*",
  "@datadog/browser-rum*",
  "@datadog/browser-logs*",
  "@microsoft/applicationinsights-*",
  "@azure/monitor-*",
  "@google-analytics/*",
  "@rudderstack/*",
  "@telemetrydeck/*",
  "@countly/*",
  "@customerio/*",
  "@honeycombio/*",
  "@grafana/faro-*",
  "@sentry-internal/*",
  "sentry",
  "sentry-browser",
  "sentry-node",
  "mixpanel",
  "mixpanel-browser",
  "amplitude-js",
  "segment-analytics",
  "analytics-node",
  "posthog-js",
  "fullstory",
  "logrocket",
  "heap-js",
  "newrelic",
  "newrelic-browser",
  "bugsnag",
  "applicationinsights",
  "rudder-sdk-node",
  "countly-sdk-web",
  "telemetrydeck",
  "splunk-otel-js",
  "honeycombio",
  "gtag",
  "react-ga",
  "react-ga4",
  "react-analytics",
];

// Cargo crates. Same convention (prefix entries end with `*`).
const CRATES_DENY = [
  "opentelemetry*",
  "sentry",
  "sentry-core",
  "sentry-types",
  "bugsnag",
  "mixpanel*",
  "amplitude-rs",
  "posthog-rs",
  "segment",
  "rudder-analytics",
  "applicationinsights",
  "countly",
  "newrelic",
  "tattle*",
  "azure_monitor*",
  "google_analytics*",
  "honeycomb*",
];

const matches = (name, rules) =>
  rules.some((r) =>
    r.endsWith("*")
      ? name.startsWith(r.slice(0, -1))
      : name === r,
  );

function readNpmLock() {
  const path = join(root, "package-lock.json");
  if (!existsSync(path)) return { files: [], deps: [], file: "package-lock.json" };
  const lock = JSON.parse(readFileSync(path, "utf8"));
  const packages = lock.packages ?? {};
  const names = [];
  for (const key of Object.keys(packages)) {
    if (key === "") continue;
    // key is "node_modules/<name>" or "node_modules/<scope>/<name>".
    const seg = key.split("node_modules/").pop();
    if (seg) names.push(seg);
  }
  return { files: [], deps: names, file: "package-lock.json" };
}

function readCargoLock() {
  const path = join(root, "src-tauri", "Cargo.lock");
  if (!existsSync(path)) return { files: [], deps: [], file: "Cargo.lock" };
  const raw = readFileSync(path, "utf8");
  const names = [];
  for (const m of raw.matchAll(/\[\[package\]\][\s\S]*?name = "([^"]+)"/g)) {
    names.push(m[1]);
  }
  return { files: [], deps: names, file: "src-tauri/Cargo.lock" };
}

function normalize(name) {
  return name.toLowerCase().replace(/[_-]+/g, "-");
}

function audit() {
  const sources = [readNpmLock(), readCargoLock()];
  const hits = [];
  let total = 0;

  for (const src of sources) {
    for (const name of src.deps) {
      const n = normalize(name);
      if (matches(n, NPM_DENY) || matches(n, CRATES_DENY)) {
        hits.push({ file: src.file, name, reason: "banned telemetry/analytics dependency" });
      }
    }
    total += src.deps.length;
    src.count = src.deps.length;
  }

  return {
    generated_at: new Date().toISOString(),
    files: sources.map((s) => ({ file: s.file, dep_count: s.count ?? 0 })),
    total_deps: total,
    hits,
    zero_telemetry: hits.length === 0,
  };
}

const report = audit();
const outPath = process.env.ZT_DEP_AUDIT_JSON
  ? resolve(process.env.ZT_DEP_AUDIT_JSON)
  : join(root, "target", "dep-audit.json");

mkdirSync(dirname(outPath), { recursive: true });
writeFileSync(outPath, JSON.stringify(report, null, 2));

console.log(`[dep-audit] ${report.files.map((f) => `${f.file}:${f.dep_count}`).join("  ")}`);
console.log(`[dep-audit] total deps: ${report.total_deps}`);
if (report.hits.length === 0) {
  console.log("[dep-audit] zero telemetry: PASS");
} else {
  for (const h of report.hits) {
    console.error(`[dep-audit] DENIED ${h.file}: ${h.name} — ${h.reason}`);
  }
  console.error(`[dep-audit] zero telemetry: FAIL (${report.hits.length} banned)`);
  process.exit(1);
}
