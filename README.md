# ZeroTrace

Local, offline network and privacy monitoring for Windows. ZeroTrace watches
connections, traffic, and startup/telemetry/uninstall bloat — all local, with no
accounts, no cloud and no telemetry.

## Screenshot-blind, by design

- **Everything runs on your machine.** The UI talks to a Rust core over Tauri IPC;
  the core reads Windows socket tables, ETW, registry and services. There is no
  server component and no outbound telemetry call in the app code. The firewall and
  radar features do use Windows networking APIs to inspect sockets and manage
  rules — the offline claim is about telemetry, not about avoiding Win32.
- **Low-privilege by default.** Destructive or system-level changes are delegated to
  a small scoped elevated helper (`zerotrace-elevated.exe`) that performs exactly
  one request over a per-run named pipe and exits. A fresh copy is built by CI into
  `src-tauri/bin/` before each release; the committed copy is the bootstrap needed
  because the helper also ships bundled.
- **Auditable.** A JSONL journal records every registry, task and firewall change
  with its exact undo command. A restore point is offered before structural changes
  such as service deletion.
- **Verified builds.** CI runs a dependency deny-list audit (npm and Cargo) and an
  offline verification gate that fails the workflow if a telemetry domain or
  unapproved source appears in the lockfiles or built artifacts.

## Features

| Area | What it shows or controls |
| --- | --- |
| Radar | Live connection table (TCP/UDP plus ESTATS traffic), local interface map, per-process aggregates |
| Process map | Which process owns a socket, signature state, blocked/unblocked |
| Firewall | Toggle block rules for programs or hosts, list/revert ZeroTrace rules |
| Startup | Registry Run/RunOnce, Startup folder, Scheduled Tasks — disable/enable with exact undo |
| Telemetry | Catalogue of Windows diagnostic/service entries with safe apply and an undo center |
| Uninstall bloat | Weighs installed apps by uninstaller footprint (self-report plus heuristic) |

All destructive toggles are reversible, and most require elevation handled by the
helper binary rather than by running the whole UI as administrator.

## Getting started (dev)

Requirements: Node 22+, Rust stable (see `rust-toolchain.toml`), and the
[WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) runtime.

```sh
npm install                                   # frontend deps
npm run build                                 # frontend bundle
cd src-tauri
cargo build --release                         # zerotrace.exe
cargo build --release -p zerotrace-elevated   # helper binary (resources/bin/)
npx tauri dev                                 # run with live reload
npx tauri build --bundles nsis                # offline installer
```

## Command surface

28 Tauri commands are registered by `run()`:

| Domain | Commands |
| --- | --- |
| Snapshot / polling | `get_snapshot`, `set_poll_interval`, `get_local_interfaces`, `get_process_info`, `resolve_host` |
| Trust and signatures | `signature_status`, `is_elevated` |
| Firewall | `block_process_outbound`, `unblock_process_outbound`, `block_host`, `unblock_host`, `list_zerotrace_rules`, `is_rule_active` |
| Telemetry and undo | `get_telemetry_catalog`, `apply_telemetry_changes`, `undo_change`, `get_undo_center`, `create_restore_point`, `export_journal` |
| Startup and services | `get_startup_entries`, `disable_startup_entry`, `enable_startup_entry`, `get_service_xray` |
| Build and settings | `get_build_info`, `get_dependency_audit`, `get_settings`, `set_settings` |
| Misc | `open_repo` |

There is no HTTP API or server and no CLI argument surface; the only external
interface is Tauri IPC.

## Zero-telemetry gates

- `npm run typecheck` / `npm run lint` — TypeScript and ESLint clean.
- `cargo fmt --check` plus
  `cargo clippy --workspace --all-targets -- -D warnings`.
- `npm run dep-audit` — deny-lists npm and Cargo dependencies; the report is
  written to `target/dep-audit.json` and embedded into build info through
  `ZT_DEP_AUDIT_JSON`.
- `scripts/verify-offline.ps1` — asserts no telemetry domains in `dist/`, `src/`,
  the npm lockfile registry hosts, or git sources in `Cargo.lock`.

ZeroTrace is hard-committed to staying offline: the dependency audit fails the
build if a telemetry dependency is added.

## Architecture

```
src/                 React + TypeScript UI (Radar, Processes, Firewall,
                     Startup, Telemetry, Uninstall, Undo Center, Settings)
src-tauri/src/       Rust core
  network/           sockets, ESTATS traffic, interfaces, resolver, firewall
  trust/             signature, journal, restore-point, service/registry ops
  startup/           Run/RunOnce/folder/task enable and disable
  telemetry/         Windows diagnostic entry catalogue + apply/undo
  elevate/           scoped helper spawn + named-pipe protocol
src-tauri/elevated/  standalone helper binary (single request, then exit)
```

The elevated helper runs as a separate workspace binary. The main app sends one
request over a per-run named pipe (`elevate/pipe.rs`) and the helper exits.

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the trust boundary, journal
and module detail.

## Build info

The **About** screen shows commit/tag provenance plus the embedded dependency audit
result (`counts`, `zero_telemetry`, `report_path`), so users can check which
dependency set shipped in a binary. Build environment names: `ZT_COMMIT`,
`ZT_TAG`, `ZT_BUILT_AT`, `ZT_DEP_AUDIT_JSON`, plus `GITHUB_SHA`, `GITHUB_REF`,
`CARGO_PKG_VERSION`, `CARGO_MANIFEST_DIR` and `USERPROFILE`.

## Tests and CI

There are **zero test files** in the repository. Quality enforcement rests on the
typecheck, lint, clippy, dependency-audit and offline-verification gates rather
than on unit tests. `release.yml` runs on `v*` tags and manual dispatch with a
Windows runner and Rust 1.97.1, performing the sign/build/release flow with the
dep-audit environment set.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE). Dependencies are enumerated in the
committed lockfiles and vetted by `scripts/dep-audit.mjs`.
