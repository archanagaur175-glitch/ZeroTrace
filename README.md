# ZeroTrace

Local, offline network & privacy monitoring for Windows. ZeroTrace watches
connections, traffic, and startup/telemetry/uninstall bloat — 100% local,
no accounts, no cloud, no telemetry.

## Screenshot-blind, by design

- **Everything runs on your machine.** The UI talks to a Rust core over Tauri
  IPC; the core reads Windows socket tables, ETW, registry and services.
  There is no server component and no outbound network path in the app code.
- **Low-privilege by default.** Destructive/system-level changes are delegated
  to a tiny, scoped elevated helper (`zerotrace-elevated.exe`) that performs
  exactly one request over a per-run named pipe and exits.
- **Auditable.** A JSONL journal records every registry/task/firewall change
  with its exact undo command. A restore point is offered before structural
  changes such as service deletion.
- **Verified builds.** CI runs a dependency deny-list audit (npm + Cargo) and
  an offline verification gate that fails the workflow if any telemetry domain
  or unapproved source appears in the lockfiles or the built artifacts.

## Features

| Area | What it shows / controls |
| --- | --- |
| Radar | Live connection table (TCP/UDP + ESTATS traffic), local interface map, per-process aggregates |
| Process map | Which process owns a socket, signature state, blocked/unblocked |
| Firewall | Toggle block rules for programs or hosts, list/revert ZeroTrace rules |
| Startup | Registry Run/RunOnce, Startup folder, Scheduled Tasks — disable/enable with exact undo |
| Telemetry | Catalog of Windows diagnostic/service entries with safe apply + undo center |
| Uninstall bloat | Weighs installed apps by uninstaller footprint (self-report + heuristic) |

All destructive toggles are reversible and most require elevation, which is
handled by the helper binary — not by running the whole UI as admin.

## Getting started (dev)

Requirements: Node 22+, Rust stable (see `rust-toolchain.toml`), and the
[WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) runtime.

```sh
npm install                          # frontend deps
npm run build                        # frontend bundle
cd src-tauri
cargo build --release                # zerotrace.exe
cargo build --release -p zerotrace-elevated   # helper binary (resources/bin/)
npx tauri dev                        # run with live reload
npx tauri build --bundles nsis       # offline installer
```

## Zero-telemetry gates

- `npm run typecheck` / `npm run lint` — TS + ESLint clean (mount-fetch
  effects are intentional and allowed).
- `cargo fmt --check` + `cargo clippy --workspace --all-targets -- -D warnings`.
- `npm run dep-audit` — deny-lists npm + cargo dependencies; report written to
  `target/dep-audit.json` (also embedded into build info via
  `ZT_DEP_AUDIT_JSON`).
- `scripts/verify-offline.ps1` — asserts no telemetry domains in `dist/`,
  `src/`, the npm lockfile registry hosts, or git sources in `Cargo.lock`.

`ZeroTrace` is **hard-committed** to staying offline: the dependency audit
fails the build if a telemetry dependency is ever added.

## Architecture

```
src/                 React + TypeScript UI (Radar, Processes, Firewall,
                     Startup, Telemetry, Uninstall, Undo Center, Settings)
src-tauri/src/       Rust core
  network/           sockets, ESTATS traffic, interfaces, resolver, firewall
  trust/             signature, journal, restore-point, service/registry ops
  startup/           Run/RunOnce/folder/task enable & disable
  telemetry/         Windows diagnostic entry catalog + apply/undo
  elevate/           scoped helper spawn + named-pipe protocol
src-tauri/elevated/  standalone helper binary (single request, then exit)
```

## Build info

`About` screen shows commit/tag provenance plus the embedded dependency audit
result (`counts`, `zero_telemetry`, `report_path`), so users can verify which
dependency set shipped in a binary.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE). Deps (npm + Cargo) are enumerated in
the committed lockfiles and vetted by `scripts/dep-audit.mjs`.