# Defect Finding: Invalid SQL Generation on Empty Schema Qualification

## Severity: P1 (Wrong database mutation / Unsafe SQL / Provider capability mismatch)

### Area
- Core SQL Query Building (`sql_builder.rs`)
- DDL Generation (`schema_service.rs`)
- Data & Schema Diffing (`data_diff.rs`, `schema_diff.rs`)
- PostgreSQL Object Operations (`cross_connection.rs`)

### Code Evidence
In `crates/core/src/application/sql_builder.rs`:
```rust
let sql = format!(
    "SELECT * FROM {}.{}{}{}{pagination}",
    dialect.quote_identifier(schema),
    dialect.quote_identifier(table),
    where_clause,
    order_clause,
);
```
When `schema` is `""`, `dialect.quote_identifier("")` outputs `""`. The resulting query is `SELECT * FROM ""."table_name"`.

Similar unconditional formatting exists in:
- `build_count` (`sql_builder.rs`)
- `build_insert` (`sql_builder.rs`)
- `build_update` (`sql_builder.rs`)
- `build_delete` (`sql_builder.rs`)
- `build_create_table_ddl` (`schema_service.rs`)
- `diff_table_data` (`data_diff.rs`)
- `rename_schema_object` (`cross_connection.rs`)

### Failure Scenario
1. User interacts with an SQLite table or database where `schema` is empty `""`.
2. Data grid queries (select/count/insert/update/delete) or DDL actions execute `sql_builder` or `schema_service` methods.
3. The generated SQL includes `""."table_name"`.
4. The database driver rejects the statement with a syntax error (`zero-length delimited identifier` in PostgreSQL or `syntax error near "."` in SQLite).
5. The operation fails completely, preventing row display, editing, deletion, insertion, DDL view, and diffing for un-scoped tables.

### Required Fix
Provide safe qualification logic (`qualify(dialect, schema, table)`) that omits `schema.` when `schema.is_empty()`.
