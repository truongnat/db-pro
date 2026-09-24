# Schema Compare Baseline

> Status: source analysis; no implementation or runtime verification performed.
>
> Source baseline: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`  
> Research date: 2026-09-24

## Scope

This document analyzes the native **Schema Compare activity/workspace**, including its in-memory schema snapshot, structural diff, migration preview/apply path, and the separate key-aware cross-connection row comparison embedded in the same view. The two comparison modes have different data paths and safety boundaries; “Data Compare” here does not execute its displayed sync preview.

## Evidence boundary

Source claims below refer to exact SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`. Line anchors are at that source baseline. They establish code paths, not rendered behavior, runtime/provider behavior, or test success. Official product docs are cited separately in the feature research report.

## Surface and dispatch

- The activity rail labels `Activity::Compare` “Schema compare”; its action sets `Activity::Compare`, `WorkspaceTab::SchemaCompare`, and opens the sidebar (`crates/ui/src/activity_bar_view.rs:85-99`; `crates/ui/src/app_lifecycle.rs:218-222`). Quick Open also activates the same activity and tab (`crates/ui/src/palette_actions.rs:13-18`; `app_tests.rs:2632-2653`).
- `SchemaCompareState` is held in `SchemaWorkspaceState` alongside Explorer and Schema Workbench state (`crates/ui/src/schema_workspace_state.rs:6-12`). The sidebar and central tab both build `SchemaCompareViewContext` from the same compare state and current Explorer schema (`sidebar_view.rs:82-98`; `workspace_view.rs:110-127`).
- Sidebar provides Take snapshot and Diff vs snapshot, then opens the central tab after the diff action. The central workspace combines structural diff, migration plan, and data compare controls (`schema_compare_view.rs:20-61,64-105,171-347`).

### Structural compare and migration pipeline

```text
Current Explorer schema summary
  → Take snapshot (one UiSchemaSnapshot held in SchemaCompareState)
  → Diff now / Diff vs snapshot (snapshot is source; current summary is target)
  → diff_snapshots → UiSchemaDiffResult
  → to_core_schema_diff → MigrationPlanner::plan_from_schema_diff
  → SQL preview + risk/warnings + explicit destructive checkbox
  → prepare_migration_sql → UiCommand::ExecuteDdl on active connection
  → DDL completion / schema re-introspection
```

`MigrationPlanner::plan_from_schema_diff` describes its direction as moving the **target toward the source** (`crates/core/src/application/migration_planner.rs:11-16`). The UI passes the saved snapshot as source and the current summary as target (`schema_compare_state.rs:61-88`). Thus applying a plan attempts to bring the current schema toward the captured snapshot. The current active connection id is the DDL target at apply time (`query_session.rs:59-82`); the snapshot stores a display label containing connection name/time but no connection id (`schema_compare.rs:5-15`; `schema_compare_state.rs:50-58`).

### Cross-connection data compare pipeline

```text
Active connection id + manually entered target id/schema/table/key columns
  → prepare_data_diff_request (sample limit 1,000)
  → UiCommand::DiffTableDataKeyed
  → DataDiffService reads both active connections and compares rows by key
  → DataDiffLoaded
  → result/filter/sample summary + comment-only sync SQL preview
```

The action reads the active connection as source and sends the user-entered target id, schema, table, key columns, and 1,000-row sample limit (`workspace_actions.rs:17-39`; `schema_compare_state.rs:91-117`). The service independently counts each whole table, queries each side ordered by the supplied keys with the sample limit, rejects duplicate keys within a sample, retains at most 200 row-difference examples, and emits at most 50 comment previews (`crates/core/src/application/data_diff.rs:37-46,62-70,91-105,109-119,182-199,238-272`). The UI explicitly labels sync SQL preview as preview-only; it does not dispatch a data mutation (`schema_compare_view.rs:262-345`).

## Current comparison behavior

### Structural snapshot/diff

