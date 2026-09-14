# PostgreSQL vs SQLite v0.1 Capability Matrix

> **Note (2026-09-11).** Provider capability semantics are UI-independent and remain valid.
> The presentation layer changed: the React frontend was archived under `_archive/frontend/`
> and the UI is now native `eframe`/`egui`. Capability gating must be enforced in the
> backend/domain layer, never in the UI.

> Source: `crates/core/src/domain/capabilities.rs` (canonical), infrastructure implementations.
> Baseline SHA: `65bbca3`; **corrected 2026-09-14 for candidate `fbf9fdab9100f08f12e29434983f32c18f14ac2f`**
> Issue: #132
>
> **Corrections in this revision (2026-09-14):** the `Cancel query` row and the key-difference
> summary were **inverted** relative to the code and are fixed here; `Sequences` and `Enum types`
> were shown as supported although no catalog implementation exists and are now `UNSUPPORTED`;
> the SQLite file-picker row no longer cites the retired Tauri dialog plugin; the backup source
> reference now names `VACUUM INTO`; a note records that provider *runtime* evidence for the
> candidate is not retrievable, so `QUALIFIED` throughout this table means automated test
> coverage, not a recorded live-provider run (`docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md`).
> PostgreSQL support is never inferred from a SQLite result, or vice versa. Original rows not named
> above are unchanged.

## Matrix legend

| Value | Meaning |
|---|---|
| **SUPPORTED + QUALIFIED** | Implemented and verified by automated integration tests. **Runtime/provider QA is not implied** — the candidate's live-provider runs have no retrievable artifacts (2026-09-14 audit) |
| **SUPPORTED + NOT YET QUALIFIED** | Implemented in source but no test coverage yet |
| **PARTIAL** | Works with caveats documented in Notes |
| **READ-ONLY** | Can read/introspect but not mutate |
| **NOT SUPPORTED** | Not implemented, no path to support in v0.1 (= release vocabulary `UNSUPPORTED`) |
| **DEFERRED** | Intentionally excluded from v0.1 |

## Connection

| Capability | PostgreSQL | SQLite | Notes |
|---|---|---|---|
| Create connection | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | PG via sqlx, SQLite via rusqlite (bundled) |
| Test connection | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | `test_connection` in DbConnector trait |
| Disconnect | SUPPORTED + QUALIFIED | SUPPORTED + QUALIFIED | |
| Reconnect | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | Manual reconnection path exists |
| Password storage | SUPPORTED + NOT YET QUALIFIED | N/A | OS keyring (Keychain/Credential Manager/keyutils); encrypted fallback is dev-only (#142). SQLite has no auth |
| TLS/SSL | SUPPORTED + NOT YET QUALIFIED | N/A | PG via `runtime-tokio-rustls`; SQLite is local file |
| SSH tunnel | SUPPORTED + NOT YET QUALIFIED | NOT SUPPORTED | Shells out to `ssh` binary; not E2E qualified (#issue) |
| File picker (SQLite) | N/A | SUPPORTED + NOT YET QUALIFIED | Native `rfd` file dialog (`crates/ui`); the earlier `tauri-plugin-dialog` wiring belonged to the retired Tauri host |

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
| Sequences | **NOT SUPPORTED** | NOT SUPPORTED | **Corrected 2026-09-14:** `DatabaseCapabilities::postgres()` declares `sequences: true`, but no catalog query or UI exists (`postgres/introspect.rs` has none; only a `nextval` textual heuristic). The flag is wrong; the matrix follows the code's real capability. Deferred to a code pass (`R-PROV`) |
| Enum types | **NOT SUPPORTED** | NOT SUPPORTED | **Corrected 2026-09-14:** `enum_types: true` is declared with zero implementation — no `pg_enum`/`pg_type` catalog query and no UI. Types arrive as `format_type()` strings. Deferred to a code pass (`R-PROV`) |
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
| Cancel query | **NOT SUPPORTED** | **SUPPORTED + QUALIFIED** | **Corrected 2026-09-14 (was inverted).** Code: `capabilities.rs` `postgres.cancel = false` / `sqlite.cancel = true`; SQLite interrupts the running VM (`sqlite/actor.rs`), PostgreSQL exposes no wire-level cancel (`postgres/connector.rs`). The Stop control is gated on this flag |
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
| Row insert | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | Backend exists; UI incomplete for complex column types (#35 backlog v0.2) |
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
| Large (Cytoscape) | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | >200 tables or L/XL tier; #18 bounded rendering + #19 safe LOD not yet complete |
| Search-first entry | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | Gate 4 Slice B; runtime qualification pending |
| Neighborhood BFS | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | Gate 4 Slice A; #18 bounded neighborhood not yet complete |

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

1. **Cancel (corrected 2026-09-14)**: **SQLite supports query cancellation** (VM interrupt + actor acknowledgement); **PostgreSQL does not** (capability-gated `Unsupported`, no wire-level cancel exposed). The previous wording stated the exact opposite.
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
- `crates/infrastructure/src/backup/sqlite_backup.rs` — SQLite backup via `VACUUM INTO` + atomic no-overwrite publish (not a file copy)
