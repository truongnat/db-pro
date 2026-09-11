# Verification

This file records commands and observed runtime evidence for the native IDE redesign. It must not claim a gate until the command or walkthrough has actually run.

## Current baseline

- Source: `goal-1.md`, completed native UI foundation plan, current `crates/ui` and `crates/runtime` sources reviewed on 2026-09-11.
- Native UI: P0–P7 source and runtime-slice evidence is present; future provider surfaces remain out of scope.
- Runtime desktop automation: an isolated Xvfb `:99` session with a minimal window manager produced real native egui frames and accepted mouse/keyboard input. The default Wayland launch was not used for this walkthrough, so Wayland-specific behavior remains unclaimed.

## Commands

- `cargo fmt --all -- --check`: PASS (2026-09-11).
- `cargo check --workspace --offline`: PASS (2026-09-11).
- `cargo clippy --workspace --offline --all-targets -- -D warnings`: PASS (2026-09-11).
- `cargo test --workspace --offline`: PASS; core 186, infrastructure 37 + 25 SQLite integration, runtime 4, Tauri 21, UI 49 (2026-09-11).
- `bash .skills/perf-audit/scripts/perf-scan.sh`: Rust check/clippy and frontend performance tests PASS; existing frontend bundle budget remains FAIL (2.35 MB JS, 122 KB CSS), outside native scope.
- Native launch: `env DISPLAY=:99 WAYLAND_DISPLAY= WINIT_UNIX_BACKEND=x11 DB_PRO_DATA_DIR=/tmp/db-pro-runtime-walkthrough-xvfb3 target/debug/db-pro-native` built and ran to completion under the isolated walkthrough; X11 exposed a 1280×800 `DB Pro` window and accepted mouse/keyboard input.
- Follow-up source fix: backup/restore and SQL completion now use the active connection rather than the first saved connection; the grid also accepts a pasted value into the selected editable cell and stages it for review. Targeted UI/runtime/native tests still pass (2026-09-11).
- Follow-up source fix: schema names now cross the runtime/native boundary, the Explorer supports selecting a loaded schema, and object lists are filtered to the selected schema. The UI regression suite is 47 tests passing (2026-09-11), including native TextEdit Ctrl-to-command select-all replacement, palette-shortcut routing, and provider-capability checks.
- Follow-up source fix: switching connections clears the previous schema snapshot before loading the new one, preventing stale cross-connection Explorer content (2026-09-11).
- Follow-up source fix: Agent context and Quick Open now use the same active-schema scope as Explorer (2026-09-11).
- Follow-up source fix: the Explorer labels the existing routine metadata as Functions / Procedures and distinguishes procedure rows visually (2026-09-11).
- Follow-up source fix: native UI now consumes core `DatabaseCapabilities`; Explain, routine navigation and backup/restore expose provider-aware states while result export remains provider-neutral. UI capability and unsupported-provider regression tests pass (2026-09-11).
- Follow-up source fix: focused native text inputs no longer lose Cmd/Ctrl+K/P to the global palette router; connection mutations immediately refresh the Explorer and pending runtime work keeps the native event loop repainting until completion; expression-only PostgreSQL `SELECT` statements are accepted by diagnostics (2026-09-11).
- Native PostgreSQL walkthrough (Xvfb `:99`, local `127.0.0.1:15433`): saved connection refreshed into Explorer, connection became green, schema loaded, query `SELECT 1 AS ok;` returned one row, Explain rendered a JSON plan, `sql_diff` opened Structure and Data workspaces, Agent produced an offline schema draft with writes left unexecuted, and invalid `SELECT (` surfaced parser diagnostics plus a recoverable query-failed status. Runtime logs recorded connection and introspection completion with 977 tables. Captures: `/tmp/db-pro-xvfb-reconnect.png`, `/tmp/db-pro-xvfb-query-valid2.png`, `/tmp/db-pro-xvfb-explain.png`, `/tmp/db-pro-xvfb-table-structure.png`, `/tmp/db-pro-xvfb-agent-response.png`, `/tmp/db-pro-xvfb-query-error-recovery.png`.
- Native SQLite walkthrough (same Xvfb session): a temporary SQLite database with `notes(id, body)` connected, introspected as 1 table/2 columns, query `SELECT id, body FROM notes;` returned the seeded row, and `notes` opened the Structure workspace. Captures: `/tmp/db-pro-xvfb-sqlite-query.png`, `/tmp/db-pro-xvfb-sqlite-table.png`.

## Provider matrix

| Provider | Backend/UI status | Evidence |
|---|---|---|
| PostgreSQL | native UI walkthrough pass for connection, schema, query/result, Explain, object workspace, Agent draft and recovery | Local `127.0.0.1:15433`; runtime logs recorded 977 introspected tables and the UI displayed the bounded selected-schema scope. |
| SQLite | native UI walkthrough pass for connection, schema, query/result and object workspace | Temporary `/tmp/db-pro-native-walkthrough-20260911.sqlite`; workspace connector/integration coverage also passes. |

## Goal acceptance mapping

| Criterion | Current evidence | Status |
|---|---|---|
| 1. Not a web admin/egui prototype | Native shell, compact rail, resizable workbench, tokenized theme, native Xvfb frames and input walkthrough | Verified in isolated native runtime |
| 2. SQL editor and grid dominate | Query workspace, virtualized grid, staged editing, bottom output, PostgreSQL and SQLite result rows | Verified |
| 3. Fewer visible controls | Query secondary controls moved behind More; contextual menus/palette | Source-complete |
| 4. Compact scalable navigation | Bounded search, real runtime schema selection/filtering, large-schema cap, 977-table PostgreSQL introspection | Verified |
| 5. Professional panels/tabs | Persistent resizable panels, query/table/output tabs | Verified |
| 6. Reusable visual system | `DbProTheme` and shared egui primitives | Source-complete |
| 7. Capability growth without toolbar clutter | Command palette, More menu, typed runtime boundary | Source-complete |
| 8. Existing DB functionality intact | Workspace tests, provider integration suites and native PG/SQLite walkthroughs pass | Verified for exposed vertical slice |
| 9. Integrated AI | Agent receives active DB context and produces an offline reviewable SQL draft with safe run/insert paths | Verified; external Codex provider remains configuration-dependent |
| 10. Premium native IDE direction | Architecture, source evidence and native runtime walkthrough support the direction | Verified for the implemented vertical slice |
