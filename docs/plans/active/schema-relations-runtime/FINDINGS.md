# S3 — Schema Relations Runtime Findings

## P1-001 — Composite FK DDL reconstruction emitted duplicate constraints

Status: FIXED IN SOURCE; TEST EXECUTION PENDING.

Evidence:
- provider introspection represents a composite FK as multiple column-mapping rows,
- PostgreSQL rows share the real constraint name,
- `build_create_table_ddl()` previously emitted one `ADD CONSTRAINT` per row.

Failure scenario:
A composite FK `(tenant_id, parent_id) REFERENCES parent(tenant_id, id)` was reconstructed as two separate constraints with the same constraint name. Executing that DDL would be invalid and would not preserve the original relation semantics.

Fix:
Group mapping rows by constraint identity and target before emitting DDL; join ordered source/target columns into one `FOREIGN KEY (...) REFERENCES ... (...)` clause.

## P1-002 — SQLite composite FK mappings had different synthetic identities

Status: FIXED IN SOURCE; RUNTIME TEST EXECUTION PENDING.

Evidence:
SQLite `PRAGMA foreign_key_list` returns `id` and `seq`, where all mappings for one composite constraint share `id`. Previous code named each mapping from `from_column`, splitting one constraint into multiple apparent relations.

Fix:
Use `table_fk_<id>` as the shared synthetic identity. Preserve PRAGMA row order for the mappings.

## P2-001 — Foreign Keys UI rendered mapping rows rather than constraints

Status: FIXED IN SOURCE; FRONTEND TEST EXECUTION PENDING.

Evidence:
`ForeignKeyList` mapped directly over scalar FK rows and used `fk.name` as the React key. PostgreSQL composite FK rows therefore produced duplicate keys and duplicate visual relation rows.

Fix:
`groupForeignKeys()` groups mappings by relation identity. The UI renders ordered source/target column lists once per constraint.

## P2-002 — ER diagram still treats scalar FK mappings as separate edges

Status: DEFERRED TO S6 ER DIAGRAM.

The current S3 scope fixes relation truth in introspection, DDL reconstruction and the Foreign Keys table. ER visualization owns a separate presentation model and will be normalized during S6 rather than expanding S3 scope.

## Runtime gaps

- PostgreSQL composite FK provider verification: PENDING
- SQLite newly added runtime test execution: PENDING
- UI relation-list/navigation runtime verification: PENDING

## Gate

Accepted source-level P1 defects have fixes on this branch, but S3 must not claim P1=0 for merge readiness until tests/review validate those fixes.
