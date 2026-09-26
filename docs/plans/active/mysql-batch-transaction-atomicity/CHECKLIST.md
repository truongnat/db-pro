# Checklist

- [ ] `MySqlConnector::execute_batch` updated to delegate to `execute_transaction`
- [ ] Unit tests added in `crates/infrastructure/src/mysql/connector.rs` for `execute_batch`
- [ ] Automated tests (`cargo test -p db-pro-core -p db-pro-ui`) pass
- [ ] Pre-commit instructions completed
- [ ] Pull request published on branch `fix/mysql-batch-transaction-atomicity`
