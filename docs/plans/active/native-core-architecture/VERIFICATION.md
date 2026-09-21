# Native Core Architecture — Verification

Source checkpoint: `532de109`.

## Current change

The native UI interaction boundary is being migrated in vertical slices.
Explorer surface composition, connection nodes, database/schema trees,
schema-object folders, table-folder/table-detail rendering, the Agent surface,
large Settings sections, table surfaces and query execution preparation,
Explain transitions, saved-query preparation and Schema Workbench mutation
planning now collect typed intents/effects in feature-owned contexts. Agent
confirmation planning now owns document targeting,
patch application and continuation payload preparation outside the app root.
Direct multiline channel bypasses are guarded.
Runtime-bound request transitions now use a prepare → dispatch → commit shape:
failed dispatches do not leave fake loading, pending, connecting, deleting or
query-running state behind. This applies to connection/schema/table/query and
mutation paths, including SQL prediction requests.
Saved Tasks now follows the same surface boundary: `saved_tasks_surface_view.rs`
owns egui rendering and emits typed actions, while `tasks_view.rs` keeps only
draft persistence, scheduler policy, runtime dispatch and the root action
adapter. Per-payload dispatch is split into focused SQL, backup, export and
maintenance handlers.
Settings Data Grid, Connections, AI, Security, Advanced and Appearance panes
now render through explicit settings contexts; `settings_view.rs` keeps
navigation, persistence/runtime adapters and the remaining backup/diagnostics
orchestration.
Backup and Restore settings now render through `SettingsBackupContext` and
return typed actions; request-id allocation and runtime command dispatch remain
at the root adapter.
The Welcome surface now renders through `WelcomeSurfaceContext`; it owns the
responsive start page, connection rows and presentation intent collection,
while `welcome_view.rs` only applies the typed root actions.
Query destructive/export/save-as/dirty-close dialogs now render through
explicit dialog contexts and return typed actions. The shell Output Panel now
owns its panel layout and tab rendering through `ShellOutputPanelContext`;
`DbProApp` retains only result selection, persistence and runtime adapters.
The plan remains `IMPLEMENTING` because other large feature surfaces
still implement rendering directly on the root and the full runtime evidence
matrix is not complete.

The Query output dock now has an explicit `QueryOutputDockContext` for resize
geometry and tab chrome. The root keeps only the pane callback because result,
chart, message, explain and history panes dispatch query-specific effects.
This also removes a duplicate output-tab render call that caused the tab strip
to be painted twice.

The shell topbar now has an explicit `ShellTopbarContext` that renders the
connection/navigation/search chrome and emits typed intents. Palette, gallery,
agent, theme and document-navigation effects remain in the root adapter.

The shell statusbar now has an explicit `ShellStatusbarContext` for connection,
runtime, editor and output-panel chrome. The root only prepares the read model
and applies the output-panel toggle.

The Agent thread now renders through an immutable
`AgentThreadSurfaceContext` and emits typed submit, result, retry and
confirmation actions. Agent panel shell geometry and context-chip presentation
now use `AgentPanelSurfaceContext` and `AgentContextSurfaceContext`; Agent
settings/header/composer runtime effects remain root adapters.

The Security drop-role confirmation now renders through
`SecurityConfirmationContext` and returns typed confirm/cancel actions. The
root keeps only the PostgreSQL command dispatch and state transition.

Monitoring presentation now renders through `MonitoringSurfaceContext`, which
owns the header, error/empty states, health snapshot, sessions and workload
presentation. It emits typed refresh, session and workload actions; polling,
snapshot dispatch and auxiliary monitoring surfaces remain at the root effect
adapter.

Result-grid viewport composition now renders through
`ResultGridBodyContext`. The surface owns viewport sizing, horizontal/vertical
scrolling and virtualized row iteration behind a renderer contract; header
actions, row selection/editing and mutation effects remain in the root adapter.

Security composition now renders through `SecuritySurfaceContext`, which owns
the PostgreSQL gating notice, error state and the roles/details/confirmation/RLS
surface order. It emits one typed action stream while request IDs, command
dispatch and provider mutations remain in `security_activity_view.rs`.

Query panel geometry now renders through `query_layout_surface_view::calculate`.
The pure layout context owns dock/editor height policy, including minimized and
maximized states, while `query_view.rs` only composes the returned layout with
the editor and output surfaces.

Audit activity presentation now renders through `AuditSurfaceContext`, which
owns the filters, page/error/empty presentation and event cards. It emits typed
refresh, export, selection, bookmark and open-query actions; audit command
dispatch and cross-feature navigation remain at the root adapter.

Event Trigger presentation now renders through `EventTriggerSurfaceContext`,
which owns inventory cards, create form, DDL preview and drop confirmation.
It emits typed refresh, preview, alter, create and drop actions; command
builders and provider dispatch remain at the root adapter.

FDW presentation now renders through `FdwSurfaceContext`, which owns inventory
cards, redacted options, create form, DDL preview and drop confirmation. It
emits typed refresh, preview, create and drop actions; FDW command builders and
provider dispatch remain at the root adapter.

Logical Replication presentation now renders through
`ReplicationSurfaceContext`, which owns inventory cards, redacted subscription
details, publication creation, DDL preview and drop confirmations. It emits
typed refresh, preview, create and drop actions; replication command builders
and provider dispatch remain at the root adapter.

PostgreSQL settings presentation now renders through
`PgSettingsSurfaceContext`, which owns filtering, setting cards, session-edit
dialog and preview dialog. It emits typed refresh, edit, reset, apply and
preview actions; setting validation, command builders and provider dispatch
remain at the root adapter.

Workspace-files shell presentation now renders through `FilesSurfaceContext`,
which owns the workspace header, empty/recent state, root selector, trust and
environment controls, and panel-tab selector. It emits typed folder, root,
trust, environment, close and tab actions; filesystem operations, feedback,
folder-picker orchestration and tab feature effects remain at the root adapter.

Query actions menu presentation now renders through
`QueryActionsSurfaceContext`, which owns the anchored menu, run/save/explain
entries, editor controls, prediction disclosure/modes, snippets and folder
input. It emits typed actions; query dispatch, prediction scheduling,
filesystem/runtime effects and cross-feature Agent navigation remain at the
root adapter.

Visual Query Builder presentation now renders through
`VisualQueryBuilderContext`, which owns the SELECT form, table/view picker,
joins, columns, predicates, ordering, limits and generated SQL preview. It
receives explicit builder state/schema/dialect inputs and emits only
apply/import/clear intents; editor document changes and feedback remain at the
root adapter.

Workspace Migrations and Graph tabs now render through the explicit secondary
tab helpers with `OpenFile` intents. The root adapter remains responsible for
opening the selected SQL document; the tab renderers no longer implement
`DbProApp` methods.

Table structure presentation now renders through `TableStructureContext`,
which owns metrics, column filtering, the columns table, cell-level display
and the centered column-detail dialog. It emits only typed column-selection and
close actions; `table_structure_view.rs` remains a small root adapter that
owns the table snapshot, search state and selected-column navigation.

Query snippets now render through `QuerySnippetsContext` and emit an insertion
intent; document mutation and panel state remain at the query root adapter.
The unused legacy inline completion and diagnostics renderers were removed,
along with their dead completion state field.

Table Profile now renders through `table_profile_surface_view.rs`, which owns
bounded page profiling, empty states and the profile grid. The table root only
routes the result snapshot. Table-structure loading/error placeholder rendering
also lives in `TableStructureContext`'s surface module rather than in the
workspace router.

Backup Settings now renders its complete database-files card through
`SettingsBackupContext`, including provider capability messaging and backup
tool hints. The root supplies the driver/capability read model and applies the
typed backup/restore command actions.

## Gate evidence at `532de109`

- Focused `cargo fmt --all`, `cargo check -p db-pro-ui` and
  `cargo clippy -p db-pro-ui --all-targets -- -D warnings`: passed.
- `cargo test -p db-pro-ui --quiet`: passed; 677 UI tests passed.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed.
- Runtime launch: the rebuilt release binary is running in terminal session
  `81089` for manual verification. The full workspace gate remains recorded at
  the immediately preceding checkpoint `fa952484`; provider/runtime state and
  the full affected-surface matrix remain unproven.

## Gate evidence at `fa952484`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed.
- Runtime launch: the rebuilt release binary is running in terminal session
  `32807` for manual verification. The previously inspected New Connection
  captures at 1280x800, 1440x900 and 1920x1080 remain valid for the unchanged
  modal surface; provider/runtime state and the full affected-surface matrix
  remain unproven.

## Gate evidence at `1b5a6859`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed.
- Runtime captures inspected at logical `1280x800`, `1440x900` and `1920x1080`:
  `/tmp/db-pro-native-core-1b5a6859.png`,
  `/tmp/db-pro-native-core-1b5a6859-1440x900.png` and
  `/tmp/db-pro-native-core-1b5a6859-1920x1080.png`. New Connection remains
  centered with a separated header/divider and right-aligned close control.
- Runtime launch: the rebuilt release binary is running in terminal session
  `82559` for manual verification. Provider/runtime state and the full
  affected-surface matrix remain unproven.

## Gate evidence at `bfa6d0df`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed; source worktree was clean before this docs
  checkpoint was recorded.
- Runtime capture: `/tmp/db-pro-native-core-bfa6d0df.png`, logical `1280x800`.
  The inspected New Connection surface remains centered with a separated
  header/divider and right-aligned close control.
- Runtime launch: the rebuilt release binary is running in terminal session
  `54226` for manual verification. Workspace secondary-tab provider/runtime
  state and the full affected-surface matrix remain unproven.

## Gate evidence at `e91385f8`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed; source worktree was clean before this docs
  checkpoint was recorded.
- Runtime capture: `/tmp/db-pro-native-core-e91385f8.png`, logical `1280x800`.
  The inspected New Connection surface remains centered with a separated
  header/divider and right-aligned close control.
- Runtime launch: the rebuilt release binary is running in terminal session
  `20599` for manual verification. Visual-builder provider/runtime state and
  the full affected-surface matrix remain unproven.

