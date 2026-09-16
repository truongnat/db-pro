# Agent evidence — #260 SQL Server provider

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | codex · SQL Server provider implementation lane |
| Issue(s) | #260 |
| Task state | Blocked |
| Baseline SHA | `cc399501338b7f38342a1bb12b9990e048537312` |
| Branch / PR | `fix/sqlserver-provider` · PR not opened |
| Scope interpretation | Build a real TDS-backed provider through the existing `DbConnector` and capability seams, with tests and fixture evidence. |
| Out of scope | #252 debugger, SQL Server administration/backup/security workspaces, issue completion without live fixture/UI evidence. |

## 2. Progress checkpoint

- Current HEAD: `774a4e7f6ce8d88939988711e8fadf0588f1c3c1`
- Completed acceptance rows: provider identity/factory, TDS lifecycle, query/execute/transactions, typed mapping, dialect, capabilities, catalog introspection, UI selection, unit/fixture test lane.
- Remaining acceptance rows: live SQL Server fixture and native UI/runtime evidence.
- Findings / risks: live SQL Server fixture absent; cancellation is explicitly capability-gated; independent review has not run.
- Tests already run: workspace 1164 passed/0 failed/42 ignored; fmt, check, clippy, release native build, clean-code scan, and SQL Server unit tests passed.
- Dependency / blocker changes: #234 provider seams are consumed; #260 cannot be marked complete until `SQLSERVER_URL` fixture evidence is collected.

## 3. Implementation handoff / review request

Ready for review with runtime-verification blocker recorded; do not mark #260 completed yet.

## 4. Review outcome

No independent review has run. Self-review verdict: ACCEPT WITH P2 / BLOCKED runtime evidence; P0=0, P1=0 introduced, P2=1 (missing live fixture/UI evidence).

## 5. Research / audit handoff

- Source date: 2026-09-17.
- Source URLs / references: GitHub issue #260 and its implementation-audit comments; `crates/core/src/ports/db_connector.rs`; `crates/infrastructure/src/connector.rs`; `Cargo.toml`.
- Factual findings: implementation SHA `774a4e7f6ce8d88939988711e8fadf0588f1c3c1` adds the SQL Server driver, TDS adapter, connector, introspection, capability matrix, UI path, and ignored fixture tests; no live SQL Server container is available.
- Inference: tiberius is a viable candidate adapter, but TLS/authentication and live fixture compatibility remain to be proven in this repository.
- Decision / recommendation: keep the plan in `RUNTIME_VERIFY`; run the four ignored tests plus native UI evidence against the canonical fixture before closing #260.
- Unresolved questions: which SQL Server image/version and authentication mode should be the canonical CI fixture; whether cancellation can be safely supported by the selected TDS client.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã triển khai provider SQL Server thật trên commit `774a4e7f6ce8d88939988711e8fadf0588f1c3c1`, gồm TDS lifecycle, query/transaction, typed decoding, introspection, capability gates, UI selection và integration lane. Tất cả gate tự động đã pass; issue vẫn chưa hoàn tất vì môi trường chưa có SQL Server live fixture và native UI evidence.
