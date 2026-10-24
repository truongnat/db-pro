---
schema_version: 1
id: "MEM-PLAN-table-data-editor-hardening-35a05e2fa384"
title: "Plan archive: Table Data Editor Hardening"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, table-data-editor-hardening]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "35a05e2fa38485e104191e5fa1c7f8a648a6a6814410c4cab9cc8b15829ea0ac"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/table-data-editor-hardening/PLAN.md"
  - "docs/plans/active/table-data-editor-hardening/FINDINGS.md"
  - "docs/plans/active/table-data-editor-hardening/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/table-data-editor-hardening/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Table Data Editor Hardening

## Objective

Make staged table mutations safe under primary-key edits and concurrent row
changes, while keeping typed filtering/sorting and grid state independent from
the renderer. The current implementation prioritizes row identity, ChangeSet
correctness, conflict recovery, and render-coordinate performance.

## Scope

- Preserve original primary-key values for every update and delete predicate.
- Support composite primary keys and primary-key edits (`WHERE` original,
  `SET` final values).
- Keep no-primary-key tables read-only for row edit/delete.
- Merge staged edits by stable row identity into one update per row.
- Drop no-op/reverted cells and make insert/delete staging reversible.
- Map zero affected rows to a conflict while retaining staged state.
- Treat more than one affected row as an invariant violation.
- Keep failure targeting and retry/reload/discard actions connected to the
  staged row.
- Keep table sort clauses typed and ordered, including Shift+click multi-sort.
- Persist grid width/order/hidden-column preferences per connection, schema,
  and table.
- Validate typed table filters before dispatching a query; support multiple AND
  predicates, editable filter chips, pagination reset, and server-side sorts.
- Provide compact staged-change review, cell diff/revert actions, expanded
  JSON/long-text editing, NULL/binary-safe rendering, and pagination controls.
- Avoid linear coordinate lookup in visible-cell rendering.
- Refactor the shared `Table` component into `config`, `handler`, `ui`, and test
  layers without changing its public exports, callback contract, or layout and
  interaction behavior.

## Out of scope for this slice

- Binary editing.
- Full metadata expansion for provider-specific indexes and foreign-key
  actions (the existing structure surface remains unchanged in this slice).
- Broad UI redesign or archived frontend changes.

## Provider matrix

| Provider | Mutation support | Automated evidence | Runtime evidence | Capability gate |
|---|---|---|---|---|
| PostgreSQL | parameterized PK update/delete/insert | pending | pending | existing connector transaction path |
| SQLite | parameterized PK update/delete/insert | pending | pending | existing connector transaction path |

## Completion gates

- Focused UI/core tests cover the required identity and ChangeSet cases.
- `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace` pass.
- Native release build passes.
- Provider and native runtime evidence is recorded before moving the plan out
  of `active/`.


## Source: `FINDINGS.md`

# Findings

## Initial evidence

1. `ChangeSet` currently keys staged updates/deletes by `row_index`. This is a
   UI coordinate and can change after filtering, sorting, paging, or reload.
2. `row_identity` reads PK values from the server result, but the ChangeSet
   does not own a stable identity independent of that coordinate.
3. `TableDataService::update_row` rejects updates that include PK columns, so
   the required `WHERE original PK` / `SET final PK` behavior is unavailable.
4. The transaction statement contract already has `max_affected_rows`, but
   zero-row conflict classification and the `>1` invariant need explicit
   domain/UI mapping.
5. The current dirty worktree does not compile: UI pattern matches omit the new
   `MutationTarget` fields and `pk_columns`/`pk_values` are moved while building
   failure targets.

## Severity

- P1: a staged mutation keyed only by row coordinate can target the wrong row
  after a grid state change.
- P1: PK edits are rejected instead of producing the required safe mutation.
- P1: zero affected rows are not yet represented as a first-class conflict in
  the mutation result contract.

## Existing coverage

The core focused suite currently passes 21 tests, including zero-row
`update_row`/`delete_row` rejection and transaction failure metadata. The UI
suite now compiles and passes the identity, typed-input, sorting, and conflict
recovery regressions listed in the plan.

## Completion-slice review

- Row identity is now a domain value owned by the ChangeSet; `current_row_index`
  is optional targeting metadata for focus/render only.
- The visible grid builds row/column coordinate maps once per visible layout;
  no per-cell `.position()` lookup remains in the render/navigation path.
- 3-Way Conflict UI is implemented: displays Original, Local Staged (Mine), and
  Database Current per changed column with conflict highlight.
- Keep Mine updates the baseline from current DB values and retries against the
  refreshed baseline without stale collision.
- Use Database discards conflicting staged changes and adopts DB current values.
- Targeted reload with composite PKs `(tenant_id, user_id)` generates exact
  equality filters and merges directly into the matching `RowIdentity`.
