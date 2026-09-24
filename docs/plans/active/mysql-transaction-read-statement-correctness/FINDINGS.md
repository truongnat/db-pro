# Findings: MySQL Transaction Read-Statement Execution and Validation

## P1 — Ignored `_read_statements` in `MySqlConnector::execute_transaction`
- **Location**: `crates/infrastructure/src/mysql/connector.rs`
- **Root Cause**: The method parameter `_read_statements` was unused. Every statement was routed through `sqlx::query(stmt).execute(&mut *tx)`, which returns rows affected rather than fetching query rows.
- **Impact**: Any `SELECT` statement in a MySQL transaction returned `Affected { row_count: 0 }`, causing callers (e.g. `QueryService`, `MultiQueryExecution`, data grid) to receive no columns and no rows for queries inside transactions.
- **Fix**: Check `statements.len() == read_statements.len()` first. Zip `statements` and `read_statements` and dispatch `is_read == true` queries through `sqlx::query(stmt).fetch_all(&mut *tx)` and `MySqlQueryMapper::map_rows`.
