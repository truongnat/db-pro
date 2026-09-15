# PostgreSQL vs SQLite v0.1 Capability Matrix (plus the post-v0.1 MySQL provider)

> **Note (2026-09-11).** Provider capability semantics are UI-independent and remain valid.
> The presentation layer changed: the React frontend was archived under `_archive/frontend/`
> and the UI is now native `eframe`/`egui`. Capability gating must be enforced in the
> backend/domain layer, never in the UI.

> Source: `crates/core/src/domain/capabilities.rs` (canonical), infrastructure implementations.
> Baseline SHA: `65bbca3`; **corrected 2026-09-14 for candidate `fbf9fdab9100f08f12e29434983f32c18f14ac2f`**
> Issue: #132

> **MySQL 8 added 2026-09-15 (#234/#235).** The two tables above and below describe the two
> **shipped v0.1 drivers**. MySQL 8 now has a real provider on `main` — a third engine, not a
> shipped one — so it is recorded in its own section, `## MySQL 8 (post-v0.1 provider)`, with the
> status of each capability area as it actually is on `main` today. MySQL is never described with
> a PostgreSQL or SQLite result, and a capability it does not have is stated as such rather than
> left out.
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
| Password storage | SUPPORTED + NOT YET QUALIFIED | N/A | OS keyring (Keychain/Credential Manager/keyutils) is the primary store in every build; a release build with no keyring item keeps the password in memory for the session only, and the encrypted-file fallback is debug/`DB_PRO_ALLOW_FILE_SECRET_FALLBACK`-gated so a release build never reads it (#142). SQLite has no auth |
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
| Backup (shipped UI) | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + QUALIFIED | PG: `pg_dump` must be on `PATH` (not bundled — LIM-015); the destination is reserved with `create_new` and removed on failure. SQLite: `VACUUM INTO` + fsync + no-overwrite publish, sidecar-clean, 8 regression tests (#145). **A running backup cannot be cancelled** (#244, stated in the release notes as `NOT SUPPORTED` — LIM-020) |
| Restore (shipped UI) | PARTIAL | SUPPORTED + QUALIFIED | PG: `psql -f` / `pg_restore` run **without a transaction boundary** by design, so a mid-script failure leaves the database partially restored — **#244 chose option (b)**: the behaviour is kept and the consequence is now stated in the restore failure itself ("the restore was NOT run in a transaction … may now be partially restored; inspect it before using it"), with the argv shape pinned by a test and the limitation recorded as LIM-020. Option (a) (`--single-transaction`) stays available to the owner but is a compatibility decision: dumps containing non-transactional statements would fail as a whole; tools must be on `PATH`. SQLite: staged copy → `PRAGMA quick_check` → atomic rename + sidecar removal + introspection-cache drop |
| CSV / TSV export (query view) | SUPPORTED + NOT YET QUALIFIED | SUPPORTED + NOT YET QUALIFIED | the shipped writer is the query view's own delimited serializer, shared with the clipboard paths and pinned by tests — **not** the core `ExportService`; **atomic publish as of #244** (sibling temp file + rename, scratch file removed on failure, overwrite confirmation, capped results named as capped — 6 regression tests) and **not cancellable** (LIM-020) |
| JSON / XLSX export | NOT SUPPORTED | NOT SUPPORTED | the core `ExportService` (`csv` + `rust_xlsxwriter`) has **no shipped surface**: `grep -rn "export_api()" crates/native-app crates/ui crates/runtime` → 0 hits, and no `xlsx` reference exists in `crates/ui`. Library code reachable only from the legacy `crates/tauri-app` |
| Import | DEFERRED | DEFERRED | Not in v0.1 scope — no code path at all (LIM-012) |
| Backup / restore cancellation | NOT SUPPORTED | NOT SUPPORTED | `RuntimeCommand::CancelOperation` is declared and handled but constructed nowhere in the workspace; #244 chose the issue's second accepted option — **no Stop affordance is promised**, and the absence is stated in the release notes as well as here (LIM-020) |

Classification and per-operation partial-failure semantics: `docs/release/audit-data-integrity.md` (#128).

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

## MySQL 8 (post-v0.1 provider)

**Status: `PARTIAL_ON_MAIN`, not a shipped driver.** Live evidence: `fixtures/mysql/` plus
`crates/infrastructure/tests/mysql_fixture_matrix.rs` and `mysql_integration.rs`, recorded in
`docs/release/evidence/v01-runtime/providers/60-mysql-live-fixture-and-mapper.md` (MySQL 8.4.11,
container `dbpro-v01-mysql-fixture`). The driver card is **disabled** in the UI, so nothing in
this section is reachable by a user in the shipped build.

Legend: the vocabulary is this document's, plus `NOT REACHABLE FROM THE UI` for capabilities that
work through the runtime/tests but have no enabled user surface.

| Capability | MySQL 8 | Notes |
|---|---|---|
| Create / test / disconnect connection | SUPPORTED + QUALIFIED | Live tests connect, execute and disconnect against the fixture |
| Connect from the UI | **NOT REACHABLE FROM THE UI** | Driver card is `is_disabled: true` ("Soon", `crates/ui/src/connection_view.rs`) |
| TLS/SSL | SUPPORTED + NOT YET QUALIFIED | `ssl_mode` is carried in the config; no MySQL TLS test exists |
| SSH tunnel | NOT SUPPORTED | `capabilities.mysql.ssh_tunnel = false` |
| Single statement | SUPPORTED + QUALIFIED | Live query/execute tests; the transaction path rolls back on failure |
| Multi-statement | SUPPORTED + NOT YET QUALIFIED | `execute_batch` iterates statements on the connector; no MySQL batch test exists |
| EXPLAIN | SUPPORTED + QUALIFIED | `mysql_explain_returns_json` |
| Cancel query | NOT SUPPORTED | Connector returns `DbError::Unsupported`; capability is `false` |
| Parameterized queries | **NOT SUPPORTED** | `MySqlConnector::query`/`execute` drop the parameter list — no binder exists (`parameters`, `positional_parameters`, `numbered_parameters` are all `false`) |
| Named / positional schemas | SUPPORTED + QUALIFIED | MySQL databases are exposed as schemas; introspection is live-verified |
| Tables / views / columns | SUPPORTED + QUALIFIED | Canonical model asserted by the live fixture tests |
| Default values / identity | SUPPORTED + NOT YET QUALIFIED | Read by the introspector (`column_default`, `extra` → `auto_increment`); not asserted live for MySQL |
| Primary keys, composite PK, FKs, indexes | SUPPORTED + QUALIFIED | Composite PK order, FK from/to and index→table grouping are pinned by live tests (indexes previously lost their table — defect D5) |
| Triggers | SUPPORTED + QUALIFIED | Trigger timing/event/table read live |
| Stored functions / routines | SUPPORTED + QUALIFIED | Routine introspection read live |
| CHECK constraints | PARTIAL | Introspection returns them empty for MySQL (recorded, not fixed) |
| Generated columns | SUPPORTED + NOT YET QUALIFIED | `extra` containing `GENERATED` maps to `is_generated`; no MySQL test asserts it |
| ENUM / SET | SUPPORTED + QUALIFIED | Both decode as `Text` in the live 41-column matrix |
| Sequences | NOT SUPPORTED | MySQL 8 has no standalone sequences |
| Schema diff | SUPPORTED + NOT YET QUALIFIED | Compares two `IntrospectResult`s, so it needs no MySQL-specific path; no MySQL test |
| Data diff | **NOT SUPPORTED** | Needs a dialect and there is no MySQL dialect arm; capability is `false` |
| Raw SELECT (query editor) | SUPPORTED + QUALIFIED | Live query path; values decode per the provider-value contract |
| Data Grid read / filter / sort / pagination | **NOT REACHABLE** | `TableDataService` asks for a dialect, which MySQL has no arm for; the capability is not the blocker here |
| Row write via the query editor | SUPPORTED + NOT YET QUALIFIED | Plain SQL through `execute`: `INSERT` is live-tested (`mysql_execute_returns_affected_rows`); `UPDATE`/`DELETE` are the same path with no MySQL-specific test |
| Table Data Editor (staged mutations) | **NOT REACHABLE** | `CompositeConnector::dialect` has no MySQL arm, so the mutation paths return a named validation error |
| Value decoding (all classes) | SUPPORTED + QUALIFIED | Class-aware mapper; 41-column live matrix, `DECIMAL`/temporal/JSON/GEOMETRY/bytes byte-exact (`providers/60-…` §3) |
| User / role management (`server_sessions`) | NOT SUPPORTED | `UserService` rejects every non-PostgreSQL driver; capability is `false` |
| Partitions | NOT SUPPORTED | `PostgresApi::partitions` resolves a PostgreSQL handle; capability is `false` |
| Tablespaces / cross-schema dependencies | NOT SUPPORTED | PostgreSQL-only paths; capabilities are `false` |
| Backup / restore | NOT SUPPORTED | `BackupService` returns a validation error for MySQL; capability is `false` |
| ER diagram | SUPPORTED + NOT YET QUALIFIED | Built from introspection, which works; no MySQL-specific ER test |
| Import / export | DEFERRED | Not implemented for any driver in v0.1 (LIM-012) |

### The rule behind the MySQL column

A capability is `SUPPORTED` for MySQL only where a shipping code path serves it. Where the engine
could do the work but the product's only path rejects MySQL, the row says `NOT SUPPORTED` and the
capability flag is `false` — that is the rule stated in `DatabaseCapabilities::mysql()`. The
provider contract itself is in `docs/architecture/provider-contract.md`.

## Key provider differences summary

1. **Cancel (corrected 2026-09-14)**: **SQLite supports query cancellation** (VM interrupt + actor acknowledgement); **PostgreSQL does not** (capability-gated `Unsupported`, no wire-level cancel exposed). The previous wording stated the exact opposite.
2. **Schemas**: PG has named schemas; SQLite has none.
3. **Types**: PG has native UUID, JSONB, arrays, enums, generated columns; SQLite stores all as TEXT/REAL/INTEGER/BLOB.
4. **Functions**: PG has stored functions/procedures; SQLite does not.
5. **Sequences**: PG has standalone sequences; SQLite uses AUTOINCREMENT only.
6. **ALTER COLUMN**: PG can change column types; SQLite cannot.
7. **SSH**: PG connections can tunnel via SSH; SQLite is local file only.
8. **Indexes**: PG supports GIN, GiST, expression indexes; SQLite supports B-tree only.
9. **MySQL (post-v0.1)**: a real third provider with a class-aware decoder and live evidence, but not a shipped driver — no parameter binder, no dialect arm, a disabled UI card, and the PostgreSQL-only services (user management, partitions, backup) refused with named errors. Its capability set advertises those as `false` rather than promising them.

## Source references

- `crates/core/src/domain/capabilities.rs` — canonical capability definitions
- `crates/infrastructure/src/postgres/connector.rs` — PG connector implementation
- `crates/infrastructure/src/sqlite/connector.rs` — SQLite connector implementation
- `crates/infrastructure/src/postgres/introspect.rs` — PG introspection (734 lines)
- `crates/infrastructure/src/mysql/` — MySQL 8 connector, class-aware query mapper and introspection (post-v0.1 provider)
- `crates/core/src/ports/provider_factory.rs` — the factory port; the contract it belongs to is `docs/architecture/provider-contract.md`
- `crates/infrastructure/tests/provider_contract_conformance.rs` — factory dispatch, capability advertisement and the live value/decoder conformance case
- `crates/infrastructure/src/sqlite/introspect.rs` — SQLite introspection (423 lines)
- `crates/infrastructure/src/ssh/tunnel.rs` — SSH tunnel (external process)
- `crates/infrastructure/src/backup/pg_dump.rs` — PG backup via pg_dump
- `crates/infrastructure/src/backup/sqlite_backup.rs` — SQLite backup via `VACUUM INTO` + atomic no-overwrite publish (not a file copy)
- `docs/release/audit-data-integrity.md` — export/import/backup classification, partial-failure semantics and cancellation findings (#128)
