# Architecture

Static description of ZeroTrace. No build or Windows run was performed.

## Frontend and core split

```
src/                 React + TypeScript + Vite UI
src-tauri/src/       Rust core
src-tauri/elevated/  standalone elevated helper binary
```

- Frontend entry: `index.html` → `src/main.tsx` → `src/App.tsx`.
- Native entry: `src-tauri/src/main.rs` → `zerotrace::run()`.
- `run()` registers 28 Tauri commands and starts the radar polling loop.
- Frontend stack: Tauri 2, React, TypeScript, Vite, Tailwind/PostCSS and
  `lucide-react`.

## Command domains

| Domain | Commands |
| --- | --- |
| Snapshot and polling | `get_snapshot`, `set_poll_interval`, `get_local_interfaces`, `get_process_info`, `resolve_host` |
| Trust | `signature_status`, `is_elevated` |
| Firewall | `block_process_outbound`, `unblock_process_outbound`, `block_host`, `unblock_host`, `list_zerotrace_rules`, `is_rule_active` |
| Telemetry and undo | `get_telemetry_catalog`, `apply_telemetry_changes`, `undo_change`, `get_undo_center`, `create_restore_point`, `export_journal` |
| Startup and services | `get_startup_entries`, `disable_startup_entry`, `enable_startup_entry`, `get_service_xray` |
| Build and settings | `get_build_info`, `get_dependency_audit`, `get_settings`, `set_settings` |
| Misc | `open_repo` |

There is no HTTP server and no CLI, so Tauri IPC is the only external interface.

## Elevated-helper trust boundary

The UI does not run as administrator. When an operation needs elevation, the core
spawns `zerotrace-elevated.exe` (a separate workspace binary) and sends exactly one
request over a per-run named pipe (`src-tauri/src/elevate/pipe.rs`). The helper
completes that request and exits.

CI builds a fresh helper into `src-tauri/bin/` before release. The committed copy
is a bootstrap because the helper ships bundled.

## Change journal and restore points

- Every registry, scheduled-task and firewall mutation is appended to a JSONL
  journal with the exact undo command (`trust/journal.rs`).
- `export_journal` exposes the journal, and the Undo Center consumes
  `get_undo_center` and `undo_change`.
- A restore point is offered before structural changes
  (`trust/restore.rs`, `create_restore_point`).

## Module map

| Module | Role |
| --- | --- |
| `network/` | Sockets, ESTATS traffic, interfaces, resolver, firewall, process attribution |
| `trust/` | Signature state, journal, restore points, service and registry operations, provenance |
| `startup/` | Run/RunOnce, Startup folder and scheduled-task enable/disable |
| `telemetry/` | Windows diagnostic entry catalogue plus apply/undo |
| `elevate/` | Scoped helper spawn and named-pipe protocol |

## Data

- No database, ORM or SQL schema.
- Settings are read and written through `get_settings` / `set_settings`
  (`AppSettings`).
- Build provenance is embedded: commit, tag, build time and dependency audit JSON
  from `ZT_*` environment values.

## Build and release

Scripts: `dev`, `build`, `preview`, `tauri`, `lint`, `typecheck`, `dep-audit`.
`package-lock.json` and `Cargo.lock` are committed.

Documented developer flow:

```sh
npm install
npm run build
cd src-tauri
cargo build --release
cargo build --release -p zerotrace-elevated
npx tauri dev
npx tauri build --bundles nsis
```

`release.yml` runs on `v*` tags and manual dispatch with a Windows runner and Rust
1.97.1, and sets the dep-audit environment.

## Quality gates

- TypeScript typecheck and ESLint.
- `cargo fmt --check` and clippy with `-D warnings`.
- Dependency deny-list audit (`scripts/dep-audit.mjs`).
- Offline verification (`scripts/verify-offline.ps1`): no telemetry domains in
  `dist/`, `src/`, the npm lockfile registry hosts, or git sources in `Cargo.lock`.

## Known factual notes

- `@tauri-apps/cli` is declared under `dependencies` rather than
  `devDependencies`.
- No test files exist; the gates above are the verification surface.
- The README's "no outbound network path" statement refers to telemetry. The app
  necessarily calls Windows networking APIs to inspect sockets and manage firewall
  rules.

## Unknowns

- Exact ETW providers consumed.
- Whether the committed elevated helper binary matches its source.
- Runtime behaviour on specific Windows builds.
