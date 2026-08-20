# RC1 P2 Grid & Connection Correctness Checklist

- [ ] Data Grid Columns picker checkbox click calls `onToggleHiddenColumn` exactly once
- [ ] Editing host, port, database, username, password, or SSH fields in Connection Editor clears previous test status badge
- [ ] Connection test error displays specific backend error details (`userMessage`) when available
- [ ] Frontend typecheck passes (`npm run typecheck`)
- [ ] Frontend tests pass (`npm run test`)
- [ ] Rust tests pass (`cargo test -p db-pro-core -p db-pro-infrastructure`)
