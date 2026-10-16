# Agent evidence — #452 typed MySQL connection options

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | MysqlTypedOptions / delegated #452 lane |
| Issue(s) | #452 |
| Task state | Ready — implementation complete; parent verification/publication pending |
| Baseline SHA | `bceb735727f81180dad678defee1000bc547d97b` |
| Branch / PR | `main`; no worker commit or PR |
| Scope interpretation | Replace interpolated MySQL URLs with native field options, retain TLS/pool/timeout behavior, remove obsolete URL tests, add isolated consumer-visible real-provider cases and audit analogous provider setup. |
| Out of scope | PostgreSQL/SQL Server edits/runtime, CSV, dependencies/manifests, shared changelog/status, fixture execution, gates, commits/push and issue closure. |

## 2. Progress checkpoint

- Source baseline: `bceb735727f81180dad678defee1000bc547d97b`; parent provided this SHA and exported it to `/tmp/db-pro-wave140-baseline`. Candidate is the uncommitted assigned-file patch based on that baseline, not a separately verified/published SHA. No worker Git commands were run.
- Completed implementation rows: [x] native field builder and `connect_with`; [x] explicit Disabled/Required/VerifyCa/VerifyIdentity mappings; [x] retain maximum 5 / acquire timeout 10 seconds; [x] remove obsolete URL helper/caller/assertions/config scaffold; [x] public identity/authentication/TLS regression cases; [x] adjacent PostgreSQL/SQL Server source audit; [x] fixture and selectors documented.
- Remaining acceptance evidence: [ ] parent baseline/candidate actual public API probes; [ ] activated live regression execution; [ ] parent gates, review and individual publication/closure. These are parent-owned, not unfinished worker edits.
- Findings: baseline P1 credential/database parsing boundary at `mysql/connector.rs:177-202` on `bceb735727f81180dad678defee1000bc547d97b`. Replacement removes URL construction. No runtime/compiled correctness claim is made by this worker.
- Tests already run: **NOT RUN**. Counts and exit statuses: n/a. Assignment explicitly forbids worker commands/tests/builds/lint/formatters.
- Fixture update: parent reports MySQL 8.4 removed `--skip-ssl`; initial fixture exited, then was restarted using `--tls-version=` (empty value). Test comment and findings use the supported flag. No production TLS-policy change.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Baseline `bceb735727f81180dad678defee1000bc547d97b`; candidate patch uncommitted, parent assigns final SHA. |
| Commit list | None — worker commits/push prohibited. |
| File / surface inventory | `crates/infrastructure/src/mysql/connector.rs`: field-based options in shared `open_pool`; `crates/infrastructure/src/mysql/tests/connector_tests.rs`: delete obsolete URL-only test and its unused config fixture; `crates/infrastructure/tests/mysql_integration.rs`: three read-only public-provider cases and separate-field env fixture; `docs/plans/active/backend-four-issue-wave/MYSQL_FINDINGS.md`: source audit, versioned references, fixture/selector contract; this evidence file. |
| Acceptance mapping | Reserved/Unicode fields → `mysql_reserved_fields_preserve_authenticated_user_and_database`; wrong-password connect/test_connection → `mysql_reserved_fields_still_reject_wrong_password`; all strict modes reject no-TLS fixture → `mysql_strict_ssl_modes_reject_a_server_without_tls`; distinct CA/identity semantics → explicit mapping backed by SQLx 0.8.6 source audit. |
| Commands and counts | Every build/test/fmt/check/Clippy/release gate **NOT RUN**; no pass/fail/ignored counts or exit status fabricated. Exact parent commands below. |
| CI run IDs / status | NOT RUN by worker. |
| Known limitations | Negative TLS fixture proves strict encryption does not downgrade, not positive CA/hostname validation. Absent activation env prints skip and is not runtime evidence. PostgreSQL/SQL Server finding is source-only. |
| Migrations / config implications | No persisted schema/config migration or new dependency. Live tests require six separate `DB_PRO_MYSQL_452_*` fields; production configuration contract unchanged. |
| Out-of-scope changes | None. |

Parent selectors (all six fixture fields exported, no URL encoding):

```sh
cargo test -p db-pro-infrastructure --test mysql_integration mysql_reserved_fields_preserve_authenticated_user_and_database -- --ignored --exact
cargo test -p db-pro-infrastructure --test mysql_integration mysql_reserved_fields_still_reject_wrong_password -- --ignored --exact
cargo test -p db-pro-infrastructure --test mysql_integration mysql_strict_ssl_modes_reject_a_server_without_tls -- --ignored --exact
```

