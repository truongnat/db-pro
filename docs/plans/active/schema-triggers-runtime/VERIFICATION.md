# S4 — Schema Triggers Runtime Verification

State: PLANNING

## Current audit findings

### Domain model gaps (CONFIRMED)

Current `Trigger` struct:
```rust
pub struct Trigger {
    pub name: String,
    pub event: String,    // SQLite: full SQL body (misuse); PG: event_manipulation only
    pub action: String,   // SQLite: always empty; PG: action_statement only
}
```

Missing fields: `table_name`, `schema`, `timing`, `definition`, `enabled`.

### SQLite introspection (CONFIRMED)

`introspect_triggers()` reads `tbl_name` but discards it:
```rust
let _table: String = row.get(1)?;  // DISCARDED
```

`event` field is misused to store the full SQL body. `action` is always empty.

### PostgreSQL introspection (CONFIRMED)

Missing: `event_object_table`, `action_timing`, `action_condition`, trigger function body from `pg_proc`.

## Commands to be executed

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace

cd frontend
npm run typecheck
npm run lint
npm run format:check
npm run test -- --run
npm run build
```

No PASS claims until these actually run after implementation.

## Provider matrix

| Provider | Introspection | CREATE | DROP | DDL | Enabled/Disabled |
|---|---|---|---|---|---|
| PostgreSQL | PENDING | PENDING | PENDING | PENDING | PENDING |
| SQLite | PENDING | PENDING | PENDING | PENDING | NOT_SUPPORTED (capability-gated) |
