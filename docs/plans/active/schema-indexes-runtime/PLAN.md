# S2 — Schema Indexes Runtime Verification

State: RUNTIME_VERIFY
Merged implementation: PR #2 / `aa77ece`

## Goal

Index management must preserve database truth across create/list/drop operations and provider boundaries.

```text
UI index action
→ dialect-aware SQL
→ DDL command/service
→ database mutation
→ introspection
→ refreshed index UI
```

## Scope

- list indexes
- create normal index
- create unique index
- create composite index
- drop index
- provider-aware identifier quoting
- metadata/cache refresh
- PostgreSQL and SQLite runtime accounting

## Non-goals

- relations/foreign keys
- triggers
- ER diagram
- expression/partial index editor UX unless explicitly added later

## Current evidence

The merged S2 work added a SQLite integration test proving:

- `CREATE UNIQUE INDEX`
- composite `CREATE INDEX`
- `DROP INDEX`
- SQLite introspection via `PRAGMA index_list` / `PRAGMA index_info`

This is SQLite automated evidence only. It does not prove PostgreSQL runtime behavior or the UI refresh lifecycle.

## Provider matrix

| Provider | Declared support | Automated evidence | Live/UI runtime evidence |
|---|---|---|---|
| PostgreSQL | yes | source only | PENDING |
| SQLite | yes | PASS: integration test | PENDING UI round-trip |

## Completion criteria

- [x] SQLite create/unique/composite/drop introspection regression test
- [ ] PostgreSQL create/unique/composite/drop verified against a real PostgreSQL provider
- [ ] UI create/drop → refreshed index list verified
- [ ] cache invalidation verified through the user-facing lifecycle
- [ ] P0 = 0 and P1 = 0

S2 remains `RUNTIME_VERIFY` until the pending provider/UI evidence is recorded.
