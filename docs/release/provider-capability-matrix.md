# PostgreSQL vs SQLite v0.1 Capability Matrix

> Source: `crates/core/src/domain/capabilities.rs` (canonical), infrastructure implementations.
> Baseline SHA: `65bbca3`
> Issue: #132

## Matrix legend

| Value | Meaning |
|---|---|
| **SUPPORTED + QUALIFIED** | Implemented and verified by integration/smoke tests |
| **SUPPORTED + NOT YET QUALIFIED** | Implemented in source but no automated test coverage yet |
| **PARTIAL** | Works with caveats documented in Notes |
| **READ-ONLY** | Can read/introspect but not mutate |
| **NOT SUPPORTED** | Not implemented, no path to support in v0.1 |
| **DEFERRED** | Intentionally excluded from v0.1 |

## Connection

| Capability | PostgreSQL | SQLite | Notes |
|---|---|---|---|
| Create connection | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | PG via sqlx, SQLite via rusqlite (bundled) |
| Test connection | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | `test_connection` in DbConnector trait |
| Disconnect | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Reconnect | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | Manual reconnection path exists |
| Password storage | SUPPORTED + QUALIFIED | N/A | Keyring + encrypted fallback (SQLite has no auth) |
| TLS/SSL | SUPPORTED + NOT YET QUALIFIED | N/A | PG via `runtime-tokio-rustls`; SQLite is local file |
| SSH tunnel | SUPPORTED + NOT YET QUALIFIED | NOT SUPPORTED | Shells out to `ssh` binary; not E2E qualified (#issue) |
| File picker (SQLite) | N/A | SUPPORTED + NOT YET QUALIFIED | `tauri-plugin-dialog` `dialog:allow-open` |

## Schema / Introspection

| Capability | PostgreSQL | SQLite | Notes |
|---|---|---|---|
| Named schemas | SUPPORTED + QUALIFIED | NOT SUPPORTED | SQLite has no schema concept |
| Tables | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Views | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Columns | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Default values | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Generated columns | SUPPORTED + QUALIFIED | NOT SUPPORTED | SQLite has no GENERATED ALWAYS AS |
| Identity columns | SUPPORTED + NOT YET QUALIFIED | NOT SUPPORTED | PG `GENERATED ... AS IDENTITY` |
| Primary keys | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Composite PK | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Unique constraints | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Foreign keys | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Composite FK | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Indexes | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Functional indexes | SUPPORTED + QUALIFIED | NOT SUPPORTED | PG expression indexes |
| GIN/GiST indexes | SUPPORTED + QUALIFIED | NOT SUPPORTED | PG-specific |
| CHECK constraints | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | Disposition pending (#68) |
| Triggers | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Functions/Procedures | SUPPORTED + QUALIFIED | NOT SUPPORTED | SQLite has no stored procedures |
| Sequences | SUPPORTED + QUALIFIED | NOT SUPPORTED | SQLite uses AUTOINCREMENT |
| Enum types | SUPPORTED + QUALIFIED | NOT SUPPORTED | SQLite uses CHECK constraints |
| Array types | SUPPORTED + QUALIFIED | NOT SUPPORTED | PG `TEXT[]` etc. |
| DDL source extraction | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | |
| Rename objects | SUPPORTED + QUALIFIED | PARTIAL | SQLite limited to RENAME TABLE/COLUMN |
| ALTER COLUMN type | SUPPORTED + QUALIFIED | NOT SUPPORTED | SQLite cannot alter column type |
| ADD COLUMN | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| DROP COLUMN | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Transactional DDL | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | Both support DDL in transactions |

## Query / Results

| Capability | PostgreSQL | SQLite | Notes |
|---|---|---|---|
| Single statement | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Multi-statement | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| EXPLAIN | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | PG returns JSON plan; SQLite returns text |
| Cancel query | SUPPORTED + QUALIFIED | NOT SUPPORTED | SQLite has no cancel mechanism |
| Parameterized queries | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Numbered params ($1) | SUPPORTED + QUALIFIED | NOT SUPPORTED | PG-specific |
| Positional params (?) | NOT SUPPORTED | SUPPORTED + QUALIFIED | SQLite-specific |
| Max rows limit | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | Configurable per-connection |
| JSON type | SUPPORTED + QUALIFIED | PARTIAL | PG: native JSONB; SQLite: TEXT with JSON content |
| UUID type | SUPPORTED + QUALIFIED | NOT SUPPORTED | SQLite stores as TEXT |
| BYTEA/BLOB | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | PG BYTEA ↔ SQLite BLOB |
| Numeric precision | SUPPORTED + QUALIFIED | PARTIAL | SQLite REAL is IEEE 754; no NUMERIC(p,s) |

## Data Grid

| Capability | PostgreSQL | SQLite | Notes |
|---|---|---|---|
| Read/filter/sort | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Pagination | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| PK update | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| PK delete | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| No-PK tables | READ-ONLY | READ-ONLY | Cannot update without PK |
| Row insert | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Transaction | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| RETURNING | SUPPORTED + QUALIFIED | PARTIAL | SQLite supports RETURNING since 3.35 |
| Provider-aware editing | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | JSON/array editing differs |

## Import / Export / Backup

| Capability | PostgreSQL | SQLite | Notes |
|---|---|---|---|
| Backup | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | PG: pg_dump; SQLite: file copy/VACUUM |
| Restore | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | |
| CSV export | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | Via `csv` crate |
| XLSX export | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | Via `rust_xlsxwriter` crate |
| Import | DEFERRED | DEFERRED | Not in v0.1 scope |

## ER Diagram

| Capability | PostgreSQL | SQLite | Notes |
|---|---|---|---|
| Metadata introspection | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | Tables, columns, FKs |
| Graph model build | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | `buildErGraphModel` |
| Small/Medium (React Flow) | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | <200 tables, S/M tier |
| Large (Cytoscape) | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | >200 tables or L/XL tier |
| Search-first entry | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | Gate 4 Slice B |
| Neighborhood BFS | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | Gate 4 Slice A |

## Other

| Capability | PostgreSQL | SQLite | Notes |
|---|---|---|---|
| SSH tunnel | SUPPORTED + NOT YET QUALIFIED | NOT SUPPORTED | External `ssh` binary; not E2E qualified |
| User/role management | SUPPORTED + NOT YET QUALIFIED | NOT SUPPORTED | PG has `pg_roles`; SQLite has no users |
| Server sessions | SUPPORTED + QUALIFIED | NOT SUPPORTED | PG capability flag |
| Partitions | SUPPORTED + QUALIFIED | NOT SUPPORTED | PG capability flag |
| Tablespaces | SUPPORTED + QUALIFIED | NOT SUPPORTED | PG capability flag |
| Cross-schema deps | SUPPORTED + QUALIFIED | NOT SUPPORTED | PG capability flag |
| Schema diff | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | |
| Data diff | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | |
| Agent workspace | DEFERRED | DEFERRED | Preview only, not production |
| MCP | DEFERRED | DEFERRED | Not in v0.1 |

## Key provider differences summary

1. **Cancel**: PG supports query cancellation; SQLite does not.
2. **Schemas**: PG has named schemas; SQLite has none.
3. **Types**: PG has native UUID, JSONB, arrays, enums, generated columns; SQLite stores all as TEXT/REAL/INTEGER/BLOB.
4. **Functions**: PG has stored functions/procedures; SQLite does not.
5. **Sequences**: PG has standalone sequences; SQLite uses AUTOINCREMENT only.
6. **ALTER COLUMN**: PG can change column types; SQLite cannot.
7. **SSH**: PG connections can tunnel via SSH; SQLite is local file only.
8. **Indexes**: PG supports GIN, GiST, expression indexes; SQLite supports B-tree only.

## Source references

- `crates/core/src/domain/capabilities.rs` — canonical capability definitions
- `crates/infrastructure/src/postgres/connector.rs` — PG connector implementation
- `crates/infrastructure/src/sqlite/connector.rs` — SQLite connector implementation
- `crates/infrastructure/src/postgres/introspect.rs` — PG introspection (734 lines)
- `crates/infrastructure/src/sqlite/introspect.rs` — SQLite introspection (423 lines)
- `crates/infrastructure/src/ssh/tunnel.rs` — SSH tunnel (external process)
- `crates/infrastructure/src/backup/pg_dump.rs` — PG backup via pg_dump
- `crates/infrastructure/src/backup/sqlite_backup.rs` — SQLite backup via file copy