- One optional `UiSchemaSnapshot` is held in state. It captures schema names, full `UiTableSummary` values, view `(schema,name)` identities, and routine `(schema,name,type)` identities (`schema_compare.rs:5-15,28-45`; `schema_compare_state.rs:15-28,30-45,50-59`). Source does not show this snapshot as a durable saved artifact.
- Table presence is compared by qualified table name. For matching tables, the UI diff checks column name presence, data type, and nullability. Views and routines are compared by identity only (`schema_compare.rs:48-117`). Primary-key flags and FK definitions present in captured table summaries are not compared here. Column order, defaults, constraints, indexes, triggers, view/routine definitions, and schema-name additions/removals are also not part of this UI diff.
- The central view groups table/view/routine additions and removals and column/type changes. It displays a total count, a zero-diff “Schemas are identical” state, and then a migration plan section (`schema_compare_view.rs:103-173`). The “identical” result means identical only across those compared properties, not full schema equivalence.

### Migration plan and apply

- `to_core_schema_diff` maps table additions/removals, column additions/removals, and type mismatches, but hardcodes both index-diff lists empty; it does not map column-nullability changes, view/routine differences, or other UI-detected properties (`schema_compare.rs:120-174`).
- The core planner includes operations for tables, columns, type changes, and indexes. Its CREATE TABLE SQL contains only the placeholder `id INTEGER PRIMARY KEY`; ADD COLUMN uses `TEXT /* type from source */`; CREATE INDEX has placeholder columns. The reachable CREATE TABLE/ADD COLUMN placeholder operations are marked provider-supported; planner warnings say source introspection must expand them. CREATE INDEX is likewise placeholder-based but the UI adapter supplies empty index diffs, so this path is not currently reachable from the UI (`crates/core/src/application/migration_planner.rs:112-151,167-195,197-217`; `crates/ui/src/state/schema_compare.rs:120-174`).
- The UI shows operation warnings and SQL preview, then exposes “Apply migration SQL” after preview. `prepare_migration_sql` checks destructive confirmation and filters unsupported operations, but does not reject placeholder SQL or incomplete warnings (`schema_compare_view.rs:173-258`; `schema_compare_state.rs:119-140`). Apply sends the SQL in one `UiCommand::ExecuteDdl` for the active connection (`query_session.rs:59-82`).
- Plan fingerprint verification compares the stored plan fingerprint against the fingerprint saved from that same plan at planning time (`schema_compare_state.rs:75-88,119-140`; `MigrationPlanner::verify_fingerprint` in `migration_planner.rs:49-51`). The source does not re-introspect and compare the target database schema at apply time.
- Destructive operations require an explicit checkbox; unchecked plans use non-destructive SQL. There is no per-operation include/exclude control in this view (`schema_compare_view.rs:245-258`; `schema_compare_state.rs:119-140`).
- The plan records the preview driver (`crates/core/src/application/migration_planner.rs:102-108`), but apply does not compare it to the current active driver or bind the target connection identity. The apply handler selects whichever connection is active when the button is pressed (`schema_compare_state.rs:75-88`; `query_session.rs:59-82`). Compare state is separate from Explorer state, which is reset on connection switch (`schema_workspace_state.rs:6-12`; `explorer_navigation.rs:209-223`).
- The central view returns early when no structural diff exists, before drawing the Data Compare section (`schema_compare_view.rs:89-101,260-284`); row comparison is unavailable until a schema diff has been created.
- `take_snapshot` replaces the snapshot and updates feedback but does not clear an existing diff, migration plan, preview SQL, or destructive confirmation. By contrast, `diff_against_snapshot` clears the plan state (`schema_compare_state.rs:50-72`).
- On DDL completion, the generic DDL path refreshes schema, but it does not clear or mark the compare diff/plan stale (`ddl_events.rs:13-34`; `operation_events.rs:187-205`; `schema_compare_state.rs:61-88`). The old preview remains available after successful apply.

### Data compare result

- The comparison requires non-empty target id, table and key-column list; keys are comma-split and trimmed; schema defaults to `public`; sample limit is fixed at 1,000 in the UI (`schema_compare_state.rs:30-45,91-117`). The data service returns a validation error if a supplied key column is missing or duplicate in the sampled rows (`data_diff.rs:238-272`).
- `DataDiff` indicates full row counts, sampled added/removed/changed/equal counts, a `truncated` flag, row samples and sync-preview comments (`data_diff.rs:109-199`). UI renders row state and key only; it does not render the included `column_changes` values (`schema_compare_view.rs:285-343`).
- `DataDiffLoaded` carries a request id, but the UI event router discards it and the reducer stores every returned diff unconditionally (`runtime_protocol.rs:550-553`; `event_router.rs:66-67`; `operation_events.rs:170-172`; `management_events.rs:139-146`). Multiple outstanding compares can therefore let an earlier response overwrite a newer result.

