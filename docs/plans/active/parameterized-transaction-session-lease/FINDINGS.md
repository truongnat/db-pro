# Findings: Parameterized Transaction Session Lease & Savepoint Isolation

## Findings & Risk Analysis
1. In `PostgresConnector::execute_parameterized_transaction` and `MySqlConnector::execute_parameterized_transaction`, when a session transaction is open (`open_session`), mutations bypassed the open connection and acquired a new connection from the pool.
2. If `execute_parameterized_transaction` was called while an explicit session transaction was open:
   - Modifications made in the open session were invisible to the parameterized transaction.
   - Row locks held by the open session caused deadlock or lock timeout when the parameterized transaction attempted to modify the same rows.
3. Solution:
   - Call `crate::transaction_session::checkout_if_held` prior to beginning a new pool transaction.
   - If an active lease exists, execute inside `SAVEPOINT dbpro_parameterized_tx`.
   - On statement or invariant failure, execute `ROLLBACK TO SAVEPOINT dbpro_parameterized_tx` and return `TransactionFailureOutcome::LeftOpen` so the outer transaction remains open for the user.