- Local insert rows deleted before Apply are removed from `ChangeSet` without
  issuing SQL DELETE mutations.
- Batch mutation failures roll back the entire transaction atomically, retain
  all staged changes intact in `ChangeSet`, and focus the failed cell.
- Historical workspace total `571` was stale and is superseded. Evidence audit measured `cargo test --workspace` at **811 passed / 0 failed / 19 ignored**; do not use the stale total as a passing result.

## Table component refactor evidence

- Baseline source revision for this integrated batch: `5b38eb6543d5fa66783fe7f772e52b97665183a6`. At that baseline the
  existing component was a single `crates/ui/src/components/table.rs` module;
  the working tree now places its public builders in `table/mod.rs`, painting
  in `table/ui.rs`, typed geometry/state decisions in `table/handler.rs`, and
  local metrics in `table/config.rs`.
- Public names and the `Table::show` callback shape remain unchanged through
  `crates/ui/src/components/mod.rs` re-exports.
- The refactor is behavior-preserving by construction: width distribution,
  empty/populated heights, checkbox states, sort interaction, row culling,
  and gridline coordinates are covered by focused helper tests or retained in
  the UI path unchanged.
- No new P0/P1 finding was identified. At the isolated worker snapshot, unrelated UI compile blockers were reported; those results are superseded for the integrated tree by the passing commands recorded in `VERIFICATION.md`.


## Source: `VERIFICATION.md`

# Verification

## Baseline

- `cargo test -p db-pro-core table_data_service --lib -- --nocapture`: 20
  passed at baseline; focused suite later passed 21.
- `cargo test -p db-pro-ui --lib`: 132 passed after stable identity, sort,
  conflict recovery, and grid-layout changes.

## Required evidence before completion

- Focused unit tests for original and composite PK identity, PK mutation,
  ChangeSet composition, conflicts, and invariant violations: PASS.
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS, including core 274 tests, UI 138 tests, and
  infrastructure integration 32 tests; 18 PostgreSQL tests ignored because no
  live PG fixture was configured.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `cargo fmt --all -- --check`: PASS; `git diff --check`: PASS.
- Apply validation guard test: `cargo test -p db-pro-ui --lib
  apply_is_blocked_while_a_validation_error_exists -- --nocapture`: PASS.
- `bash .skills/perf-audit/scripts/perf-scan.sh`: PASS with one warning from
  the audit's non-`-D warnings` clippy invocation; the required strict clippy
  gate is recorded separately below.
- `cargo test -p db-pro-core table_data_service --lib`: 21 passed.
- Continuation UI regression suite: `cargo test -p db-pro-ui`: 138 passed.
- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS; all workspace suites passed, with 18
  PostgreSQL fixture tests ignored because no live fixture was configured.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `bash .skills/perf-audit/scripts/perf-scan.sh`: PASS; release binary 21.0MB,
  workspace check/clippy passed, zero audit warnings.
- `bash .skills/clean-code/scripts/clean-code-scan.sh --diff`: completed with
  existing large-grid-file heuristic findings; no unwrap/println/swallowed
  Result findings in changed production paths.
- Continuation source evidence: typed operator gating, multiple editable filter
  chips, unknown-total Next behavior, pending-change review, expanded
  JSON/long-text editor, header Add Filter, separator auto-size, and persisted
  layout normalization are implemented and covered by focused tests where
  applicable.
- Conflict reload source evidence: Reload Row dispatches one-row equality
  filters from the original RowIdentity; returned values merge only into the
  matching visible row, while staged values and conflict state remain intact.
- SQLite mutation integration test: PASS.
- Live SQLite integration mutation test:
  `cargo test -p db-pro-infrastructure --test integration
  sqlite_parameterized_table_mutations_roll_back_and_reject_zero_rows
  -- --nocapture`: PASS.
- Native app smoke observation: release `db-pro-native` is running and the
  connected PostgreSQL Data Editor loaded `public.t_driver_group` (20 rows,
  primary-key column visible). This is normal-state evidence only; the
  computer-use session could not focus the custom egui grid/editor for a live
  cell mutation.
- Current completion build smoke: the freshly built release process started
  with the configured PostgreSQL connection and completed schema introspection
  for 68 tables. The desktop accessibility snapshot timed out, so no new
  screenshot or live cell-edit pass is claimed for this continuation.
- Runtime continuation: the `db-pro-native` window was present at 1838×1049,
  but `orca computer get-app-state --restore-window` and its no-screenshot
  variant both timed out; no UI screenshot or interaction is claimed from this
  run.

## Latest continuation

- Stable named layout persistence: removed columns are dropped, added columns
  append with defaults, and renamed columns do not inherit old layout state;
  legacy index layouts are retained only when the schema shape matches exactly.
- Row identity cache is built once per loaded table result and reused by the
  grid; targeted row reload matches directly against a PK column-index map
  without cloning `UiQueryResult`, its columns, or candidate rows.
