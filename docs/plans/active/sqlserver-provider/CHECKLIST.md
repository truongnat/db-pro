# SQL Server provider checklist (#260)

## Planning

- [x] Confirm live GitHub issue and dependency state.
- [x] Record exact baseline SHA and scope boundary.
- [x] Decide and document TDS TLS/authentication support.

## Core/provider

- [x] Add `DriverType::SqlServer` with backward-compatible serde behavior.
- [x] Add TDS dependency and provider module.
- [x] Register SQL Server factory in `CompositeConnector`.
- [x] Implement connect/test/disconnect and handle lifecycle.
- [x] Implement query/execute/batch/transaction paths with typed parameters.
- [x] Implement explicit cancellation capability behavior.
- [x] Implement canonical SQL Server row decoding.
- [x] Implement SQL Server dialect and capability reasons.

## Introspection and product path

- [x] Implement SQL Server catalog introspection.
- [x] Verify shared schema/table data services consume the provider without name
      branches that bypass capability dispatch.
- [x] Add connection editor/provider selection and safe defaults.
- [ ] Verify native UI command → runtime worker → provider → database → event path
      with a live SQL Server fixture and native UI evidence.

## Verification

- [x] Add provider unit tests for quoting, parameters, decoding, and error mapping.
- [x] Add SQL Server fixture/conformance tests (ignored until `SQLSERVER_URL` exists).
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo check --workspace`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Run `cargo test --workspace` (1164 passed, 0 failed, 42 ignored).
- [x] Run `cargo build --release --locked -p db-pro-native`.
- [x] Run the clean-code diff scan.
- [ ] Record SQL Server live evidence and native UI evidence at required sizes.

## Handoff

- [x] Update `FINDINGS.md` with all P0/P1/P2 dispositions.
- [x] Update `VERIFICATION.md` with exact commands and results.
- [x] Fill `AGENT_EVIDENCE.md` with exact reviewed SHA.
- [ ] Keep the plan active until every completion gate is proven.
