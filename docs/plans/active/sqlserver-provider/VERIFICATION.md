# SQL Server provider verification (#260)

## Baseline

- SHA: `cc399501338b7f38342a1bb12b9990e048537312`
- State: `RUNTIME_VERIFY`
- Implementation SHA: `774a4e7f6ce8d88939988711e8fadf0588f1c3c1`

## Commands

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace --quiet` | PASS: 1164 passed, 0 failed, 42 ignored |
| `cargo build --release --locked -p db-pro-native` | PASS |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | PASS: 13 pass, 3 warnings, 0 fail |
| `cargo test -p db-pro-infrastructure --test sqlserver_integration -- --list` | PASS: 4 ignored fixture tests registered |
| `cargo test -p db-pro-infrastructure sqlserver -- --nocapture` | PASS: 2 provider unit tests |

## Provider matrix

| Provider | Supported operation | Automated evidence | Live/runtime evidence | Capability gate |
|---|---|---|---|---|
| SQL Server | connect/query/execute/batch/transaction/introspection/dialect implemented | unit tests PASS; 4 ignored fixture tests registered | BLOCKED: no `SQLSERVER_URL` or live SQL Server fixture | explicit capability matrix in `db-pro-core` |
| PostgreSQL | existing support | NOT RE-RUN | NOT VERIFIED in this task | unchanged/out of scope |
| SQLite | existing support | NOT RE-RUN | NOT VERIFIED in this task | unchanged/out of scope |

## Runtime evidence

No native UI screenshot/recording or live SQL Server evidence has been collected.
Docker was checked on 2026-09-17: no SQL Server container/image was available in
the environment. The fixture command is documented in
`crates/infrastructure/tests/sqlserver_integration.rs`.
