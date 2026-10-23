# Findings — Parameterized Transaction Session Isolation

## P0/P1 Findings
- **P1**: In `MySqlConnector::execute_parameterized_transaction` and `PostgresConnector::execute_parameterized_transaction`, active connection session leases (`open_session`) were ignored when executing parameterized transactions. This caused staged mutations to run on an unisolated connection from the pool rather than within the open transaction on the leased connection.