## Gate evidence at `facc2ae0`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- Query-specific `cargo test -p db-pro-ui query_view_tests --no-fail-fast
  --quiet`: passed; 8 tests passed.
- `git diff --check`: passed; source worktree was clean before this docs
  checkpoint was recorded.
- Runtime capture: `/tmp/db-pro-native-core-facc2ae0.png`, logical `1280x800`.
  The inspected New Connection surface remains centered with a separated
  header/divider and right-aligned close control.
- Runtime launch: the rebuilt release binary is running in terminal session
  `20419` for manual verification. Query provider/runtime state and the full
  affected-surface matrix remain unproven.

## Gate evidence at `a9134f4b`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed; source worktree was clean before this docs
  checkpoint was recorded.
- Runtime capture: `/tmp/db-pro-native-core-a9134f4b.png`, logical `1280x800`.
  The inspected New Connection surface remains centered with a separated
  header/divider and right-aligned close control.
- Runtime launch: the rebuilt release binary is running in terminal session
  `53325` for manual verification. Workspace-files provider/runtime state and
  the full affected-surface matrix remain unproven.

## Gate evidence at `c25886b3`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed; source worktree was clean before this docs
  checkpoint was recorded.
- Runtime capture: `/tmp/db-pro-native-core-c25886b3.png`, logical `1280x800`.
  The inspected New Connection surface remains centered with a separated
  header/divider and right-aligned close control.
- Runtime launch: the rebuilt release binary is running in terminal session
  `26614` for manual verification. pg_settings provider-state and the full
  runtime matrix remain unproven.

## Gate evidence at `e984bd73`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed for the source checkpoint; documentation changes
  are recorded after the code commit.
- Runtime capture: `/tmp/db-pro-native-core-e984bd73.png`, logical `1280x800`.
  The inspected New Connection surface remains centered with a separated
  header/divider and right-aligned close control.
- Runtime launch: the rebuilt release binary is running in terminal session
  `59914` for manual verification. Replication provider-state and the full
  runtime matrix remain unproven.

## Gate evidence at `8d5c1c7c`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- Runtime capture: `/tmp/db-pro-native-core-8d5c1c7c.png`, logical `1280x800`.
  The inspected New Connection surface remained centered with a separated
  header/divider and right-aligned close control.
- Runtime launch: the rebuilt release binary ran in terminal session `78453`.
  FDW provider-state and the full runtime matrix remained unproven.

## Gate evidence at `4c0ba2a4`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed; worktree clean and `main` is aligned with
  `origin/main`.
- Runtime capture: `/tmp/db-pro-native-core-4c0ba2a4.png`, logical `1280x800`.
  The inspected New Connection surface remains centered with a separated
  header/divider and right-aligned close control.
- Runtime launch: the rebuilt release binary is running in terminal session
  `50413` for manual verification. Event Trigger provider-state and the full
  runtime matrix remain unproven.

## Gate evidence at `ba0d3570`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed; worktree clean and `main` is aligned with
  `origin/main`.
- Runtime capture: `/tmp/db-pro-native-core-ba0d3570.png`, logical `1280x800`.
  The inspected New Connection surface remains centered with a separated
  header/divider and right-aligned close control.
- Runtime launch: the rebuilt release binary is running in terminal session
  `23025` for manual verification. Audit provider-state and the full runtime
  matrix remain unproven.

## Gate evidence at `e2694c0f`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed; worktree clean and `main` is aligned with
  `origin/main`.
- Runtime capture: `/tmp/db-pro-native-core-e2694c0f.png`, logical `1280x800`.
  The inspected New Connection surface remains centered with a separated
  header/divider and right-aligned close control.
- Runtime launch: the rebuilt release binary is running in terminal session
  `5934` for manual verification. Query provider-state and the full runtime
  matrix remain unproven.

## Gate evidence at `41994db6`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 675 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed; `main` was clean and aligned with `origin/main`
  at the source checkpoint before this documentation commit.
- Runtime capture: `/tmp/db-pro-native-core-41994db6.png`, logical `1280x800`.
  The inspected New Connection surface remains centered with a separated
  header/divider and right-aligned close control.
- Runtime launch: the rebuilt release binary is running in terminal session
  `13484` for manual verification. No dedicated Security provider-state
  capture was collected; the runtime matrix remains incomplete.

## Gate evidence at `cc459b8d`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 675 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed; `main` was clean and aligned with `origin/main`
  at the source checkpoint before this documentation commit.
- Runtime capture: `/tmp/db-pro-native-core-cc459b8d.png`, logical `1280x800`.
  The inspected New Connection surface remains centered with a separated
  header/divider and right-aligned close control.
- Runtime launch: the rebuilt release binary is running in terminal session
  `38636` for manual verification. No dedicated result-grid provider-state
  capture was collected; the runtime matrix remains incomplete.

## Gate evidence at `46a9920b`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 675 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed, 0 warnings and 0 failures.
- `git diff --check`: passed; `main` is aligned with `origin/main` at the
  source checkpoint before this documentation commit.
- Runtime launch: the rebuilt release binary from this checkpoint is running
  in terminal session `97544`. No dedicated monitoring provider-state capture
  was collected; the runtime matrix remains incomplete.

## Gate evidence at `bac3bbe7`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 675 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed on the clean post-commit tree; 16 checks passed, 0 warnings and 0
  failures.
- `git diff --check`: passed; worktree clean and `main` is aligned with
  `origin/main`.
- Runtime launch: the latest release binary is running in terminal session
  `50398`. No dedicated Security interaction capture was collected; the
  runtime matrix remains incomplete.

## Gate evidence at `3cd32bf2`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 675 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed on the exact pre-commit source tree; 16 checks passed, 0 warnings and
  0 failures.
- `git diff --check`: passed; worktree clean and `main` is aligned with
  `origin/main`.
- Runtime launch: the latest release binary is running in terminal session
  `64169`. No dedicated Agent interaction capture was collected; the runtime
  matrix remains incomplete.

## Gate evidence at `e0d8105e`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 675 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed on the clean post-commit tree; 16 checks passed, 0 warnings and 0
  failures.
- `git diff --check`: passed; worktree clean and `main` is aligned with
  `origin/main`.
- Runtime launch: the latest release binary is running in terminal session
  `78351` for manual verification. No dedicated statusbar interaction capture
  was collected; the runtime matrix remains incomplete.
- Runtime viewport evidence was collected and visually inspected for the
  native Welcome/shell surface:
  - `/tmp/db-pro-native-core-e0d8105e-1280x800.png` — exact logical
    `1280x800` (framebuffer `2560x1600`).
  - `/tmp/db-pro-native-core-e0d8105e-1440x900.png` — requested width honored;
    host-constrained logical height was `838` (framebuffer `2880x1676`).
  - `/tmp/db-pro-native-core-e0d8105e-1920x1080.png` — requested width honored;
    host-constrained logical height was `838` (framebuffer `3840x1676`).
  The latter two are useful responsive checks but do not close the exact
  `1440x900` / `1920x1080` acceptance requirement.

## Gate evidence at `d0952762`

- `cargo fmt --all -- --check`: passed.
- `cargo check -p db-pro-ui`: passed.
- `cargo test -p db-pro-ui --no-fail-fast --quiet`: passed; 675 UI tests.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed on the exact pre-commit tree; 14 checks passed, 2 inherited function
  size warnings and 0 failures.
- `git diff --check`: passed before commit.
- Release rebuild after the commit: both normal and `capture` native builds
  passed. The rebuilt binary is running in terminal session `99120` for manual
  verification. No dedicated topbar interaction capture was collected.

## Gate evidence at `ebaed39c`

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 675 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed on the clean post-commit tree; 16 checks passed, 0 warnings and 0
  failures.
- `git diff --check`: passed; worktree clean and `main` is aligned with
  `origin/main`.
- Runtime launch: the release binary built from this checkpoint is running in
  terminal session `31444` for manual verification. A dedicated interaction
  capture for the Query output dock is still pending; launch evidence alone
  does not satisfy the full runtime surface matrix.

## Gate evidence at `b0b3d095`

- `cargo check --workspace`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 672 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed on the
  exact pre-commit source tree.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed on the exact pre-commit source tree; 14 checks passed, 2 inherited
  `shell_chrome_view.rs` function-size warnings and 1 pre-existing `app.rs`
  size warning were retained by the ratchet, and 0 checks failed.
- `git diff --check`: passed before commit.
- Runtime launch: the latest release binary built from this checkpoint is
  running in terminal session `19628` for manual verification. No dedicated
  Output Panel/dialog interaction capture was collected in this checkpoint.

## Gate evidence at `a919e870`

- `cargo check --workspace`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 669 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed on the
  exact pre-commit source tree.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed on the exact pre-commit source tree; 15 checks passed, 1 pre-existing
  `app.rs` size warning was retained by the ratchet, and 0 checks failed.
- `git diff --check`: passed before commit.
- Runtime launch: the latest release binary built from this checkpoint is
  running in terminal session `90661` for manual verification. No dedicated
  Welcome interaction capture was collected in this checkpoint.

## Gate evidence at `5cd134db`

- `cargo check --workspace`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 668 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed on the
  exact pre-commit source tree.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed on the exact pre-commit source tree; 15 checks passed, 1 pre-existing
  `app.rs` size warning was retained by the ratchet, and 0 checks failed.
- `git diff --check`: passed before commit.
- Runtime launch: the latest release binary built from this checkpoint is
  running in terminal session `97958` for manual verification. No dedicated
  Settings interaction capture was collected in this checkpoint.

## Gate evidence at `7cffc59c`

- `cargo check --workspace`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 668 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed on each exact pre-commit source tree for the Settings slices; 15
  checks passed, 1 pre-existing `app.rs` size warning was retained by the
  ratchet, and 0 checks failed.
- `git diff --check`: passed before each commit.
- Runtime launch: the latest release binary built from this checkpoint is
  running in terminal session `38792` for manual verification. No dedicated
  Settings interaction capture was collected in this checkpoint.

## Gate evidence at `56973275`

- `cargo check --workspace`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri and 667 UI tests passed, with only
  environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed on the exact pre-commit source tree; 15 checks passed, 1 pre-existing
  `app.rs` size warning was retained by the ratchet, and 0 checks failed.
- `git diff --check`: passed before commit.
- Runtime launch: the release binary built from this checkpoint is running in
  terminal session `43954` for manual verification. No deterministic Saved
  Tasks interaction capture was collected in this checkpoint.

## Gate evidence at `f1df8f36`

- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri, 664 UI and all other workspace suites
  passed, with only environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed with 0 warnings and 0 failures (the working tree
  was clean after commit, so the scanner reported 0 changed production files).
- `git diff --check`: passed.
- Runtime capture: `/tmp/db-pro-native-core-f1df8f36-welcome.png`, logical
  `1280x800` (PNG framebuffer `2560x1600` on the 2x host), was visually
  inspected. It shows the native Welcome/Explorer empty state with the New
  connection entry point. The rebuilt release binary is running in terminal
  session `50716` for manual verification. Explorer table/schema-object runtime
  capture remains pending because deterministic capture has no live schema
  provider.

## Gate evidence at `7a401686`

- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri, 664 UI and all other workspace suites
  passed, with only environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 14 checks passed, with the existing 9 function-size and `app.rs`
  size warnings retained by ratchet and 0 failures.
- `git diff --check`: passed before commit.
- Runtime capture: `/tmp/db-pro-native-core-7a401686-welcome.png`, logical
  `1280x800` (PNG framebuffer `2560x1600` on the 2x host), was visually
  inspected. It shows the native Welcome/Explorer empty state with the New
  connection entry point. The rebuilt release binary is running in terminal
  session `68483` for manual verification. Explorer table/schema-object runtime
  capture remains pending because deterministic capture has no live schema
  provider.

## Gate evidence at `bad96b53`

- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri, 660 UI and all other workspace suites
  passed, with only environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed with 0 warnings for this committed diff.
- Runtime capture: `/tmp/db-pro-native-core-bad96b53-welcome.png`, logical
  `1280x800`, was visually inspected. It shows the normal native Welcome
  surface with Explorer empty state and the New connection entry point. The
  latest release binary from this checkpoint is running in terminal session
  `37627` for manual verification. Explorer table/schema-object runtime capture
  remains pending because deterministic capture has no live schema provider.

## Gate evidence at `257b7bde`

- The full workspace gate was run on the source tree committed as `257b7bde`
  immediately before commit: formatting, workspace check, clippy with
  `-D warnings`, workspace tests, both native release builds, architecture
  boundary, clean-code scan and `git diff --check` all passed.
- Workspace tests included 404 core, 119 infrastructure, 32 runtime, 4 Tauri,
  660 UI and all other workspace suites, with only environment-gated tests
  ignored. Clean-code scan reported 15 passes, 1 pre-existing `app.rs` size
  warning and 0 failures.
- Runtime capture `/tmp/db-pro-native-core-257b7bde-welcome.png`, logical
  `1280x800`, was visually inspected. The latest release binary is running in
  terminal session `47176` for manual verification. Explorer table/schema-object
  runtime capture remains pending because deterministic capture has no live
  schema provider.

## Gate evidence at `b0debd01`

- The full workspace gate was run on the source tree committed as `b0debd01`
  immediately before commit: formatting, workspace check, clippy with
  `-D warnings`, workspace tests, both native release builds, architecture
  boundary, clean-code scan and `git diff --check` all passed.
- Workspace tests included 404 core, 119 infrastructure, 32 runtime, 4 Tauri,
  660 UI and all other workspace suites, with only environment-gated tests
  ignored. Clean-code scan reported 15 passes, 1 pre-existing `app.rs` size
  warning and 0 failures.
- Runtime capture `/tmp/db-pro-native-core-b0debd01-welcome.png`, logical
  `1280x800`, was visually inspected. The latest release binary is running in
  terminal session `38176` for manual verification. Explorer table/schema-object
  runtime capture remains pending because deterministic capture has no live
  schema provider.

## Gate evidence at `6bbd6b62`

- `cargo test --workspace --no-fail-fast --quiet`: passed; 404 core, 119
  infrastructure, 32 runtime, 4 Tauri, 654 UI and all other workspace suites
  passed, with only environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `cargo fmt --all -- --check`: passed on the checkpoint.
- `cargo check --workspace`: passed on the checkpoint.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed on the checkpoint.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed; 16 checks passed with 0 warnings for this committed diff.
- Runtime capture: `/tmp/db-pro-native-core-ddefb967.png`, 1280×800, showed
  the centered New Connection dialog with separated header, divider and
  right-aligned close icon. The latest release binary was then left running
  for manual verification as PID `10200`. That evidence predates this
  checkpoint; no new screenshot was collected for `6bbd6b62`.

- `6bbd6b62`: Agent confirmation preparation now returns a typed
  `PreparedAgentConfirmation` from `agent_confirmation.rs`; `DbProApp` keeps
  only pending-session lookup, UX error feedback and runtime dispatch. Focused
  Agent tests and the full workspace gate passed. The freshly rebuilt release
  binary was running for manual verification in terminal session `94688`.

- `cac0b77e`: Agent query-result opening now uses an explicit
  `AgentResultWorkspaceContext`; query-document result replacement, output-tab
  selection, grid invalidation and feedback are no longer implemented in the
  root Agent adapter. The complete workspace gate passed again, and the latest
  release binary is running for manual verification in terminal session
  `93643`.

- `d7dda78f`: Schema activation now uses `SchemaActivationContext`; staged
  change blocking, schema/table/object selection reset, table workspace reset,
  navigation-cache invalidation and Welcome activation are owned by the schema
  feature context. The full workspace gate passed again with 655 UI tests, and
  the latest release binary is running for manual verification in terminal
  session `84390`.

- `1d1d92ab`: Explorer connection connect/disconnect transitions now use an
  explicit `ExplorerConnectionContext`. Transaction/staged-change guards,
  connection-scoped schema/table/Agent resets and lifecycle request state are
  feature-owned; `DbProApp` retains only request allocation and command
  dispatch. The complete workspace gate passed with 657 UI tests, and the
  latest release binary is running for manual verification in terminal session
  `18507`.

- `b6949238`: Table selection now uses `TableSelectionContext`; staged-change
  blocking, layout persistence/restore, selected table/object reset, table-view
  activation and recent-table state are feature-owned. The root retains only
  query text generation and metadata/data request dispatch. The complete
  workspace gate passed with 659 UI tests, and the latest release binary is
  running for manual verification in terminal session `47888`.

- `00a23cec`: Schema-object activation now uses `SchemaObjectActivationContext`
  with a typed `SchemaObjectActivation` request; table reset, DDL surface
  selection, routine preview cleanup/sync and feedback stay outside the root
  adapter. The complete workspace gate passed with 660 UI tests, and the
  latest release binary is running for manual verification in terminal session
  `12401`.

- `5f6e7870`: Table detail folders (Columns, Foreign keys and Indexes) now
  render through the presentation-only `TableDetailsView`; these folders no
  longer implement methods on `DbProApp`. The complete workspace gate passed
  with 660 UI tests, and the latest release binary is running for manual
  verification in terminal session `88237`.

- Runtime capture from the `5f6e7870` release binary at logical `1280x800` was
  visually inspected:
  - `/tmp/db-pro-native-core-5f6e7870-new-connection.png`: centered modal,
    separated header, right-aligned close and visible sticky footer.
  - `/tmp/db-pro-native-core-5f6e7870-connection-error.png`: error alert is
    visible, the form body remains scrollable and the footer remains visible.
  The PNG framebuffer is `2560x1600` because the host uses a 2x display scale.
  Explorer table/schema-object runtime capture is still pending.

- `fc3cebf2`: Views, Functions and Triggers folders now render through the
  explicit `SchemaObjectFoldersView` and emit typed open/query/copy actions;
  the root only applies those effects. The complete workspace gate passed with
  660 UI tests, and the latest release binary is running for manual
  verification in terminal session `25513`.

- `bad96b53`: schema-scoped tables, table details and Views/Functions/Triggers
  folder rendering now run through `ExplorerSchemaObjectsView` with an explicit
  `ExplorerSchemaObjectsModel`; table selection, table-row actions and schema
  object actions are returned as typed intents. The complete workspace gate
  passed with 660 UI tests, and the rebuilt release binary is running in
  terminal session `37627`.

- `4c5b9864`: connected database/schema tree rendering now runs through
  `ExplorerSchemaTreeView`; the root assembles its read model and applies only
  refresh, schema-activation and schema-object intents. This is a docs-only
  follow-up to the fully gated code commit `bad96b53`; the rebuilt release
  binary is running in terminal session `47176`.

- `257b7bde`: the connection row, connecting/failed/disconnected hints and
  connected schema-tree composition now run through
  `ExplorerConnectionNodeView`; the root applies only typed connection and
  schema-tree actions. The full workspace gate passed on this exact source
  tree, and the release binary is running in terminal session `47176`.

- `b0debd01`: the Explorer toolbar, empty state, scroll container and
  connection-node composition now run through `ExplorerSurfaceContext`; the
  root only assembles the read model and applies typed surface actions. The
  full workspace gate passed on this exact source tree, and the release binary
  is running in terminal session `38176`.

- `f5fc419d`: table/query surface contexts were extracted; query execution
  preparation now owns destructive gating, parameter binding, query history and
  document-running transitions, while `DbProApp` remains the command-send
  boundary. Workspace tests report 647 UI tests passed.

- `ef33bb1c`: Explain capability validation, ANALYZE confirmation and
  document/output transitions moved into `QueryExplainContext`; the root now
  only resolves the connection/capability inputs, allocates the request ID and
  dispatches the prepared command. Workspace tests report 649 UI tests passed.

- `2b711995`: saved-query payload preparation and request tracking moved into
  `QuerySaveContext`; workspace-backed filesystem saves remain at the filesystem
  boundary. Workspace tests report 651 UI tests passed.

- `cae4a9c2`: closing a Table workspace now delegates to the canonical table
  reset transition, preventing filters, sorts, row caches and mutation dialogs
  from leaking into the next table session.

- `1d8647dc`: Schema Workbench mutation-request planning moved into
  `SchemaWorkbenchState`; the root now only resolves driver/orchestration. The
  workspace gate reports 654 UI tests passed. Clean scan retains one existing
  large `app.rs` warning and one ownership-conversion clone heuristic in the
  new planner.

- `d687de8e`: Agent API-key and Saved Task backup dispatches now use the central
  `dispatch_command` adapter. The architecture guard was strengthened to catch
  multiline direct `TaskBridge::send` calls. Workspace tests report 654 UI tests
  passed.

- `bdf2e363`: Explorer schema changes, disconnects and schema-object activation
  now use the canonical table workspace reset, clearing stale metadata, query
  filters/sorts, caches and mutation dialogs together. Workspace tests report
  654 UI tests passed.

- `40e875fe`: Agent workflow reducer, SQL patch safety and Agent-result
  projection moved out of the root state module.
- `90b72fae`: Agent run preparation became an explicit state transition and
  unused legacy conversation state was removed.
- `8720c788`: Agent API-key settings now emit typed intents; command dispatch
  remains in the composition-root adapter.
- `40666229`: Agent header mode/clear/close/settings interactions now emit
  typed intents; session reset is owned by `AgentState`.
- `68fcb73a`: Agent confirmation target selection, patch application and
  pending-to-running continuation are feature-owned.
- `15321ca5`: Agent context quick actions now emit typed submit intents.
- `bcc9aa2e`: clippy-driven `AgentRunPreparation` DTO and settings condition
  cleanup; full workspace gates were rerun on this source state.
- `2a31d9d1`: Keybindings settings rendering/edit/reset now lives in a
  state-owned context and emits a reset intent.
- `350d0e8b`: Diagnostics settings rendering emits copy/export intents while
  serialization and filesystem I/O remain in the root adapter.
- `3f8a5fe1`: General settings and named-workspace-session controls emit
  typed save/restore/duplicate/delete intents.
- `ddefb967`: Editor settings rendering now consumes Preferences and Query
  feature state directly; full workspace gates and release/runtime evidence
  were rerun on this source state.

- `761db9ed`: connection-row painting and context menu now return a typed
  `ConnectionRowAction`; lifecycle/workspace/clipboard/dialog effects remain
  in the root reducer adapter.
- `0d290db7`: table rows now return typed `TableRowAction` intents, while SQL
  preview generation and table workspace transitions stay in the reducer.
- `e9405d82`: schema-node expansion and schema activation intent are isolated
  from staged-change/workspace reset logic.
- `a02b8f4e`: connected database-node expansion is isolated in its own view
  context.
- `242b918a`: View/Function/Trigger rows share a typed schema-object row
  context for open/query/copy intents; schema-object activation remains in the
  root adapter.

- Connection dialog state aggregate added under `crates/ui/src/connection/state.rs`.
- Connection dialog view, form, advanced panels, events and workspace actions
  now address the aggregate instead of individual `DbProApp` fields.
- Connection lifecycle state now owns active/pending/error/request state.
- `ConnectionCatalogState` now owns the saved-connection read model and its
  replacement/lookup operations.
- `WorkspaceShellState` now owns shell navigation, panel visibility/geometry,
  welcome lifecycle and pending navigation state; panel resize values are
  clamped through state setters.
- `QuerySessionState` now owns query documents, active selection, selected text,
  save/close request tracking and Save As lifecycle.
- Query-document lifecycle transitions now run through an explicit
  `QueryDocumentContext`; opening, duplication, closing and fallback-tab
  behavior receive their feature aggregates directly, while query execution
  remains a composition-root command decision.
- Query-document switching now uses the same context, so active cursor and
  selected-text synchronization is kept beside the document transition.
- Active query text edits, document connection/schema binding and prediction
  cancellation now use the same context; the root retains only cross-aggregate
  result-grid invalidation and command-level orchestration.
- Query connection/schema/capability resolution now uses a read-only
  `QueryConnectionContext` over the query session, connection catalog/lifecycle
  and schema explorer instead of embedding the lookup algorithm in the root.
- Table-editor value generation and typed parsing now live in the pure
  `table_editor_values.rs` module; UUID, numeric/decimal, JSON, temporal and
  binary validation no longer depends on `DbProApp`.
- Table mutation capability checks, staged-value lookup/revert and discard
  transitions now use `TableMutationContext`; the root keeps only reload and
  runtime-command orchestration.
- Staged-change transaction planning and retry-target filtering now live in
  `TableMutationState::build_apply_plan`; `apply_staged_changes` only performs
  boundary validation, command dispatch and request lifecycle updates.
- Primary-key row-reload filter construction now lives in
  `TableMutationState::row_reload_filters`, with composite-key metadata
  coverage in the state tests.
- Insert and duplicate-row mapping now live in pure functions in
  `table_editor_values.rs`; identity/generated-column handling, required-field
  validation and typed parsing are covered by focused tests.
- Synthetic-data plan construction now lives in `synthetic_data.rs`; table
  lookup, numeric input validation, inferred generators and bounded FK seed
  pools are covered by focused tests. The native capture adapter also rejects
  framebuffer dimensions that cannot be represented by PNG dimensions instead
  of truncating them.
- Masking preview construction now lives in `masking.rs`; requested-column
  parsing, stable fallback headers, sample rows and mask-rule output are covered
  by focused tests.
- PostgreSQL RLS/table-policy preview planning now lives in `security_rls.rs`
  with a shared quote dialect and explicit request structs; missing identity,
  role parsing and generated SQL are covered by focused tests.
- Schema compare keyed data-diff request validation and effect construction now
  live in `SchemaCompareState`; tests cover required target/table/key fields and
  normalized schema/key payloads.
- `456dc9c7`: Schema Compare rendering now consumes `SchemaCompareViewContext` and returns
  explicit `SchemaCompareAction` intents; the view no longer implements methods
  on `DbProApp`, and the architecture guard freezes that boundary.
- `43a43503`: Query search rendering now consumes `QuerySearchContext`; overlay
  close, match navigation and selection updates stay inside the query feature
  context, with no `DbProApp` dependency in the search module.
- `fe7c554f`: Query output tab chrome now consumes `QueryOutputTabsContext`;
  document-specific tab selection and dock controls are isolated in a guarded
  module without `DbProApp`.
- `9fab05c8`: Query chart and message panes now consume
  `QueryOutputPanesContext`; chart configuration and message presentation no
  longer depend on the composition root.
- `5d42d5e9`: Query explain/history panes now consume
  `QueryOutputActionsContext` and return explicit actions; only the root
  applies runtime/document orchestration.
- Monitoring state and snapshot/workload/session-control command planning now
  live in `monitoring_state.rs`; tests cover bounded workload requests and
  explicit confirmation flags for destructive commands.
- The architecture guard now freezes `monitoring_state.rs` as an explicit-state
  module that may not depend on the composition-root type.
- Audit filter construction and selected/bookmarked export planning now live in
  `audit_state.rs`; tests cover bounded load effects and export preconditions.
- The architecture guard now freezes `audit_state.rs` as an explicit-state
  module that may not depend on the composition-root type.
- The former `database_feature_states.rs` catch-all was removed; each remaining
  database-management aggregate now has an explicit state module and the guard
  checks those modules for composition-root dependencies.
- PostgreSQL settings, FDW, logical replication and event-trigger command
  payload construction now lives in the owning state modules; focused FDW
  coverage checks copied form values and explicit confirmation.
- Security role, membership, privilege and RLS-inspection command construction
  now lives in `SecurityState`; focused coverage checks the RLS boundary's
  required schema/table invariant.
- RLS preview application now also builds its `ExecuteDdl` effect in
  `SecurityState`, with coverage proving empty preview SQL cannot dispatch.
- Saved-query and query-folder refresh effects now build in `QueryLibraryState`,
  with focused coverage for both command identities.
- Table metadata and DDL effects now build in `TableState`; table-data request
  effects build in `TableDataQueryState`, keeping paging/filter/sort and row
  reload lifecycle out of metadata state. Focused coverage verifies empty DDL
  is rejected at the state boundary.
- Migration apply and schema-workbench DDL effects now build in their owning
  aggregates; focused coverage verifies both apply paths reject missing plans.
- Query-folder creation and saved-query save/rename/delete effects now build in
  `QueryLibraryState`; focused coverage checks folder normalization and the
  empty-folder precondition.
- Backup/restore and file-picker effects now build in `OverlayState`; focused
  coverage checks both path-bearing effects and their connection identity.
- Palette and explorer connection switching now use the lifecycle-owned
  `Connect` effect builder; focused coverage checks request and connection
  identity preservation.
- The architecture guard now freezes `table_editor_context.rs` and
  `table_editor_values.rs` as explicit-state modules that may not depend on
  the composition-root type.
- `VisualQueryBuilderState` now owns visual-builder form inputs, the
  `VisualQueryModel`, validation errors, SQL preview and state transitions for
  table/join/column/filter/order/import operations; the view retains only egui
  rendering plus active-document and feedback adapters.
- The architecture guard also freezes `visual_query_builder_state.rs` as an
  explicit-state module that may not depend on the composition-root type.
- `QueryOutputState` now owns the active output tab and per-document output-tab
  overrides.
- `TableDataState` now owns grid projection/layout, filtering/sorting,
  selection, cell editor, inspector and insert-row interaction state.
- `TableState` now owns table metadata, table view, introspection/DDL requests,
  metadata searches and details. `TableDataQueryState` owns paged data,
  filters, sorts, data requests and row reload state.
- `TableMutationState` now owns staged changes, mutation requests, retries and
  conflict/apply state.
- `AgentState` now owns provider settings, composer input and agent sessions;
  the saved-task scheduler remains in `DbProApp`.
- `SchemaExplorerState` now owns schema loading, selection, navigation cache,
  pinned/recent tables and schema-object view state.
- `QueryEditorState` now owns editor overlays, visual-builder drafts,
  diagnostics caches, problem filters and query history.
- `WorkspaceFilesState`, `DiagramState`, named database-management aggregates
  from `database_feature_states.rs`, `SchemaCompareState`,
  `PaletteState`, `QueryExecutionPolicyState`, `QueryLibraryState`,
  `SavedTaskState`, `WorkspaceSessionState`, `OverlayState`, `FeedbackState`,
  `PreferencesState` and `WelcomeState` now own their feature state.
- `ConnectionLifecycleState` now also owns connection status and fallback name;
  `SchemaExplorerState` owns persisted explorer pane heights.
- Connection dialog fields are private to the `connection` feature module, and
  saved-connection storage is private behind catalog read-model methods
  (`iter`, `get`, `find`, `len`, `is_empty`).
- Connection lifecycle request flags and fallback naming are private behind
  lifecycle methods; tests use explicit lifecycle setup APIs rather than
  production field access.
- Connection connected status is private behind `is_connected` and
  `set_connected` lifecycle APIs.
- Active connection identity is private behind lifecycle accessors; consumers
  no longer read or mutate the storage field directly.
- Pending request/target and connection failure storage are private behind
  lifecycle APIs; production consumers no longer access those storage fields
  directly.
- Connection dialog rendering is now driven by `ConnectionDialogView<'a>` with
  explicit state/runtime/feedback dependencies; `view.rs`, `form_fields.rs`
  and `advanced_panels.rs` no longer implement methods on `DbProApp`.
