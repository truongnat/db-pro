# Fix — Column ALTER path (audit slice 01, P1)

Date: 2026-10-13. Implements provider-aware column ALTER (type / nullable / default)
that the audit flagged as missing on HEAD (`object_mutation_builders.rs` only had
Create/Drop/Rename for `ObjectDefinition::Column`).

## What changed

**Core (`db-pro-core`)**
- `application/ddl_builder.rs` — new `ColumnAlterDef` + three builders:
  - `build_alter_table_alter_column` (PostgreSQL): one `ALTER TABLE` combining
    `ALTER COLUMN … TYPE`, `SET|DROP NOT NULL`, `SET|DROP DEFAULT` — matching the
    existing single-statement + `validate_raw_fragment` conventions.
  - `build_alter_table_modify_column` (MySQL): `MODIFY COLUMN` restating the full
    definition (type + nullability + DEFAULT; omitted DEFAULT drops it).
  - `build_alter_table_alter_column_sqlserver` (SQL Server): `ALTER COLUMN … <type>
    NULL|NOT NULL`; default changes are rejected (defaults are named constraints).
- `application/object_mutation_builders.rs` — `(Column, Alter)` arm +
  `build_column_alter_statements` dispatching on `request.driver`; empty
  type/name → `MutationError::Validation`; SQL Server + `Some(default)` →
  `MutationError::Unsupported` with reason (never emits wrong SQL).
- `application/object_mutation_service.rs` — `(Column, Alter)` gated on SQLite in
  `sqlite_unsupported_reason` ("cannot ALTER COLUMN … without a table rebuild");
  column ALTER marked `long_running` (PG type change can rewrite the table).
- `domain/capabilities.rs` — reused the existing `schema.alter_column_type` flag /
  `CapabilityFeature::AlterColumnType` (already true for PG/MySQL/SQL Server,
  false for SQLite); doc comment broadened to cover nullability/default.
- `domain/capability_limitations.rs` — SQLite reason broadened.

**UI (`db-pro-ui`)**
- `schema_workbench.rs` — `PreviewDialect` (Ansi / backtick / bracket quoting,
  selected by `preview_dialect_for`) replaces hardcoded double-quote `QuoteDialect`
  in `run_plan`, so MySQL previews are executable under default `sql_mode`
  (removed dead `QuoteDialect`). `column_alter_limitation()` helper resolves the
  capability reason via `query_capabilities().feature_limitation(AlterColumnType)`;
  threaded into `SchemaWorkbenchSurfaceContext`.
- `schema_workbench_surface_view.rs` / `schema_workbench_form.rs` — new
  `column_alter_limitation` context field; Column mode gains a "Plan alter" button,
  disabled + hover-reason when the provider can't ALTER COLUMN.
- `table_structure_view.rs` / `table_structure_surface_view.rs` — Structure view
  gets an "Actions" column with a per-row ghost "Alter" button (disabled + reason
  tooltip when gated). `TableStructureAction::AlterColumn` →
  `DbProApp::plan_column_alter` seeds the workbench from introspected column data,
  plans `ObjectAction::Alter`, and opens the workbench — Apply still routes
  through the existing confirm dialog → `UiCommand::ExecuteDdl` (no new runtime
  path; `safety = "mutating"`, `long_running = true`).

## Tests run

- `cargo test -p db-pro-core -- application::ddl_builder application::object_mutation`
  — 25 passed. Covers PG combined TYPE/NOT NULL/DEFAULT clauses, nullable toggle,
  default set/drop, quoting + unqualified table, fragment validation; MySQL
  `MODIFY COLUMN`; SQL Server default rejection; SQLite `unsupported_reason`;
  validation errors.
- `cargo test -p db-pro-ui -- table_structure schema_workbench` — 6 passed, incl.
  new `plan_column_alter_seeds_workbench_and_plans_alter_sql` and
  `preview_dialect_quotes_identifiers_per_driver`.
- `cargo check -p db-pro-ui -p db-pro-infrastructure -p db-pro-core` — clean, 0 warnings.
- `cargo fmt --check -p db-pro-core -p db-pro-ui` — clean.

## Notes / left

- PG combined `ALTER TABLE` (comma-separated sub-actions) is the documented PG
  syntax and atomic; type changes that fail (bad cast) roll the whole statement
  back — combined mutation scope item from S1 is covered.
- MySQL `MODIFY COLUMN` requires the restated full definition; seeding comes from
  the introspected column row, so nothing is lost — but a stale introspection
  snapshot could restate outdated attributes (same caveat as every plan-preview).
- SQLite remains gated out at plan level (preview shows the reason; no SQL
  emitted). Table-rebuild ALTER is out of scope per the ticket.
- SQL Server type/nullable alter works; a `Some(default)` change returns
  `MutationError::Unsupported` with a named-constraint reason.
- Live PG/MySQL execution was not run here; correctness evidence is builder +
  service unit tests (the audit's pg/mysql live lanes exist as fixtures).
