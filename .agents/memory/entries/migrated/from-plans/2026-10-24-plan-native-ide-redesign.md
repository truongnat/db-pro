---
schema_version: 1
id: "MEM-PLAN-native-ide-redesign-52fedbf2a837"
title: "Plan archive: Native Ide Redesign"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, native-ide-redesign]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "52fedbf2a837ae196cd2a99704834c425118787c73869b3167195575d6fbdf3c"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/native-ide-redesign/PLAN.md"
  - "docs/plans/completed/native-ide-redesign/FINDINGS.md"
  - "docs/plans/completed/native-ide-redesign/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/native-ide-redesign/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Native IDE Redesign

## Status

`COMPLETED`

## Source requirement

Root requirement: [`goal-1.md`](../../../../goal-1.md).

## Goal

Evolve the Rust + egui native workspace into a professional database IDE with a stable workbench shell, compact database navigation, keyboard-first query work, staged data editing, contextual Agent actions, and provider-safe advanced tooling.

## Scope

- P0: shell layout, resizable/persistent panels, design tokens and keyboard routing.
- P1: compact hierarchical explorer and object context actions.
- P2: multi-document SQL workspace and result/message/explain output surface.
- P3: staged editable result grid with apply/discard and safe row identity.
- P4: table/object workspace metadata views.
- P5: command palette and Quick Open parity.
- P6: contextual Agent actions and reviewable SQL drafts.
- P7: provider-aware advanced tooling already exposed by the shared runtime.

## Non-goals

- Rewriting the Rust domain/application services that already provide the database contract.
- Replacing egui with a web renderer or adding a second UI stack.
- Silent destructive database operations.
- Claiming runtime completion without a visible native walkthrough for PostgreSQL and SQLite.

## Completion gates

- Every P0–P7 item has source and automated evidence, with unsupported provider paths explicit.
- `cargo fmt --all -- --check`, workspace check/clippy/tests pass.
- PostgreSQL and SQLite are independently exercised where the UI reaches database behavior.
- Native runtime walkthrough covers connection, schema, query, result grid, object workspace, Agent draft and recovery/error states.
- `goal-1.md` acceptance criteria are mapped to evidence in `VERIFICATION.md`.


## Source: `FINDINGS.md`

# Findings

## Initial audit — 2026-09-11

| ID | Severity | Evidence | Decision |
|---|---|---|---|
| NID-001 | P1 | `crates/ui/src/app.rs` has fixed activity/sidebar/workspace orchestration; no persisted panel widths or bottom output surface. | Build the workbench state first because every later surface depends on its layout and focus model. |
| NID-002 | P1 | `crates/ui/src/navigation_view.rs` renders bounded object groups but not a complete connection/database/schema hierarchy or context actions. | Add compact hierarchy/context menus without changing domain services. |
| NID-003 | P1 | `crates/ui/src/query_view.rs` has one result surface and no explain/messages output state. | Add an output-tab state model and route explain through the existing runtime API. |
| NID-004 | P1 | `crates/ui/src/table_editor_view.rs` dispatches update/delete immediately from grid actions. | Introduce local staged changes, then reuse existing typed mutation commands for apply. |
| NID-005 | P2 | `crates/ui/src/palette_view.rs` advertises Ctrl+P while `goal-1.md` requires Cmd/Ctrl+K. | Make K canonical and retain P only as a compatibility shortcut. |
| NID-006 | P1 | `crates/ui/src/app.rs::agent_context` only includes connection/driver/tables/columns. | Extend context with selected object, SQL, result/error summaries; keep destructive actions confirmation-gated. |

The earlier native foundation, provider adapters and runtime facades remain reusable and are intentionally not rewritten.

## Progress — 2026-09-11

| Area | Evidence now present | Remaining verification |
|---|---|---|
| P0 shell | Persistent sidebar/Agent/output dimensions, resizable panels, native modifier helper, bottom output panel | Native Xvfb walkthrough pass |
| P1 explorer | Database → schema nesting, bounded groups, connection/table/view/function context menus | PostgreSQL 977-table introspection and SQLite table interaction pass |
| P2 query | Results/Messages/Explain/History tabs; typed Explain command through runtime; compact More menu | PostgreSQL query/Explain and SQLite query walkthrough pass |
| P3 grid | Update/delete/insert stage locally; dirty rows; sequential Apply and Discard; reload guarded while dirty | Automated state/identity tests pass; provider-specific mutation commands remain capability-gated |
| P4 object workspace | Structure/Data/Indexes/Foreign Keys/Constraints/Dependencies/DDL tabs | PostgreSQL `sql_diff` and SQLite `notes` Structure/Data workspaces pass |
| P5 palette | Cmd/Ctrl+K, dynamic table Quick Open, Explain/Export actions | Native keyboard routing regression and runtime shell pass |
| P6 Agent | Context includes schema/table/columns/SQL/result/plan/error and contextual Ask Agent actions | Offline provider response and safe draft flow pass |

### Follow-up — 2026-09-11

- The explorer no longer hard-codes `public`/`main` as its only schema. `IntrospectResult.schemas` now crosses the runtime/native boundary, the selected schema is stateful, and table/view/trigger/function lists are filtered to it.
- Switching connections clears the previous schema snapshot before the new introspection request, so loading/error states cannot display objects from the wrong connection.
- Agent context and Quick Open now reuse the active-schema table/column scope instead of leaking objects from other schemas.
- PostgreSQL routines are now presented as one compact Functions / Procedures group with routine type visible; the existing provider adapter already returns both kinds.
- The native process exposes an X11 window when launched with `WINIT_UNIX_BACKEND=x11`; the default Wayland launch is not discoverable by the current desktop automation path.
- Future provider-specific surfaces not exposed by this vertical slice remain explicitly out of scope (user/session/lock/sequence/type management). Exposed Explain, backup/restore and routine paths now consult core capabilities.
- Global shortcuts now defer to focused native text inputs; Cmd/Ctrl+K remains global while panel/query shortcuts no longer steal Ctrl+B/F/Enter from form fields.
- Connection mutations now immediately enqueue an Explorer refresh, and pending runtime work requests bounded repainting so async connection/schema state reaches the native UI without relying on a blinking text caret.
- Native walkthrough evidence covers PostgreSQL and SQLite connection/schema/query/result/object flows, Agent offline draft, Explain and SQL error recovery; the plan completion gate is satisfied.


## Source: `VERIFICATION.md`

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