- Active connection, schema and statusbar helpers are pure functions in
  `connection_status.rs`; the module no longer implements methods on
  `DbProApp`, and the architecture guard enforces that boundary.
- Connection lifecycle event reducers are pure functions in
  `connection_events.rs`; the root wrapper only performs follow-up runtime
  orchestration after the reducer returns an explicit transition result.
- Connection deletion confirmation now lives in
  `connection/delete_dialog.rs` and receives explicit overlay/catalog/
  lifecycle/runtime/feedback dependencies; it no longer implements a
  `DbProApp` method.
- Query-folder deletion confirmation now lives in
  `query_folder_delete_dialog.rs`; the old mixed connection/folder confirmation
  module was removed.
- `DbProApp` now composes one `ConnectionFeatureState` aggregate containing
  catalog, lifecycle and dialog sub-states; the architecture allowlist rejects
  the former three root fields.
- `DbProApp` now composes one `WorkspaceFeatureState` aggregate containing
  shell/navigation, local-file activity and named-session sub-states; the
  architecture allowlist rejects the former `workspace_files` and
  `workspace_sessions` root fields. Existing shell field access is preserved
  through a typed `Deref` facade while file/session ownership remains explicit
  under `workspace.files` and `workspace.sessions`.
- Schema event handling now lives in explicit-state reducers in
  `schema_events.rs`; the root wrapper only performs the follow-up table-info
  request returned by `SchemaLoadedTransition`. Stale request rejection and
  missing-table reconciliation are covered by reducer tests, and the
  architecture guard rejects `DbProApp` references in the reducer module.