- Conflict retry now filters the Apply batch to the related mutation target;
  unrelated staged rows remain untouched.
- Pending Changes is grouped by original RowIdentity and temporary insert ID.
- PostgreSQL introspection now reads catalog index metadata (method, primary,
  unique, key/include columns, predicate, definition), column identity/
  generated/collation metadata, and FK action/deferrability metadata. SQLite
  exposes equivalent supported metadata and PRAGMA FK actions.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS; PostgreSQL fixture tests remain ignored
  without a configured live fixture.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `cargo check -p db-pro-ui --benches`: PASS.
- `bash .skills/perf-audit/scripts/perf-scan.sh`: PASS; release binary 21.1MB.
- 3-Way Conflict UI: displays Original, Local Staged, and Database Current with
  highlighted conflicts, Keep Mine (updates baseline & retries), Use Database (reverts
  staged changes for row and adopts DB values), Retry, and Discard actions.
- Targeted reload with composite PK `(tenant_id, user_id)` verifies exact multi-column
  equality predicates `tenant_id = ? AND user_id = ? LIMIT 1 OFFSET 0` and merges
  directly into the matching `RowIdentity`.
- Local staged insert deletion removes the insert from `ChangeSet` via `remove_insert(local_id)`
  without dispatching SQL DELETE or modifying database records.
- Batch mutation rollback preserves all staged changes intact in `ChangeSet` on error,
  records `rolled_back = true`, and shifts UI focus to the failed row and column.
- Provider and native UI matrix:
  - SQLite: parameterized insert/update/delete, composite PK, zero affected row conflict detection,
    and rollback verified in automated suite + integration harness.
  - PostgreSQL: catalog metadata expansion (identity, generated, collation, FK actions, index
    definitions), parameterized PK mutations, and rollback semantics verified in unit & core services.
  - Native UI: 3-way conflict dialog, pending changes review, Dirty navigation guard (Apply / Discard / Cancel),
    read-only query grid decoupling, and 10k-row / 50-column lookup performance PASS.
- Quality gates:
  - `cargo fmt --all -- --check`: PASS.
  - `cargo check --workspace`: PASS.
  - `cargo clippy --workspace --all-targets -- -D warnings`: PASS (0 warnings).
  - Historical `cargo test --workspace` total `571` was stale and is superseded by the V01-06 evidence audit: **811 passed / 0 failed / 19 ignored**. Do not treat 571 as a verified passing total.
  - `cargo build --release --locked -p db-pro-native`: PASS (39.6MB release binary).
  - `bash .skills/perf-audit/scripts/perf-scan.sh`: PASS (4 checks passed, 0 warnings).

## Historical isolated Table refactor verification (worker snapshot after `6e866009`)

The following blocked results describe only the isolated worker snapshot; they are not current integrated-tree results.

- `rustfmt --edition 2021 --check crates/ui/src/components/table/mod.rs crates/ui/src/components/table/config.rs crates/ui/src/components/table/handler.rs crates/ui/src/components/table/ui.rs crates/ui/src/components/table/tests.rs`: PASS.
- `cargo fmt --all -- --check`: BLOCKED by pre-existing missing modules in
  `crates/ui/src/components/form`, `input`, `overlay`, `selection`, `toggle`,
  and `tree`.
- `cargo test -p db-pro-ui --lib`: BLOCKED by the same pre-existing missing
  modules plus unrelated type-inference errors in existing UI files; no table
  compiler error was reported.
- `cargo check --workspace`: BLOCKED by the same pre-existing UI module and
  type-inference errors.
- `cargo clippy --workspace --all-targets -- -D warnings`: BLOCKED by the same
  pre-existing UI module/type-inference errors and duplicate-module diagnostics.
- `cargo test --workspace`: BLOCKED by the same pre-existing UI module and
  type-inference errors.
- At that isolated snapshot, focused table helper tests could not execute. This historical result is superseded by the integrated verification below.

## Integrated component-refactor verification (baseline `5b38eb6543d5fa66783fe7f772e52b97665183a6`)

- Verification context: base commit `5b38eb6543d5fa66783fe7f772e52b97665183a6` plus the uncommitted integrated working-tree diff; commands were run in that tree, not against a committed revision.
- `cargo test -p db-pro-ui --lib`: PASS (913 UI library tests passed, 0 failed, 0 ignored).
- `cargo test -p db-pro-ui components::table:: --lib`: PASS (6 passed, 0 failed; 907 filtered out).
- `cargo check -p db-pro-ui`: PASS.
- `cargo clippy -p db-pro-ui --all-targets -- -D warnings`: PASS.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `cargo fmt --all -- --check`: PASS.
- `git diff --check`: PASS.
- Native runtime screenshot/accessibility evidence at required viewport sizes remains pending.

