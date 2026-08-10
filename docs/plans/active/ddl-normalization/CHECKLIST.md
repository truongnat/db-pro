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
- [ ] DDL editor trigger toggle preview test

## Provider/runtime evidence
- [x] CI: Rust checks PASS, Frontend checks PASS
- [ ] View DDL verified for PostgreSQL syntax
- [ ] View DDL verified for SQLite syntax

## Review gate
- [ ] P0 = 0
- [ ] P1 = 0
- [ ] quality gates actually executed (CI passing)
- [ ] VERIFICATION.md updated with observed evidence
