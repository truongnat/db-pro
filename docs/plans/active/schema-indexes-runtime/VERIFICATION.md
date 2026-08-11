# S2 — Schema Indexes Runtime Verification

State: RUNTIME_VERIFY

## Automated evidence recorded by merged S2 work

Command recorded in the S2 implementation:

```text
cargo test --test schema_indexes_runtime_verification
```

Recorded result:

```text
running 1 test
test verify_create_and_drop_index ... ok

test result: ok. 1 passed; 0 failed
```

The test uses SQLite and proves:

1. `CREATE UNIQUE INDEX` is visible through SQLite introspection and marked unique.
2. a composite `CREATE INDEX` preserves ordered columns.
3. `DROP INDEX` removes the target index while leaving the other index intact.

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | PENDING | PENDING | source implementation exists; no provider runtime result recorded |
| SQLite | PASS | PARTIAL | backend integration proven; UI round-trip still pending |

## UI lifecycle

```text
UI create/drop
→ DDL service
→ database
→ cache invalidation
→ introspection
→ refreshed index list
```

Status: PENDING runtime evidence.

## Completion decision

Not completed. S2 has valid SQLite automated evidence but still requires PostgreSQL provider evidence and the user-facing refresh lifecycle before moving to `docs/plans/completed/`.
