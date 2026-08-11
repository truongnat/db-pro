# S1 — Schema Columns Runtime Verification

State: RUNTIME_VERIFY
Baseline: `fb91c4e`
Implementation commit: `741a18d`

## Goal

Column mutation must satisfy:

```text
UI draft
→ deterministic SQL preview
→ risk classification
→ atomic execution
→ database truth
→ fresh introspection
→ all UI surfaces synchronized
```

## Scope

- rename
- data type change (widening/narrowing)
- nullable toggle
- default value (literal/expression)
- combined mutations
- rollback on failure
- cache invalidation: introspect + tableInfo + tableDdl + dependencies + catalog
- PostgreSQL/SQLite capability behavior

## Non-goals

- indexes
- relations
- triggers
- ER diagram

## Implemented source contract

- `execute_batch()` exists on `DbConnector`
- PostgreSQL batch path uses a transaction
- SQLite batch path uses a transaction
- frontend uses one batch mutation for combined column changes
- schema caches are invalidated after DDL

These are source/implementation claims, not live runtime evidence.

## Provider matrix

| Provider | Declared support | Automated evidence | Live/runtime evidence |
|---|---|---|---|
| PostgreSQL | partial/provider-dependent ALTER support | source coverage | PENDING |
| SQLite | limited ALTER support; capability-gating required | source coverage | PENDING |

## Completion criteria

- [ ] rollback regression test proves no partial persistence after a middle-statement failure
- [ ] PostgreSQL combined mutation verified on a live provider
- [ ] PostgreSQL failing/narrowing mutation proves rollback
- [ ] SQLite operation matrix verifies supported vs capability-gated operations
- [ ] DDL/tableInfo/dependency surfaces refresh after mutation
- [ ] P0 = 0 and P1 = 0

S1 remains `RUNTIME_VERIFY` until these items are proven.