## Source-observed correctness and safety risks

### Migration apply can execute placeholder DDL

A schema diff that reports a table only in the snapshot maps to a core `CreateTable` operation whose SQL is a minimal placeholder, explicitly warning that columns must be expanded. The operation is still marked supported; the UI offers Apply; and `prepare_migration_sql` does not reject it. For example, the generated table body can be `id INTEGER PRIMARY KEY` rather than the captured table's real columns. ADD COLUMN also carries a placeholder type. The core CREATE INDEX branch has placeholder definitions, but the current UI adapter hardcodes index diffs to empty, so that branch is not reached from this UI path. Applying reachable placeholder DDL can mutate the active database into an incomplete schema while presenting it as the migration plan. Severity: **P1** — wrong database mutation. Source evidence: `schema_compare.rs:120-174`; `migration_planner.rs:112-151,167-195,197-217`; `schema_compare_view.rs:173-258`; `schema_compare_state.rs:119-140`; `query_session.rs:59-82`. This is a source-derived risk, not a runtime reproduction.

### Preview can be applied to a different active provider/connection

The migration plan stores a driver, but `prepare_migration_sql` does not verify it against the active driver. `apply_migration_preview` sends the SQL to the current active connection id. Connection switching resets Explorer state but does not reset the separate `SchemaCompareState`, so the old plan remains available (`schema_workspace_state.rs:6-12`; `explorer_navigation.rs:209-223`; `schema_compare_state.rs:75-88,119-140`; `query_session.rs:59-82`). A plan generated for one provider/target can therefore be submitted to another active connection. Severity: **P1** — wrong database/provider mutation risk.

### Replacing a snapshot leaves the previous migration plan active

Taking a new snapshot overwrites `schema_snapshot` but does not invalidate `schema_diff` or `migration_plan`; the central Apply control is based on the retained migration preview. Until the user diffs again, a plan built from the previous snapshot can still be applied. Severity: **P2** — stale plan/provenance mismatch. Source: `schema_compare_state.rs:50-72`; `schema_compare_view.rs:224-258`.

### Structural diff can say “identical” while omitted schema properties differ

The comparison inspects only names/types/nullability for table columns and names for views/routines. It does not compare captured PK/FK metadata or other schema properties. The migration adapter additionally drops nullability changes and view/routine deltas and supplies empty index diffs. Thus an “identical” message or a migration plan can omit meaningful drift. Severity: **P1** when the result is treated as complete migration evidence; otherwise **P2** for incomplete inspection. Source: `schema_compare.rs:48-117,120-174`.

### Data Compare response can be stale

The event includes `request_id`, but that identifier is ignored by UI reduction. If multiple comparisons are in flight, completion order—not latest user request—determines the displayed result. Severity: **P2**. Source: `runtime_protocol.rs:550-553`; `event_router.rs:66-67`; `management_events.rs:139-146`.

### Migration preview remains applyable after successful execution

Successful DDL completion refreshes schema but leaves `schema_diff`, `migration_plan`, preview SQL, and confirmation state unchanged. The Apply button remains available against an old plan and can resubmit it. Severity: **P2** — stale migration action after successful schema mutation. Source: `ddl_events.rs:13-34`; `operation_events.rs:187-205`; `schema_compare_state.rs:61-88`; `schema_compare_view.rs:224-258`.

## Test coverage observed in source

Source includes tests for a basic added-table/type-change diff (`schema_compare.rs:184-236`), safe defaults, target/key input validation and migration requiring a plan (`schema_compare_state.rs:144-199`), plus core planner ordering, SQLite type-alter capability and drop filtering (`migration_planner.rs:339-393`). These are source-visible tests only; none were run for this analysis. They do not demonstrate provider runtime correctness, complete metadata diffing, placeholder rejection, live-schema staleness protection, or out-of-order DataDiff response handling.

## Evidence status

Source analysis only at SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`. No build/test, native UI traversal, PostgreSQL runtime, or SQLite runtime was performed.