- Agent provider/workflow event handling now lives in explicit-state reducers
  in `agent_events.rs`; provider configuration failure is request-scoped and
  reducer tests cover stale configuration events and provider readiness. Toast
  emission is owned by `FeedbackState`, not an app-only helper, and the
  architecture guard rejects `DbProApp` references in the agent reducer.
- Agent workflow event routing now also lives in `agent_events.rs`; document,
  session and run identity checks are performed against `AgentState` there,
  while the `DbProApp` method is only a thin composition-root adapter.
- Agent document snapshots and UI-to-core context conversion now live in the
  pure `agent_context.rs` mapper; it receives explicit query/schema inputs and
  no longer depends on `DbProApp`.
- Table event handling now lives in explicit-state reducers in
  `table_events.rs`; metadata/data/row-reload transitions return typed cache
  invalidation and staged-apply effects, while the root only executes those
  follow-ups. Existing table mutation, reload and stale-request tests remain
  green, and the architecture guard rejects `DbProApp` references in the
  table reducer.
- Saved-query and query-folder read-model replacement now lives in
  `query_library_events.rs`; the root keeps only thin adapters for event
  routing, and reducer tests cover replacement semantics.
- SQL prediction ready/failed handling now lives in
  `query_prediction_events.rs`; stale request and document-version checks stay
  in the query-session reducer boundary, while the root only adapts runtime
  event payloads.
