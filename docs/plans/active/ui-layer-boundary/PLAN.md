# UI Layer Boundary Enforcement

## Goal

After the 2026-10-14 feature-folder reorg (`crates/ui/src` now has 28 feature
folders + `lib.rs` only), `scripts/check-ui-architecture.sh` still references
the old flat layout (`src/app.rs`, `src/events.rs`, …) and fails at its first
check — the CI gate is currently dead. This plan (a) repairs every path, and
(b) adds the new layer rule the reorg enables: the pure widget layer must not
import feature/app modules.

## Context

- Script: `scripts/check-ui-architecture.sh` (~330 lines). It mixes three
  concerns: composition-root guards (`app.rs`/`DbProApp`), event-boundary
  guards (events, reducers, task_bridge), and state-boundary guards
  (explicit-state allowlist + `*_state.rs` DbProApp bans + UiCommand bans).
- New layout (post-reorg):
  - Composition root: `app/mod.rs` (was `app.rs`), `app/app_modules.rs`,
    `app/app_{state,types,lifecycle,storage,tests}.rs`.
  - Runtime/event plumbing: `runtime/` (events.rs, event_router.rs,
    operation_events.rs, task_bridge.rs, runtime_*).
  - Feature folders: agent, chart, component_gallery, connection, dev_tools,
    diagram, diagram_surface, editor, explorer, files, monitoring, palette,
    query, query_workbench, result_grid, schema, security, settings, shell,
    table, tokens, tools, visual_query_builder, workspace.
  - Widget layer (must stay feature-agnostic): `components/`, `theme/`,
    `tokens/` (inside theme), `editor/` (pure text engine), `diagram/` (pure
    graph renderer).

## Tasks

### 1. Repair broken paths in `scripts/check-ui-architecture.sh`

Replace old paths:

| Old (flat) | New |
|---|---|
| `src/app.rs` | `src/app/mod.rs` |
| `src/events.rs` | `src/runtime/events.rs` |
| `src/event_router.rs` | `src/runtime/event_router.rs` |
| `src/tasks_view.rs` | `src/tools/tasks_view.rs` |
| `src/activity_bar_view.rs` | `src/shell/activity_bar_view.rs` |
| `src/database_operations_state.rs` | `src/schema/database_operations_state.rs` (still must stay deleted — keep existence check on new path AND old path) |
| `src/table_state.rs` | `src/table/table_state.rs` |
| `src/table_data_query_state.rs` | `src/table/table_data_query_state.rs` |
| `src/query_library_state.rs` | `src/query_workbench/query_library_state.rs` |
| `src/schema_workbench.rs` | `src/schema/schema_workbench.rs` |
| `src/explorer_navigation.rs` | `src/explorer/explorer_navigation.rs` |
| `src/query_execution_actions.rs` | `src/query_workbench/query_execution_actions.rs` |
| `src/query_save_actions.rs` | `src/query_workbench/query_save_actions.rs` |
| `src/app_state.rs` | `src/app/app_state.rs` |
| `src/connection/*` | `src/connection/*` (unchanged) |

### 2. Rewrite the `explicit_state_modules` allowlist

Every entry is a moved file. Rewrite the list as folder globs instead of ~170
explicit files — the intent is "view/feature helpers must take explicit
state/context, not `DbProApp`". The allowlist semantics must be preserved:
`#[cfg(test)]` blocks are exempt. Preferred: iterate over the *same* set of
files by globbing `src/{agent,chart,component_gallery,connection,dev_tools,
diagram,diagram_surface,explorer,files,monitoring,palette,query,query_workbench,
result_grid,schema,security,settings,shell,table,tools,visual_query_builder,
workspace}/` for `*.rs` while excluding `*_tests.rs` and `mod.rs` composition
files. Document the rule in a comment.

### 3. Rewrite per-file loops

- `for state_module in src/*_state.rs` → recurse `src/**/*_state.rs` (keep the
  `app/app_state.rs` exemption).
- `for reducer in src/{agent,connection,ddl,file_picker,management,schema,
  table,query_*}_events.rs` → glob `src/**/*_events.rs` minus
  `src/runtime/*` (runtime events are the dispatcher, not reducers).
- `*_state.rs` UiCommand bans: expand to every `src/**/*_state.rs` except
  `app/app_state.rs` — same semantic, broader coverage.
- `pub(in crate::app)` leak check: `src/connection/state.rs` stays.
- Event-module existence check: modules moved to `runtime/` — check
  `src/runtime/{events,event_router,operation_events}.rs` + glob feature
  `*_events.rs`.

### 4. Fix the raw-widgets exclusion list

Current exclusions (`/components/|/dev_tools/|_tests.rs:|app_tests.rs:`) —
`app_tests.rs:` moved to `app/tests/app_tests.rs`; keep name match (still
hits) but also exclude `*/tests/` dirs generally.

### 5. NEW: widget-layer boundary rule (the ask)

Add a check: files under `src/components/`, `src/theme/`, `src/tokens/` (dir),
`src/editor/`, `src/diagram/` must not import feature/app state:

```
rg -n 'use crate::(app|agent|query_workbench|table|schema|explorer|files|shell|
       tools|settings|security|monitoring|palette|workspace|visual_query_builder|
       diagram_surface|component_gallery)::' on widget-layer dirs
```

Allowed deps for widget layer: `crate::{components,tokens,theme,i18n,policy,
runtime (Ui* protocol types only),editor,diagram,result_grid,chart,DbProTheme}`.
Implement as a denylist of feature module names — the allowlist approach is
brittle because `crate::{DbProTheme, components::*}` are legitimately used.

### 6. Keep gate semantics

- Script must exit non-zero on first failure with the same message style.
- Run: `bash scripts/check-ui-architecture.sh` → PASS locally.
- Update `AGENTS.md` "Rust quality gates" note: fmt is now
  `cargo +nightly fmt --all -- --check`.

### 7. Commit slices

One commit per logical change:
1. fix(script): repoint architecture checks at feature-folder layout
2. feat(script): enforce widget-layer import boundary
3. docs: quality-gate fmt on nightly

## Acceptance

- `bash scripts/check-ui-architecture.sh` prints `PASS` on clean tree.
- Deliberately adding `use crate::app::…` inside `src/components/` makes the
  script fail.
- Deliberately adding `use crate::components::…` inside `src/editor/` passes
  (widget↔widget ok).
- `cargo check --workspace` green; no .rs files modified except the script.
