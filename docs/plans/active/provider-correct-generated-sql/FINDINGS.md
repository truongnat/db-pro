# Findings — #441 / #442

All repository source observations in this file refer to baseline `76b120724a7542de8b587a002811ddacad28156a` (not runtime observations).

| Severity | Baseline evidence | Failure and bounded decision |
|---|---|---|
| P1 / #441 | `crates/core/src/application/sql_builder/mutation.rs:94-110` appends `LIMIT 1`; `table_data_service.rs:103-121` submits the result unchanged. | SQL Server cannot execute this reload SQL. Existing key validation checks shape, not catalog uniqueness, so preserve the row cap via `SqlDialect::pagination_clause("1", "0")`. Add PK-column ORDER BY only when the dialect requires it; preserve key/value parameter order, with no added parameters or port changes. |
| P2 / #442 | `schema_ddl.rs:251-254` hardcodes double quotes throughout reconstruction; `schema_service.rs:197-214` already passes the actual connection driver. | MySQL default mode rejects reconstructed identifiers. Reuse `driver_dialect.rs:40-45` rather than retain a second quoting convention. A small crate-visible typed constructor avoids converting DriverType to a string. |
| P2 / related DDL syntax | `schema_ddl.rs:188-205` schema-qualifies reconstructed index names for every non-SQLite driver. | CREATE INDEX requires an unqualified index name for PostgreSQL/MySQL/SQL Server. Keep target table qualification; preserve authoritative PostgreSQL `index.definition` verbatim. |
| P2 / acceptance prerequisite | `schema_ddl.rs:108-116,208-219` only emits SQLite and PostgreSQL FKs; MySQL/SQL Server FKs are omitted. | Use the existing inline FK path for every non-PostgreSQL driver, with driver-correct target/constraint/column quoting. Keep composite order and existing action formatting. This is necessary for the requested FK-preserving reconstruction. |

## Consumer and invariant inventory

- Baseline `76b120724a7542de8b587a002811ddacad28156a`: `SchemaService::get_table_ddl` is the reconstruction consumer; views retain stored definitions, triggers use `format_trigger_ddl`. Helpers are private to `schema_service::schema_ddl`; the only direct old quote-helper consumer outside that module is its test import.
- Baseline `76b120724a7542de8b587a002811ddacad28156a`: `sql_builder/read.rs:16-38` already delegates pagination and SQL Server's ORDER BY requirement to `SqlDialect`. The other mutation builders have no LIMIT/OFFSET. No broad pagination changes are necessary.
- Baseline `76b120724a7542de8b587a002811ddacad28156a`: the reload builder/service accepts PK column/value lists without introspection and relies on the caller to supply the full real key. Reusing dialect pagination preserves the existing at-most-one-row behavior even when catalog uniqueness is not independently proven; no new catalog validation is added.
- Existing SQLite schema-prefix omission, inline unique constraints, PG/SQLite FK action/deferral formatting, raw CHECK/default/generated expressions and trigger definition/function ordering are intentionally preserved. Cross-dialect translation of SQL expression bodies and identity/generated-column syntax is outside #442.

## External SQL syntax references

Read on 2026-10-16; documentation evidence, not server execution:

- [PostgreSQL CREATE INDEX](https://www.postgresql.org/docs/current/sql-createindex.html): “No schema name can be included here”; index is created in its parent table's schema.
- [MySQL 8.4 CREATE INDEX](https://dev.mysql.com/doc/refman/8.4/en/create-index.html): `CREATE ... INDEX index_name ... ON tbl_name`.
- [SQL Server CREATE INDEX](https://learn.microsoft.com/en-us/sql/t-sql/statements/create-index-transact-sql?view=sql-server-ver17): `INDEX index_name ON <object>`; only the object accepts schema qualification.

## Remaining evidence boundary

Implementation and regression tests are handed off unexecuted. No claim of passing tests, live provider success or issue closure is made. Live SQL Server/MySQL require reachable writable disposable databases, credentials, the provider's expected TLS configuration and catalog visibility. Parent owns prerequisite discovery and all execution gates.

## Independent review corrections

ReviewDataSql blocked the first uncommitted candidate (schema DDL blob `6a56b45d9a14ed06d4eec068a32fbc21ed2efe2e`) on two newly consumed producer-contract defects. Parent classified the cross-database wrong-target integrity risk as P1. Both implementations and regression cases below are unexecuted worker corrections, not a claim that review/runtime now passes.

- **P2 SQL Server actions:** baseline `76b120724a7542de8b587a002811ddacad28156a`, `sqlserver/introspect.rs:247-280`, returns `NO_ACTION`, `SET_NULL`, `SET_DEFAULT` directly. Normalize descriptors to canonical spaced SQL actions in the producer, so all domain consumers see `NO ACTION` / `SET NULL` / `SET DEFAULT`. Added adjacent direct catalog-action unit coverage and live SchemaService recreation for all three action pairs. Also removed two inherited literal `// cc-scan` lines embedded inside that FK SQL statement: they were SQL payload, not Rust comments, and would prevent this producer path executing.
- **P1 MySQL referenced database:** baseline `76b120724a7542de8b587a002811ddacad28156a`, `mysql/introspect.rs:214-271`, omits REFERENCED_TABLE_SCHEMA and joins catalog rows on constraint name alone. Select/retain the actual referenced schema and scope joins by constraint schema, table and name. The ignored provider test creates identical constraint/table names across two databases and divergent local/remote parent keys; after SchemaService reconstruction, remote-only keys must succeed and local-only keys must fail.
- **Supporting-index interaction:** baseline `76b120724a7542de8b587a002811ddacad28156a`, `mysql/introspect.rs:158-207`, includes FK supporting indexes in Index metadata; the newly enabled inline FK branch creates its own index if not already declared. Emit MySQL indexes inline before FKs and suppress duplicate post-create index statements. Primary filtering and unique flags are retained; PostgreSQL native expression index SQL remains unchanged. Existing MySQL functional-index EXPRESSION metadata is not selected at baseline and is an inherited limitation, not claimed support.

Parent expanded ownership to these two introspection sources and adjacent tests. No port/dependency change or FK-dropping workaround was introduced. Review recheck and all gates remain integration-owner tasks.