- Saved-query completion now lives in `query_save_events.rs`; the reducer
  returns an explicit close-document transition and the root performs only the
  resulting tab orchestration.
- Query-history retention now lives in `query_history_events.rs`; the reducer
  owns the 500-entry cap and only receives `QueryEditorState` plus a history
  record.
- Query queued feedback now lives in `query_queue_events.rs`; the reducer
  receives only `FeedbackState` and the request identity.
- Database-management event state transitions now live in
  `management_events.rs` for monitoring, audit, pg settings, FDW, replication,
  event triggers, security and data compare. `operation_events.rs` retains
  only composition-root orchestration and cross-feature follow-ups.
- File-picker state transitions now live in `file_picker_events.rs`, and DDL
  completion now returns a typed refresh transition from `ddl_events.rs`.
  Workspace opening and schema/RLS requests remain explicit root side effects.
- Explain completion and query cancellation now live in
  `query_execution_events.rs`, including output-tab selection, document
  cleanup and cancellation history.
- Single-statement query completion now lives in `query_result_events.rs`
  behind an explicit `QueryResultContext`, including grid invalidation,
  history, output selection and active-document presentation state.
- Multi-statement query completion now lives in
  `query_multi_result_events.rs` behind the same explicit state boundary;
  diagnostics, history status and result presentation remain request-scoped.
- Query-local failure handling now lives in `query_failure_events.rs`; the
  root only routes failures to other feature reducers before invoking the
  explicit query failure context. The old mixed `events_query.rs` module is
  deleted.
- Recent-table MRU ownership now lives on `SchemaExplorerState`; workspace,
  palette, explorer and tests use the state API instead of a root facade.
- Palette open lifecycle now lives on `PaletteState`; navigation, welcome,
  sidebar, query shortcuts and tests no longer call a `DbProApp` palette
  mutation facade.
- New-connection dialog opening now lives on `ConnectionFeatureState`; all
  shell entry points call the feature transition directly.
- Workspace close/refresh lifecycle now lives on `WorkspaceFilesState`; only
  the native folder-picker command remains in the composition root.
- Workspace search/replace, task, refactor, context, schema snapshot and drift
  transitions now live on `WorkspaceFilesState`; the root only composes the
  schema input needed by snapshot/drift operations.
- Workspace Git status/stage/unstage/diff/commit transitions and external-file
  change detection now live on `WorkspaceFilesState`; query documents are
  passed in as an explicit snapshot at the view boundary.
- Schema compare snapshot, diff and migration-plan transitions now live on
  `SchemaCompareState`; only migration apply remains root orchestration because
  it allocates a request and dispatches provider work.
- Transaction policy transitions now live on `QueryExecutionPolicyState` and
  return explicit SQL effects; the root only dispatches the returned effect.
- The capture-only native entrypoint now uses the feature-owned new-connection
  helper, so the capture-feature release build stays aligned with the dialog
  lifecycle migration.
- Named-session store mutations and persistence now live on
  `WorkspaceSessionState`; capture/restore of cross-feature layout remains
  explicit composition-root orchestration.
- Query output-tab override and active-tab mutations now live on
  `QueryOutputState`; the root only resolves the active document identity.
- Grid projection epoch and row-identity cache invalidation now live on
  `TableDataState`; query/result reducers and table event orchestration call
  that explicit state API.
- Query document collection invariants now live on `QuerySessionState`; add,
  select, remove, keep-one, truncate-right and reset operations no longer
  mutate the document vector and active index ad hoc in `DbProApp` helpers.
- Active query text, explain state, running request, result selection/count and
  message projections now live on `QuerySessionState`; the root keeps only
  cancellation, connection lookup and grid-invalidation orchestration.
- Query-document connection/schema metadata changes and prediction
  invalidation now live on `QuerySessionState`; the root only dispatches the
  returned prediction-cancel command.
- Result-grid row/cell selection and mutation-error matching now live on
  `TableDataState` and `TableMutationState`; grid cells/views no longer call
  selection helpers through `DbProApp`.
- Result-grid keyboard navigation target calculation now lives on
  `TableDataState`; the root handles only egui input and commit-edit effects.
- Result-grid layout scope, persistence and restore transitions now live on
  `TableDataState`; the former `grid_layout.rs` root facade is deleted and
  table-opening flows pass an explicit layout scope into the state owner.
- Result-grid row-identity derivation and cache rebuilding now live on
  `TableDataState`; table-editor code keeps mutation orchestration while the
  grid identity algorithm has one state owner.
- Result-grid identity lookup now also lives on `TableDataState`; grid views
  pass table metadata explicitly and no longer depend on a root lookup facade.
- Mutation-error clearing now lives on `TableMutationState`; table-editor
  views keep feedback and request orchestration but no longer implement the
  target-matching state transition.
- Selected-row projection now lives on `TableDataState`; clipboard code only
  consumes the state-owned indexes for copy/export operations.
- Table metadata primary-key and column-write-policy projections now live on
  `TableState`; connection mutability remains an explicit lifecycle concern at
  the composition boundary.
- Active query buffer-version projection now lives on `QuerySessionState`; query
  dispatchers consume the state API instead of a root helper.
- Active query running-request projection now also lives on
  `QuerySessionState`; query cancellation/dispatch checks read the owning
  session directly.
- Explain-plan/request and result-count projections now live on
  `QuerySessionState`; output, agent and navigation surfaces read query state
  directly without root projection facades.
- Active query messages projection now lives on `QuerySessionState`; output
  and navigation surfaces consume the session-owned message slice directly.
- Active query result projection now lives on `QuerySessionState`; output,
  navigation, palette and agent surfaces consume the session-owned result.
- Active query text read projection now lives on `QuerySessionState`; callers
  read the session directly, while the root setter remains only for prediction
  cancellation plus text mutation orchestration.
- Active output-tab read projection now lives on `QueryOutputState`; output
  rendering reads the tab by active document identity, while tab mutations
  remain explicit orchestration transitions.
- Active output-tab mutation now also lives on `QueryOutputState`; query views
  pass the active document identity directly and the root tab setter is gone.
- Per-document output-tab routing now lives on `QueryOutputState`; the active
  document condition is evaluated by the feature state, not by query view code.
- Row-identity matching now uses the published `table_events` function
  directly; the composition root no longer exposes a forwarding helper.
- Toast mutations now live on `FeedbackState`; error/success/info notifications
  no longer route through `DbProApp` wrappers.
- Activity-bar rendering now lives in an explicit renderer that owns only
  workspace-shell state and returns navigation intents; it no longer
  implements a `DbProApp` method. The architecture guard enforces this seam.
- Named workspace-session capture, restore, duplication and persistence now
  run through `WorkspaceSessionContext` with explicit aggregate dependencies;
  the session module no longer implements `DbProApp` methods.
- Runtime event dispatch now lives in `crates/ui/src/event_router.rs`; feature
  transition handlers remain independently callable from the router.
- Agent and table event handlers now live in `agent_events.rs` and
  `table_events.rs`.
- Connection, schema and operation event handlers now live in their own
  feature event modules; `events.rs` contains only the event pump and tests.
- `event_router.rs` is now a pure event-to-handler dispatch table; database
  operation, agent, query and feature-failure transitions no longer mutate
  state inline in the router.
- Runtime event application is bounded to `64` events per egui frame; a full
  batch schedules another repaint. The native adapter uses a bounded
  `sync_channel(256)` and retries asynchronously when the UI queue is full.
- Runtime command sends are centralized through the dispatch adapter; closed
  command boundaries are logged and surfaced as a user-visible runtime error.
