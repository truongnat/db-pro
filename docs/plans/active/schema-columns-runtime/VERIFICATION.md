# S1 — Schema Columns Runtime Verification

State: RUNTIME_VERIFY

## Existing evidence

Source inspection records that the column mutation path is designed as:

```text
UI edit
→ classify mutation
→ execute DDL batch
→ connector transaction
→ cache invalidation
→ query refetch
```

Historical test evidence recorded in the previous plan:
- `cargo check --workspace` — PASS at the time recorded
- frontend Vitest — 1291 tests / 103 files PASS at the time recorded

These historical results are retained as historical evidence only; they are not a claim about the current branch.

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | PARTIAL/source | PENDING | combined mutation + failure rollback still required |
| SQLite | PARTIAL/source | PENDING | supported vs capability-gated ALTER paths still required |

## Required live/runtime evidence

- [ ] PostgreSQL: rename + type + nullable combined → all requested changes applied
- [ ] PostgreSQL: failing/narrowing change → no partial mutation persisted
- [ ] SQLite: supported mutation operations succeed
- [ ] SQLite: unsupported operations are blocked by capability rules rather than invalid SQL
- [ ] DDL viewer reflects changed table definition
- [ ] tableInfo and dependencies refresh after mutation

## Completion decision

Not completed. The previous completed classification was inconsistent with the still-pending runtime matrix, so S1 is reopened as `RUNTIME_VERIFY`.
