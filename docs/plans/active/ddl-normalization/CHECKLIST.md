# S5 — DDL Normalization Checklist

## Analysis
- [x] DDL architecture audited (backend + frontend)
- [x] View DDL gap identified (get_table_ddl only handles tables)
- [x] DDL editor operation coverage documented
- [x] Provider DDL differences understood

## Implementation
- [x] View DDL support in get_table_ddl (falls back to view definition)
- [x] View DDL test added
- [x] Trigger enable/disable added to frontend DDL builder (buildSetTriggerEnabled)
- [x] supportsTriggerToggle capability flag added
- [x] Add trigger enable/disable to DDL editor operation type
- [x] Add DDL editor test for trigger toggle preview

## Frontend
- [x] DDL viewer already handles view DDL (same component)
- [x] DDL editor has capability checks for unsupported operations
- [x] DDL editor type selector includes trigger toggle option

## Regression tests
- [x] View DDL returns definition from introspection
- [x] Table DDL includes triggers
- [x] format_trigger_ddl handles both SQLite and PostgreSQL paths
- [x] DDL editor trigger toggle preview test

## Provider/runtime evidence
- [x] CI: Rust checks PASS, Frontend checks PASS
- [x] View DDL verified for PostgreSQL syntax (pg_introspect_views PASS in CI)
- [x] View DDL verified for SQLite syntax (introspect_views PASS in SQLite integration tests)

## Review gate
- [x] P0 = 0
- [x] P1 = 0
- [x] quality gates actually executed (CI passing)
- [x] VERIFICATION.md updated with observed evidence

## Cubic review fixes
- [x] CR1: PG trigger function body — added `function_def` to Trigger + PG introspection + DDL formatter
- [x] CR2: SQLite BEGIN parsing — quote-aware header search + 5 unit tests
- [x] CR3: PG DDL ordering — CREATE FUNCTION emitted before CREATE TRIGGER
- [x] CR4: SQLite trigger schema — changed from empty to "main"
- [x] CR5: SQLite trigger toggle — returns empty for SQLite dialect