- Legacy `RunAgent`/`ExecuteAgentTool` commands and ignored tool completion
  events were removed; the runtime now exposes one agent workflow command/event
  contract.
- Aggregate fields are scoped to the app boundary; the architecture guard
  rejects crate-public state fields in feature state modules and connection
  state modules.
- `scripts/check-ui-architecture.sh`: PASS; it allowlists the composition-root
  fields, rejects event handlers in `events.rs`, rejects direct state access in
  `event_router.rs`, requires bounded event draining, and rejects feature code
  bypassing the command dispatch adapter.
- Shared dialog layout now reserves an explicit chrome budget, centers the card
  inside the safe viewport, gives the body its own scroll budget, and renders a
  full-width separated header with the close action aligned to the right.
- Deterministic native capture of the affected New Connection modal: PASS at
  logical `1280x800` (`/tmp/db-pro-evidence-core-error-latest.png`).
  The inspected framebuffer shows balanced vertical margins, a separated
  header, right-aligned close action, independently scrolling body and sticky
  footer.
- Deterministic state captures at logical `1280x800`: normal Welcome state
  (`/tmp/db-pro-evidence-core-normal-1280x800.png`), loading Welcome state
  (`/tmp/db-pro-evidence-core-loading-1280x800.png`) and New Connection error
  state (`/tmp/db-pro-evidence-core-error-1280x800.png`). The error alert is
  visible immediately below the separated header instead of being hidden at the
  end of the scroll body.
- Release runtime smoke: PASS; `target/release/db-pro-native` launched from
  the verified HEAD and rendered the Welcome/empty state in a `1440x870` DB Pro
  window. Capture was inspected from the native window after startup settled.
- Post-refactor release framebuffer capture: PASS at logical `1280x800`
  (`/tmp/db-pro-native-post-refactor.png`); Welcome/empty state, activity rail,
  sidebar, query tabs and status bar were inspected after the state-owner
  changes.
- Latest query-state release framebuffer capture: PASS at logical `1280x800`
  (`/tmp/db-pro-native-post-query-state-refactor.png`); the same Welcome/empty
  surface was inspected after the active-query projection extractions.
- Unit tests for the extracted aggregates are included in the UI test suite.
- Query-document context tests cover explicit connection/schema binding and
  document-owned output-tab cleanup during close.
- `cargo check -p db-pro-ui`: PASS.
- `cargo fmt --all`: executed.
- `cargo clippy -p db-pro-ui --all-targets -- -D warnings`: PASS.
- `cargo test -p db-pro-ui --lib`: 632 passed, 0 failed.
- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace --no-fail-fast`: 1304 passed, 0 failed, 42 ignored;
  all workspace doc-tests passed with 0 tests.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `cargo build --release --locked -p db-pro-native --features capture`: PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: 12 pass, 4 warnings, 0 failures; warnings are ratcheted size/file/clone heuristics.
- Synthetic/transfer harness extraction: `navigation_view.rs` reduced from
  3503 to 3021 lines; the 482-line implementation now lives in
  `transfer_harness_view.rs`.
- Post-extraction focused verification: `cargo check -p db-pro-ui` PASS,
  `cargo test -p db-pro-ui --lib` 632 passed, architecture guard PASS,
  `cargo clippy -p db-pro-ui --all-targets -- -D warnings` PASS, and
  `git diff --check` PASS.
- Table mutation-dialog extraction: `table_editor_view.rs` reduced from 2643
  to 2146 lines; the 504-line discard/pending/conflict slice now lives in
  `table_mutation_dialogs_view.rs`. Focused clippy, architecture guard, clean
  scan and 632 UI tests all PASS; clean scan remains 12 pass, 4 ratcheted
  warnings, 0 failures.
- Insert-row workflow extraction: `table_editor_view.rs` reduced from 2146 to
  1795 lines; the 357-line open/duplicate/submit/dialog slice now lives in
  `table_insert_row_view.rs`. Focused clippy, architecture guard, clean scan
  and 632 UI tests all PASS; clean scan remains 12 pass, 4 ratcheted warnings,
  0 failures.
- Security activity extraction: `navigation_view.rs` reduced from 3021 to 2438
  lines; the 586-line roles/RLS/policy slice now lives in
  `security_activity_view.rs`. Focused clippy, architecture guard, clean scan
  and 632 UI tests all PASS; clean scan remains 12 pass, 4 ratcheted warnings,
  0 failures.
- Management activity extraction: `navigation_view.rs` reduced from 2438 to
  971 lines; the 1470-line monitoring/audit/settings/FDW/replication/event
  trigger slice now lives in `database_management_view.rs`. Focused clippy,
  architecture guard, clean scan and 632 UI tests all PASS; clean scan remains
  12 pass, 4 ratcheted warnings, 0 failures.
- Final source checkpoint: `426c8790` on `main`; worktree clean after the
  management extraction.
- Final full-gate rerun at this checkpoint: `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `cargo test --workspace --no-fail-fast` (1304 passed, 0 failed,
  42 ignored), `cargo build --release --locked -p db-pro-native`, and the
  capture-feature release build all PASS.
- Final runtime evidence: `/tmp/db-pro-native-core-current.png`, captured
  from the release binary at logical `1280x800` with the New Connection modal
  open; centered card, separated header, right-aligned close control, scroll
  body and footer were visually inspected.
- Management view topology extraction: deleted the 1490-line aggregate
  `database_management_view.rs` and split it into seven feature-owned view
  modules. Monitoring snapshot rendering is now split into health/local,
  sessions, server stats, workload and confirmation methods. Focused check,
  632 UI tests, clippy, architecture guard and clean scan all PASS; clean scan
  reports 14 pass, 2 ratcheted warnings, 0 failures.
- Runtime event/lifecycle extraction: `app.rs` reduced from 1187 to 734 lines;
  runtime transitions live in `runtime_event_handlers.rs` (231 lines) and the
  `eframe::App` adapter plus persistence/frame helpers live in
  `app_lifecycle.rs` (253 lines). Focused clippy, architecture guard and 632 UI
  tests PASS; clean scan is 16 pass, 0 warnings, 0 failures.
- Table editor state aggregation: `DbProApp` now owns one `TableEditorState`
  aggregate instead of separate `table_state`, `table_data` and
  `table_mutation` root fields. Consumers use `self.table.state`,
  `self.table.data` or `self.table.mutation`; the architecture allowlist was
  updated to require the aggregate. Focused UI tests (632 passed), focused
  clippy, architecture guard and clean scan PASS; clean scan reports 12 pass,
  4 ratcheted baseline warnings, 0 failures.
- Current release runtime capture: PASS at logical `1280x800`
  (`/tmp/db-pro-native-table-data-query.png`) after rebuilding both
  `db-pro-native` release variants. The New Connection surface still shows a
  centered modal card, separated header, right-aligned close action, scrollable
  body and footer; the state-owner refactor did not regress the visual surface.
- Query state aggregation: `DbProApp` now owns one `QueryFeatureState`
  aggregate instead of separate query session/editor/output/execution/library
  root fields. Focused UI tests (632 passed), focused clippy, architecture
  guard and clean scan PASS; clean scan reports 12 pass, 4 ratcheted baseline
  warnings, 0 failures.
- Storage hydration extraction: startup key parsing now lives in
  `app_storage.rs` behind `NativeStorageContext` and
  `NativeStorageDependencies`; `app_state.rs` only sequences restore phases.
  Focused compile, clippy, architecture guard and 632 UI tests PASS; clean
  scan reports 15 pass, 1 ratcheted warning, 0 failures.
- Table data-query boundary extraction: `TableState` no longer owns result,
  paging, filters, sorts or row-reload lifecycle. Those concerns now live in
  `TableDataQueryState`; table reducers and views receive the explicit state
  boundary. Focused UI tests (632 passed), clippy, architecture guard and
  clean scan PASS; clean scan reports 12 pass, 4 ratcheted warnings, 0 failures.
- Source checkpoint `96d52bd7`: full release gate rerun on `main` passed
  (`cargo fmt --all -- --check`, workspace check/clippy, workspace tests;
  1304 passed, 42 ignored, 0 failed), both native release builds passed, and
  the deterministic capture at `/tmp/db-pro-native-table-data-query.png`
  passed at logical `1280x800`.
- Table-data behavior ownership follow-up: filter-operator compatibility and
  table-query reset/invalidate/page transitions now live in
  `TableDataQueryState`; focused UI tests (632 passed), clippy, fmt and
  architecture guard PASS.
- Final merged `main` checkpoint `5332876c`: after merging the concurrent SQL
  Server PR, workspace format/check/clippy/tests and both native release builds
  passed again; final modal capture is `/tmp/db-pro-native-final-main.png` at
  logical `1280x800`.
- Grid layout ownership follow-up: `TableDataState` now owns column ordering,
  visibility, movement, auto-sizing and width calculation; `result_grid_view`
  no longer exposes those as `DbProApp` methods. Focused UI tests (632 passed),
  clippy, fmt and architecture guard PASS.
- Table editing ownership follow-up: cell-edit buffers, inspector state,
  discard confirmations and insert-row form state now live in
  `TableEditingState`, separate from grid projection/layout/selection state.
  Focused UI tests (632 passed), clippy, architecture guard and diff checks
  PASS.
- Design Mode action ownership follow-up: ER foreign-key draft parsing,
  mutation-plan preview and query-runtime apply orchestration now live in
  `diagram_design_actions.rs`; `diagram_view.rs` retains the Design Mode UI
  surface and diagram rendering. Focused UI check, clippy, 632 UI tests,
  architecture guard and diff checks PASS.
- ER canvas interaction ownership follow-up: empty state, canvas surface,
  pan handling and click-through table navigation now live in
  `diagram_canvas_view.rs`; `diagram_view.rs` coordinates schema candidates,
  toolbar and Design Mode. Focused UI check, clippy, 632 UI tests,
  architecture guard, clean scan and diff checks PASS.
- Current source checkpoint: `6b258257` on `main`. Full regression gate PASS:
  `cargo fmt --all -- --check`, workspace check, workspace clippy with
  `-D warnings`, workspace tests (`0 failed`), native release build and
  capture-feature release build.
- Current runtime evidence: `/tmp/db-pro-native-core-final.png`, captured from
  the rebuilt release binary at logical `1280x800` with the
  New Connection modal open. Visual inspection confirms the centered card,
  separated header, right-aligned close control, scrollable body and footer.
