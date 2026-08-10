# S3 — Schema Relations Runtime Findings

## P1-001 — Composite FK DDL reconstruction emitted duplicate constraints

Status: FIXED IN SOURCE; CI PASS (tests executed and passed in CI run 31412787979).

Evidence:
- provider introspection represents a composite FK as multiple column-mapping rows,
- PostgreSQL rows share the real constraint name,
- `build_create_table_ddl()` previously emitted one `ADD CONSTRAINT` per row.

Failure scenario:
A composite FK `(tenant_id, parent_id) REFERENCES parent(tenant_id, id)` was reconstructed as two separate constraints with the same constraint name. Executing that DDL would be invalid and would not preserve the original relation semantics.

Fix:
Group mapping rows by constraint identity and target before emitting DDL; join ordered source/target columns into one `FOREIGN KEY (...) REFERENCES ... (...)` clause.

## P1-002 — SQLite composite FK mappings had different synthetic identities

Status: FIXED IN SOURCE; CI PASS (tests executed and passed in CI run 31412787979).

Evidence:
SQLite `PRAGMA foreign_key_list` returns `id` and `seq`, where all mappings for one composite constraint share `id`. Previous code named each mapping from `from_column`, splitting one constraint into multiple apparent relations.

Fix:
Use `{table_name}_fk_{id}` as the shared synthetic identity (e.g., `child_fk_0` for the first FK on the `child` table). Preserve PRAGMA row order for the mappings.

## P2-001 — Foreign Keys UI rendered mapping rows rather than constraints

Status: FIXED IN SOURCE; CI PASS (frontend tests executed and passed in CI).

Evidence:
`ForeignKeyList` mapped directly over scalar FK rows and used `fk.name` as the React key. PostgreSQL composite FK rows therefore produced duplicate keys and duplicate visual relation rows.

Fix:
`groupForeignKeys()` groups mappings by relation identity. The UI renders ordered source/target column lists once per constraint.

## P2-002 — ER diagram still treats scalar FK mappings as separate edges

Status: DEFERRED TO S6 ER DIAGRAM.

The current S3 scope fixes relation truth in introspection, DDL reconstruction and the Foreign Keys table. ER visualization owns a separate presentation model and will be normalized during S6 rather than expanding S3 scope.

## Runtime gaps

- PostgreSQL composite FK provider verification: PENDING (requires live PG credentials)
- SQLite runtime test: CI PASS
- UI relation-list/navigation runtime verification: PENDING (requires running app)

## Gate

All source-level P1 fixes validated by CI. S3 is ready for RUNTIME_VERIFY once PostgreSQL credentials and UI runtime evidence are available.

## VPS Kilo Review #1 Triage (commit b7e9c8d)

### P1-003 — FK enforcement test did not enable PRAGMA foreign_keys
- **Classification**: CONFIRMED
- **Severity**: P1 (test proving nothing without FK enforcement enabled)
- **Fix**: Added `PRAGMA foreign_keys = ON` before table creation + readback assertion to verify pragma took effect
- **Commit**: f2a2805

### P2 — S1 atomicity test only covers SQLite
- **Classification**: CONFIRMED, ACCEPTED as known gap
- **Action**: PostgreSQL rollback coverage belongs to S1 RUNTIME_VERIFY, not S3 scope

### P2 — Frontend groupForeignKeys test coverage gaps
- **Classification**: CONFIRMED, ACCEPTED as known gap
- **Action**: Single-column FK grouping is trivially correct; no regression risk

### P2 — S3 plan has pending runtime evidence
- **Classification**: CONFIRMED, TRACKED
- **Action**: Documented in VERIFICATION.md

### Rejected patterns (correct as-is)
- SQLite FK synthetic name `{table}_fk_{id}`: correct, uses PRAGMA `id`
- `group_foreign_keys_for_ddl` omits `from_table`: safe, TableInfo already filtered
- `\^@` separator in frontend key: safe, SQL identifiers cannot contain null bytes
- DDL `ALTER TABLE ADD CONSTRAINT` on SQLite: viewer-only, not executed

## Cubic Review Triage

### P3 — FINDINGS.md doc accuracy (P1-002 format)
- **Classification**: CONFIRMED, FIXED in d1d27d7

### P3 — Duplicated test setup helper
- **Classification**: CONFIRMED, DEFERRED (polish, not merge blocker)

### P3 — VPS review prompt `${BASE_REF}` placeholder
- **Classification**: CONFIRMED, FIXED in ea3312f
