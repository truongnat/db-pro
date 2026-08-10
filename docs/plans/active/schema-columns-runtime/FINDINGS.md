# S1 — Schema Columns Runtime Findings

## Resolved source findings

### P1-001 — Multi-statement schema mutation was non-atomic
Status: FIXED in `741a18d`.

Combined column mutations previously executed statements sequentially. The implementation now routes the batch through connector transaction support.

### P1-002 — Schema cache invalidation was incomplete
Status: FIXED in `741a18d`.

The implementation now invalidates introspection, table info, table DDL, dependencies and the frontend schema catalog.

## Open evidence gaps

### P2-001 — Atomic rollback regression evidence is not currently preserved
Status: OPEN.

A transaction implementation exists, but S1 does not currently have preserved automated evidence proving that statement 1 is absent when statement 2 fails. Restore a focused regression test before completion.

### P2-002 — Provider runtime matrix is incomplete
Status: OPEN.

PostgreSQL and SQLite live behavior has not been independently recorded for the required mutation matrix. Source support is not sufficient evidence.

### P2-003 — Cache/UI runtime proof is incomplete
Status: OPEN.

The invalidation paths exist in source, but the DDL/tableInfo/dependency surfaces have not been recorded as runtime evidence after a real mutation.

## P0/P1 gate

No currently accepted open P0/P1 finding is recorded in this plan. Completion is still blocked by runtime evidence gaps above.