- Design Mode panel ownership follow-up: draft table/column/FK form rendering
  now lives in `diagram_design_panel_view.rs`; `diagram_view.rs` only
  coordinates whether the panel is shown. Focused UI tests (632 passed),
  clippy, architecture guard and clean scan (14 pass, 2 ratcheted warnings)
  PASS. One timing-sensitive chart performance assertion exceeded its budget
  once at 300.7ms and passed on the isolated rerun and subsequent full UI run;
  no product test failure remains.
- Database-management state ownership follow-up: audit, event-trigger, FDW,
  masking, monitoring, PostgreSQL settings, replication, routine, security,
  synthetic-data and transfer state now cross the single
  `DatabaseManagementState` aggregate through `DbProApp.management`. Focused
  UI tests (632 passed), clippy, architecture guard, clean scan and diff
  checks PASS.
- Schema workspace state ownership follow-up: explorer, schema workbench,
  schema compare and ER diagram state now cross the single
  `SchemaWorkspaceState` aggregate through `DbProApp.schema`; storage and
  reducer contexts still receive explicit child dependencies. Focused UI
  tests (632 passed), clippy, architecture guard and diff checks PASS.
- Schema comparison view ownership follow-up: schema-compare sidebar,
  migration/data-compare surface and keyed data-diff dispatch now live in
  `schema_compare_view.rs`; `navigation_view.rs` is reduced to shell and
  activity surfaces. Focused UI tests (632 passed), clippy, architecture
  guard, clean scan (15 pass, 1 ratcheted warning) and diff checks PASS.
- Transfer activity ownership follow-up: backup/restore entry point,
  synthetic seed, masking preview, transfer harness controls and job history
  now live in `transfer_activity_view.rs`; `navigation_view.rs` is reduced to
  shell/status/output and schema/diagram navigation. Focused UI tests (632
  passed), clippy, architecture guard, clean scan (15 pass, 1 ratcheted
  warning) and diff checks PASS.
- Shell chrome ownership follow-up: top bar, status bar and output dock now
  live in `shell_chrome_view.rs`; `navigation_view.rs` is reduced to the
  remaining navigation activity composition. Focused UI tests (632 passed),
  clippy, architecture guard, clean scan (15 pass, 1 ratcheted warning) and
  diff checks PASS.
- Query tool ownership follow-up: query action menu/editor actions now live
  in `query_actions_view.rs`, while find overlays/bars live in
  `query_search_view.rs`; `query_view.rs` is below the 800-line file boundary.
  Focused UI tests (632 passed), clippy, architecture guard, clean scan (15
  pass, 1 ratcheted warning) and diff checks PASS.
- Palette ownership follow-up: static Quick Open/Command catalog builders now
  live in `palette_catalog.rs`, and palette action routing lives in
  `palette_actions.rs`; `palette_view.rs` is reduced to indexing/filtering and
  dialog coordination. Focused UI tests (632 passed), UI clippy, architecture
  guard, clean scan (14 pass, 2 ratcheted warnings) and diff checks PASS.
- Table editor ownership follow-up: data-grid/paging rendering now lives in
  `table_data_view.rs`, while row editing and staged mutation lifecycle live in
  `table_mutation_actions.rs`; `table_editor_view.rs` is reduced to DDL and
  table-request/filter coordination. Focused UI tests (632 passed), UI clippy,
  architecture guard and diff checks PASS; clean scan has no failures and only
  ratcheted legacy function-size/clone warnings.
- Query editor ownership follow-up: hover/signature/completion support now
  lives in `query_editor_support.rs`; `query_editor_panel.rs` is reduced to
  editor interaction orchestration. Focused UI tests (632 passed), UI clippy,
  architecture guard and diff checks PASS; clean scan has no failures and one
  ratcheted function-size warning group.
- Table metadata ownership follow-up: structure/columns now live in
  `table_structure_view.rs`, while foreign keys, constraints and dependencies
  live in `table_relations_view.rs`; `table_metadata_view.rs` owns indexes only.
  Focused UI tests (632 passed), UI clippy, architecture guard and diff checks
  PASS; clean scan has no failures.
- IDE workspace ownership follow-up: persisted workspace types now live in
  `ide_workspace_types.rs`, and bounded filesystem scanning/indexing lives in
  `ide_workspace_scan.rs`; the state operation module is 712 lines. Focused UI
  tests (632 passed), UI clippy, architecture guard and diff checks PASS; clean
  scan has no failures after documenting the retained contract allowance.
- Files activity ownership follow-up: workspace tabs/tree/search/tasks/graph/Git
  now live in `files_activity_tabs.rs`; `files_activity_view.rs` is reduced to
  the 201-line compositor. Focused UI tests (632 passed), UI clippy,
  architecture guard and diff checks PASS; clean scan has no failures.
- Workspace shell ownership follow-up: tab lifecycle/rendering now lives in
  `workspace_tabs_view.rs`, reusable tab chrome in `workspace_tab_primitives.rs`,
  and `workspace_view.rs` is reduced to an 18-line content compositor. Focused
  UI tests (632 passed), UI clippy, architecture guard and diff checks PASS.
- Chart ownership follow-up: chart configuration/projection, numeric parsing,
  aggregation, downsampling and engine tests now live in `chart_engine.rs`;
  `chart_view.rs` retains only the egui renderer facade and public API
  re-exports at source SHA `0ba94810`. Focused UI tests (632 passed), UI
  clippy, architecture guard, clean scan (14 pass, 2 ratcheted warnings) and
  diff checks PASS. The timing-sensitive diagram benchmark
  failed once during the first full run, then passed in isolation and in the
  subsequent full UI run.
- Agent thread ownership follow-up: message/activity/result rendering,
  confirmation preview/actions, empty state and retry/thinking controls now
  live in `agent_thread_view.rs`; panel/settings/context composition remains in
  `agent_view.rs` at source SHA `f0ddf324`. Focused UI tests (632 passed), UI clippy, architecture
  guard, clean scan (15 pass, 1 ratcheted legacy function-size warning) and
  diff checks PASS.
- Schema Workbench action ownership follow-up: mutation request construction,
  object preview planning, database actions and DDL application now live in
  `schema_workbench_actions.rs`; `schema_workbench.rs` retains state and view
  composition at source SHA `08acab31`. Focused UI tests (632 passed), UI clippy, architecture guard,
  clean scan (14 pass, ratcheted legacy/planner warnings) and diff checks PASS.
- Runtime protocol ownership follow-up: `UiCommand`/`UiEvent` now live in
  `runtime_protocol.rs`, while the bounded `TaskBridge` transport and channel
  limits live in `task_bridge.rs`; `runtime.rs` retains DTOs and public API
  re-exports at source SHA `b85d65da`. Focused UI tests (632 passed), UI
  clippy, architecture guard, clean scan (16 pass, 0 warnings) and diff
  checks PASS.
- Final core refactor gate at source SHA `b85d65da`: `cargo fmt --all --
  --check`, workspace `cargo check`, workspace clippy with `-D warnings`,
  workspace tests (`404 core`, `119 infrastructure`, `34 runtime`, `30
  tauri`, `21 native`, `632 UI`, plus integration suites), release native
  build, capture build and architecture guard all PASS. Runtime capture is
  recorded at `/tmp/db-pro-native-core-b85d65da.png` for the New Connection
  surface at logical `1280x800`; the macOS host still clamps requested
  `1440x900` and `1920x1080` captures to logical height `838`.
- Runtime model-family ownership follow-up at source SHA `4bb3d21d`:
  connection, schema/table and query/result/history DTOs now have separate
  modules; `runtime.rs` is a 104-line facade. Focused UI tests (632 passed),
  UI clippy, architecture guard, clean scan (16 pass, 0 warnings) and diff
  checks PASS.
- ER diagram boundary refactor at source SHA `3f8ec33b`: layout polling,
  diagram canvas and design-mode actions now receive `DiagramViewContext` and
  return typed `DiagramAction` intents; only composition-root orchestration
  applies cross-feature table/query effects. Focused UI check, clippy, 632 UI
  tests, architecture guard, clean scan and diff checks PASS.
- Static catalog boundary at source SHA `74c0f870`: palette command builders
  and shared SQL snippets no longer attach pure data to `DbProApp`; the new
  snippet module has a stability test. Focused clippy, 633 UI tests,
  architecture guard, clean scan and diff checks PASS.
- Query diagnostics boundary at source SHA `94b8dbb3`: parser/lint analysis,
  diagnostics debounce/cache refresh and formatting no longer implement
  `DbProApp` methods. Focused check, clippy, 633 UI tests, architecture guard,
  clean scan and diff checks PASS.
- Sidebar renderer boundary at source SHA `01ec23d1`: diagram navigation and
  maintenance controls now receive explicit inputs/state instead of a root
  receiver. Focused check, clippy, 633 UI tests, architecture guard, clean
  scan and diff checks PASS.
- Result export boundary at source SHA `f64ae90d`: CSV/TSV, exact JSON, SQL
  INSERT/COPY and literal formatting are pure module functions rather than
  `DbProApp` methods. Focused check, clippy, 633 UI tests, architecture guard,
  clean scan and diff checks PASS.
- Grid navigation boundary at source SHA `84f94f26`: keyboard selection now
  receives `GridNavigationContext`; edit commit stays explicit at the table
  orchestration boundary. Focused check, clippy, 633 UI tests, architecture
  guard, clean scan (16 pass, 0 warnings) and diff checks PASS.
- Final full gate at source SHA `4bb3d21d`: `cargo fmt --all -- --check`,
  workspace `cargo check`, workspace clippy with `-D warnings`, workspace
  tests, release native build, capture build and architecture guard all PASS.
  The current New Connection runtime capture is
  `/tmp/db-pro-native-core-4bb3d21d.png` at logical `1280x800`; the dialog is
  centered, its header is separated by a divider, and the close action is
  aligned to the header's right edge. The release binary is running for
  manual verification.
- Query output boundary follow-ups at source SHA `441d3972`: output tabs,
  chart/messages, explain/history and result selection/export now use explicit
  contexts and typed intents. Full gate PASS: fmt, workspace check, workspace
  clippy with `-D warnings`, workspace tests (`633` UI tests passed), native
  release build, capture-feature release build, architecture guard and clean
  scan (16 pass, 0 warnings). Runtime capture is
  `/tmp/db-pro-native-core-441d3972.png`; the New Connection dialog is
  centered with a separated header and right-aligned close control.
