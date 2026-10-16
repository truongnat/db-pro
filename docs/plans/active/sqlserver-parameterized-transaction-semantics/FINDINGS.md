# Findings — SQL Server Parameterized Transaction Semantics

## Findings

1. `SqlServerConnector::execute_transaction` correctly implements `already_open` detection via `open_transaction_count(handle)`.
2. `SqlServerConnector::execute_parameterized_transaction` lacked `open_transaction_count(handle)` checks and unconditionally executed `BEGIN TRANSACTION`, `ROLLBACK TRANSACTION` on error, and `COMMIT TRANSACTION` on success.
3. On SQL Server, `ROLLBACK TRANSACTION` resets `@@TRANCOUNT` to 0 regardless of transaction nesting depth, so issuing `ROLLBACK TRANSACTION` when an outer transaction is active destroys the caller's active transaction and produces wrong error outcomes (`RolledBack` instead of `LeftOpen`).
