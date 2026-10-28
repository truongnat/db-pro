# Findings: MySQL Parameterized Transaction Session Lease

## Initial Discovery
`MySqlConnector::execute_parameterized_transaction` directly called `pool.begin().await` without calling `checkout_if_held(session, ...)` on `entry.open_session`.
This meant that when a transaction session was open on a query tab, parameterized mutations ran on a second connection from the pool.
By using `checkout_if_held` and savepoints (`SAVEPOINT dbpro_tx_<seq>`), parameterized mutations now execute on the active session lease and roll back with the query tab transaction when rolled back.