- Core boundary follow-ups at source SHA `764548e1`: result-grid toolbar,
  query context strip and grid selection projection now have explicit context
  or pure-state module boundaries. Full gate PASS: fmt, workspace check,
  workspace clippy with `-D warnings`, workspace tests (`630` UI tests and
  `0 failed` overall), native release build, capture-feature release build,
  architecture guard and clean scan (16 pass, 0 warnings). Runtime capture is
  `/tmp/db-pro-native-core-764548e1.png`; the New Connection dialog remains
  centered with a separated header and right-aligned close control.
- Query run-control follow-up at source SHA `072f44be`: Run/Stop rendering now
  returns typed command intents and the root remains the command executor.
  Full gate PASS: fmt, workspace check, workspace clippy with `-D warnings`,
  workspace tests (`630` UI tests and `0 failed` overall), native release
  build, capture-feature release build, architecture guard and clean scan
  (16 pass, 0 warnings). Runtime capture is
  `/tmp/db-pro-native-core-072f44be.png`; the New Connection dialog remains
  centered with a separated header and right-aligned close control.
- Query context-picker follow-up at source SHA `ecde08d7`: picker rendering
  now returns typed connection/schema/close intents and the root remains the
  document mutation boundary. Focused UI check, clippy, 630 UI tests,
  architecture guard and clean scan PASS; both native release builds were
  rebuilt from this SHA. Runtime capture is
  `/tmp/db-pro-native-core-ecde08d7.png`; the New Connection dialog remains
  centered with a separated header and right-aligned close control.
- Query parameter-panel follow-up at source SHA `925b8b83`: placeholder
  discovery and in-memory parameter editing now cross an explicit session
  context. Focused UI check, clippy, 630 UI tests, architecture guard and
  clean scan PASS; full workspace check/clippy/tests also pass (`0 failed`),
  both native release builds were rebuilt from this SHA. Runtime capture is
  `/tmp/db-pro-native-core-925b8b83.png`; the New Connection dialog remains
  centered with a separated header and right-aligned close control.
- Schema Workbench secondary-view follow-up at source SHA `edd17b0f`:
  dependency navigation and docs export now consume explicit workbench
  context and typed actions. Focused UI check, clippy, 630 UI tests,
  architecture guard and clean scan PASS; release and capture builds were
  rebuilt from this SHA. Runtime capture is
  `/tmp/db-pro-native-core-edd17b0f.png`; the New Connection dialog remains
  centered with a separated header and right-aligned close control.
- Table-data placeholder follow-up at source SHA `aef4c270`: loading and error
  rendering now consume `TableDataPlaceholderContext` and return a typed retry
  intent; the root remains the request executor. Focused UI check, clippy, 630
  UI tests, architecture guard and clean scan PASS. Current full-gate runtime
  capture is recorded below.
- Table-data pagination follow-up at source SHA `eb03c668`: page navigation and
  page-size selection now consume explicit pagination context and return typed
  request/reset intents. Focused UI check, clippy, 630 UI tests, architecture
  guard and clean scan PASS. Current full-gate runtime capture is recorded
  below.
- Table-data filter follow-up at source SHA `00de1a97`: filter scope,
  operator/input controls and filter chips now consume explicit context and
  return typed commit/reload/remove/clear intents. Focused UI fmt/check,
  clippy, 630 UI tests, architecture guard and clean scan PASS; current
  full-gate runtime capture is recorded below.
- Table-data sort follow-up at source SHA `cffae04f`: sort label/selection and
  staged-change guard now consume explicit context and return typed reload or
  blocked intents. Focused UI fmt/check, clippy, 630 UI tests, architecture
  guard and clean scan PASS; current full-gate runtime capture is recorded
  below.
- Table-data mutation-toolbar follow-up at source SHA `fe7c5aea`: refresh,
  staged-change, mutation-failure and selection-status controls now consume
  explicit context and return typed intents. Focused UI fmt/check, clippy, 630
  UI tests, architecture guard and clean scan PASS; current full-gate runtime
  capture is recorded below.

- Current full gate at source SHA `613090c0`: `cargo fmt --all -- --check`,
  workspace check, workspace clippy with `-D warnings`, workspace tests
  (`404 core`, `119 infrastructure`, `32 runtime`, `4 tauri`, `3`, `21`, `9`,
  `34`, `31`, `630 UI`; no failures), native release build, capture-feature
  release build, architecture guard and clean scan (`16 pass`, `0 warnings`)
  all PASS. Runtime capture is
  `/tmp/db-pro-native-core-613090c0.png` at logical `1280x800`; the New
  Connection dialog is centered with a separated header and right-aligned
  close control. The rebuilt release binary is running for manual
  verification.
- Result-grid header menu follow-up at source SHA `1f0551c1`: column context
  menu rendering now returns typed sort/filter/layout actions while the grid
  root applies them. Focused UI check, clippy, 630 UI tests, architecture guard
  and clean scan PASS; full native rebuild/runtime capture is pending for this
  follow-up.
- Current full gate at source SHA `1f0551c1`: workspace fmt/check/clippy,
  workspace tests (`404 core`, `119 infrastructure`, `32 runtime`, `4 tauri`,
  `3`, `21`, `9`, `34`, `31`, `630 UI`; no failures), native release build,
  capture-feature release build, architecture guard and clean scan (`16 pass`,
  `0 warnings`) all PASS. Runtime capture is
  `/tmp/db-pro-native-core-1f0551c1-settled.png` at logical `1280x800`; the
  New Connection dialog is centered with a separated header and right-aligned
  close control. The rebuilt release binary is running for manual
  verification.
- Result-grid header content follow-up at source SHA `f0c3a822`: PK/FK badges,
  column labels, data types and sort markers now render through an explicit
  visual context. Focused UI check, clippy, 630 UI tests, architecture guard
  and clean scan PASS; full native rebuild/runtime capture is pending for this
  follow-up.
- Current full gate at source SHA `f0c3a822`: workspace fmt/check/clippy,
  workspace tests (`404 core`, `119 infrastructure`, `32 runtime`, `4 tauri`,
  `3`, `21`, `9`, `34`, `31`, `630 UI`; no failures), native release build,
  capture-feature release build, architecture guard and clean scan (`16 pass`,
  `0 warnings`) all PASS. Runtime capture is
  `/tmp/db-pro-native-core-f0c3a822-default.png` at logical `1280x800`; the
  New Connection dialog is centered with a separated header and right-aligned
  close control. The rebuilt release binary is running for manual
  verification.
- Result-grid cell follow-up at source SHA `b69b82c0`: cell surface/value
  painting and the 17-command context menu now use explicit view contexts and
  typed menu actions. Focused UI fmt/check, clippy, 630 UI tests and
  architecture guard PASS; clean scan has only the existing size warnings for
  the remaining grid composition methods. Full native rebuild/runtime evidence
  is recorded below.
- Current full gate at source SHA `b69b82c0`: workspace fmt/check/clippy,
  workspace tests (`404 core`, `119 infrastructure`, `32 runtime`, `4 tauri`,
  `3`, `21`, `9`, `34`, `31`, `630 UI`; no failures), native release build,
  capture-feature release build, architecture guard and clean scan (`16 pass`,
  `0 warnings`) all PASS. Runtime capture is
  `/tmp/db-pro-native-core-b69b82c0.png` at logical `1280x800`; the New
  Connection dialog is centered with a separated header and right-aligned
  close control. The rebuilt release binary is running for manual
  verification.

## Core boundary checkpoint at `67965be2`

- Workspace tab intent boundary: `6a117654`, `7b6c7d0f`.
- Query completion/editor surface boundaries: `27baf92c`, `d8b7629d`.
- Result-grid header intent boundary: `67965be2`.
- Full gate at `67965be2`: format, workspace check, workspace clippy with
  `-D warnings`, workspace tests (`404`, `119`, `32`, `4`, `3`, `21`, `9`,
  `34`, `31`, `630` UI; no failures), native release build, capture build,
  architecture guard and clean-code scan (`16 pass`, `0 warnings`) all pass.
- Runtime capture: `/tmp/db-pro-native-core-67965be2.png` at logical
  `1280x800`; New Connection is centered, its header has a divider, and close
  is aligned at the right edge. Release binary from this SHA is running for
  manual verification.

## Not yet proven

- Native screenshot/runtime evidence for the requested `1440x900` and
  `1920x1080` logical heights remains host-limited: macOS capture clamps both
  to a logical height of `838`. The required normal/loading/error/empty states
  are now captured at exact logical `1280x800`.

## Core boundary checkpoint at `db6013ee`

- Table workspace surface boundary: `35776e2b`.
- Result-grid keyboard intent boundary: `8801fdce`.
- Schema-object surface boundary: `d3414e38`.
- Result-grid row-gutter surface boundary: `db6013ee`.
- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --no-fail-fast --quiet`: passed; 630 UI tests and
  all workspace suites passed, with environment-gated tests ignored.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed with 16 checks and 0 warnings.
- Runtime capture: `/tmp/db-pro-native-core-db6013ee.png`, logical `1280x800`.
  The New Connection dialog is centered, has a separated header/divider and a
  right-aligned close control. Release binary PID `49440` is running for manual
  verification.

## Final core checkpoint at `e4552773`

- Result-grid row surface boundary: `748889bb`.
- Capture stability fix: `e4552773` (default settle window raised to 60 frames).
- Full fmt/check/clippy/workspace test/release-build/capture-build gate:
  passed. Workspace suites include 630 passing UI tests; environment-gated
  tests remain ignored.
- `bash scripts/check-ui-architecture.sh`: passed.
- Clean scan: 16 pass, 0 warnings.
- Default-settle runtime capture: `/tmp/db-pro-native-core-e4552773.png`,
  logical `1280x800`. The New Connection body rendered fully; dialog is
  centered, header is separated by a divider, and close is right-aligned.
- Release binary from this code SHA is running as PID `53667` for manual
  verification.

## Current runtime checkpoint at `fc28d660`

- Files tree surface boundary: `fc28d660`.
- Full fmt/check/clippy/workspace test/release-build/capture-build gate:
  passed; workspace suites include 630 passing UI tests and only
  environment-gated tests ignored.
- `bash scripts/check-ui-architecture.sh`: passed.
- Clean scan: 16 pass, 0 warnings.
- Default-settle runtime capture: `/tmp/db-pro-native-core-fc28d660.png`,
  logical `1280x800`; the New Connection body rendered fully, with centered
  dialog, separated header/divider and right-aligned close control.
- Release binary from this code SHA is running as PID `56582` for manual
  verification.
