# S5 — DDL Normalization Verification

State: REVIEW (All Cubic review findings resolved; CI passing)

## CI Evidence

| Check | Run | Result |
|---|---|---|
| Rust checks (cargo fmt + clippy + build + test) | [31429564670](https://github.com/truongnat/db-pro/actions/runs/31429564670) | PASS |
| Frontend checks (tsc + eslint + prettier + build + test) | [31429564670](https://github.com/truongnat/db-pro/actions/runs/31429564670) | PASS |
| Frontend typecheck (local) | tsc --noEmit | 0 errors |
| Frontend tests (local) | vitest run | 105 files, 1318 tests PASS |
| PR #8 mergeable | MERGEABLE | — |

## Changes

### View DDL (FIXED)
- `get_table_ddl` now checks `introspect.views` before failing
- Returns view definition directly from introspection data
- Unit test added: `get_table_ddl_view`

### DDL Editor Trigger Operations (DONE)
- `enableTrigger`/`disableTrigger` added to `DdlOperation` type
- Trigger name input field in DDL editor form
- Trigger operations in DDL type selector
- Capability check: SQLite shows unsupported warning
- Translation keys (en + ja)
- 6 new tests (3 builder + 3 capability + 3 preview)

### Cubic P2 Fixes

#### CR1: PostgreSQL trigger function body (FIXED)
- Added `function_def` field to `Trigger` domain struct (with `#[serde(default)]`)
- PG introspection SQL extended: `LEFT JOIN pg_proc ON pg_proc.oid = pg_t.tgfoid` + `pg_get_functiondef(pg_proc.oid)`
- `format_trigger_ddl` appends function definition after CREATE TRIGGER when present
- `TriggerDto` (Rust) + `TriggerDto` (TypeScript) updated
- New test: `format_trigger_ddl_includes_function_def_when_present`

#### CR2: SQLite BEGIN parsing (FIXED)
- Extracted `find_trigger_header()` helper with quote-aware search
- Walks forward past double-quoted identifiers to avoid matching BEGIN inside names
- 5 new unit tests: standard, BEFORE, INSTEAD OF, unquoted BEGIN-in-name, quoted BEGIN-in-name

#### CR3: PG DDL ordering (FIXED)
- `format_trigger_ddl` now emits `function_def` BEFORE `CREATE TRIGGER` statement
- Ensures DDL replay succeeds (function exists before trigger references it)
- Added ordering assertion in test

#### CR4: SQLite trigger schema (FIXED)
- Changed `schema: String::new()` → `schema: "main".into()` in SQLite trigger introspection
- `get_table_ddl` filter now correctly includes SQLite triggers

#### CR5: SQLite trigger toggle (FIXED)
- `buildSetTriggerEnabled` returns empty string for SQLite dialect
- `ALTER TABLE ENABLE/DISABLE TRIGGER` is PostgreSQL-only

### Provider Matrix

| Provider | View DDL | Trigger Toggle | DDL Editor | Function Def |
|---|---|---|---|---|
| PostgreSQL | DONE | DONE (ALTER TABLE ENABLE/DISABLE TRIGGER) | DONE (with preview) | DONE (pg_get_functiondef) |
| SQLite | DONE | N/A (capability-gated) | DONE (with warning) | N/A (full SQL from sqlite_master) |