Fixture: disposable MySQL 8.4.11 on `127.0.0.1:25433`, `--tls-version=` with secure-transport enforcement off, dedicated `%` SQL-authenticated user and database. Required env suffixes: `HOST`, `PORT`, `USERNAME`, `PASSWORD`, `DATABASE`, `CURRENT_USER` under `DB_PRO_MYSQL_452_` (six fields; password remains external). Username/password include `# @ : / ? %` and Unicode; database includes legal reserved characters `# @ : ? %` and Unicode (MySQL forbids slash in database identifiers). `CURRENT_USER` must exactly match the dedicated username plus `@%`. No test mutates database state; successful pools are closed. Parent owns fixture account/schema/container/volume removal after verification. Full contract and constraints: `MYSQL_FINDINGS.md`.

Gate disposition: baseline red smoke **NOT RUN**; candidate smoke **NOT RUN**; three provider regressions **NOT RUN**; adjacent unit suite **NOT RUN**; formatting **NOT RUN**; build/check **NOT RUN**; Clippy **NOT RUN**; workspace tests **NOT RUN**; locked release build **NOT RUN**. Parent runs these once integration stabilizes.

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | n/a — no independent review by this worker; baseline source examined at `bceb735727f81180dad678defee1000bc547d97b`. |
| Verdict | n/a — READY for parent review, not ACCEPT or Done. |
| P0 / P1 / P2 counts | Independent candidate assessment NOT RUN; inherited #452 unsafe URL boundary recorded above, not treated as a newly introduced finding. |
| Findings | No independent review findings asserted; parent must validate candidate behavior and gate results. |
| CI disposition | NOT RUN by worker. |
| Next tasks unblocked | Parent fixture/probes, integrated verification, #452 commit/push and closure if acceptance succeeds. |

## 5. Research / audit handoff

- Source date: 2026-10-16.
- Repository source references at `bceb735727f81180dad678defee1000bc547d97b`: `Cargo.lock:4922-4925,5013-5016`; MySQL `connector.rs:177-232` and adjacent `tests/connector_tests.rs:186-202`; PostgreSQL `connection_string.rs:5-19`, `connector.rs:322-330,355-363`; SQL Server `connector.rs:146-180,185-186,199-201`; existing `tests/mysql_integration.rs` conventions. Canonical wave PLAN/requirements/context/dod/tasks and `_template/AGENT_EVIDENCE.md` were read.
- Installed official SQLx source references: `/home/vietis/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sqlx-mysql-0.8.6/src/options/mod.rs:117-177`, `options/ssl_mode.rs:8-31`, `connection/tls.rs:24-69`; sibling `sqlx-core-0.8.6/src/pool/options.rs:528-557`.
- Versioned API references: https://docs.rs/sqlx/0.8.6/sqlx/mysql/struct.MySqlConnectOptions.html and https://docs.rs/sqlx/0.8.6/sqlx/mysql/enum.MySqlSslMode.html . API decisions verified against installed source, not a fetched web page. Parent-supplied MySQL 8.4 TLS-flag reference: https://dev.mysql.com/doc/refman/8.4/en/encrypted-connection-protocols-ciphers.html .
- Factual source findings: MySQL baseline formats a URL before SQLx parsing; native setters preserve separate fields; SQLx typed pool opening enforces its acquire deadline; strict SSL variants reject servers without TLS and preserve distinct CA/hostname checks. PostgreSQL already uses `PgConnectOptions` and `connect_with`; SQL Server already uses Tiberius `Config`/`AuthMethod`, so neither setup shares this URL-interpolation defect. No unrelated provider edits.
- [INFERENCE] Reserved/Unicode live credentials should now authenticate unchanged; parent must establish actual baseline failure/candidate success rather than treating this inference as PASS.
- Decision: direct existing SQLx builders, not percent-encoding, URL helpers, new validation or an options abstraction. Tests use separate environment fields to avoid the old integration URL parser.
- Unresolved questions: no implementation decision pending. Actual fixture/gate observations remain parent-owned.
- Downstream tasks activated: #452 integration/review/publication; no new issue requested.

## 6. Tổng kết (Vietnamese summary)

Đã hoàn tất phần sửa #452: dùng `MySqlConnectOptions`/`connect_with`, giữ nguyên pool/timeout và bốn ánh xạ SSL; xóa helper cùng kiểm thử URL lỗi thời. Ba ca kiểm thử DB thật kiểm tra danh tính, từ chối sai mật khẩu và từ chối hạ cấp TLS; fixture dùng `--tls-version=` cho MySQL 8.4, không dùng cờ đã bị loại bỏ. Kiểm tra mã nguồn cho thấy PostgreSQL/SQL Server không có cùng lỗi. Trạng thái READY; mọi gate **NOT RUN** theo phân công, parent cần chạy bằng chứng baseline/candidate, review rồi mới commit/push/đóng issue.
