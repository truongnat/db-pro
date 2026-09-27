# Findings: MySQL Batch Statement Execution Transaction Atomicity

## Issue Analysis
`MySqlConnector::execute_batch` directly executed each SQL statement against a `MySqlPool` instance without initiating a transaction (`pool.begin()`).

In MySQL, executing statements directly against the pool executes each statement in `autocommit` mode.
If a batch contains statements $S_1, S_2, \dots, S_n$ and statement $S_i$ ($1 < i \le n$) fails:
- Statements $S_1 \dots S_{i-1}$ are committed immediately and cannot be rolled back.
- The caller receives an error for statement $S_i$, leaving the database in an inconsistent partial state.

## Comparison across Connectors
- **PostgresConnector**: Opens `pool.begin()`, executes statements inside `tx`, and performs `tx.rollback()` on error or timeout.
- **SqliteActor**: Executes batch statements inside a rusqlite transaction (`conn.transaction()`), rolling back on error.
- **SqlServerConnector**: Delegates batch execution directly to `execute_transaction(handle, statements, &vec![false; statements.len()])`.

## Solution Strategy
Align `MySqlConnector::execute_batch` with `SqlServerConnector::execute_batch` by delegating batch execution directly to `self.execute_transaction(handle, statements, &vec![false; statements.len()]).await`.
This ensures:
1. `BEGIN` is issued before executing statements.
2. All statements execute within the same transaction context.
3. If any statement fails, `tx.rollback()` is executed automatically by `execute_transaction`.
4. If all statements succeed, `tx.commit()` is issued.
