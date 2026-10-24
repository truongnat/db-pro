---
schema_version: 1
id: "MEM-SRC-1bfeda7a0f9fc14f"
title: "Source mirror: docs/plans/active/backend-four-issue-wave/MYSQL_FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/backend-four-issue-wave/MYSQL_FINDINGS.md"
source_sha256: "1bfeda7a0f9fc14f69edb1e635e5e3db4388f150d97bb9c45ef95c74331f3c60"
mirror_body_sha256: "1bfeda7a0f9fc14f69edb1e635e5e3db4388f150d97bb9c45ef95c74331f3c60"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# MySQL #452 findings

State: READY for parent integration and independent verification, not runtime-verified by this worker.
Baseline: `bceb735727f81180dad678defee1000bc547d97b`. Candidate changes are uncommitted edits based on that SHA; publication SHA belongs to the parent.

## Scope and observed defect

At the baseline, `crates/infrastructure/src/mysql/connector.rs:177-202` interpolates username, password, host and database into `mysql://...` before SQLx parses it. `connect` and `test_connection` both use this `open_pool`. Reserved delimiters therefore cross the URL syntax boundary rather than remaining authentication/database fields. Source establishes the unsafe construction; the parent owns actual baseline reproduction from `/tmp/db-pro-wave140-baseline` and candidate runtime evidence.

Assumptions: preserve existing pool limits, timeout and all four SSL policies; no new certificate-path support, authentication policy or validation. Configured strings should reach the driver unchanged, not be percent-encoded into a new URL.

## Implemented cutover

- `open_pool` now builds `MySqlConnectOptions` directly with host, port, username, password and database setters, then uses `MySqlPoolOptions::connect_with`.
- Explicit mapping remains Disable → `Disabled`, Require → `Required`, VerifyCa → `VerifyCa`, VerifyFull → `VerifyIdentity`. Pool maximum remains 5 and acquire timeout remains 10 seconds.
- Removed `mysql_connection_url`, its sole production caller, URL-string assertions and the now-unused unit-test config helper. No replacement getter/default-wiring/source-inspection assertions were added. Existing unrelated transaction/cancellation tests are unchanged.
- Live tests exercise public `DbConnector::connect`, `query`, `disconnect` and `test_connection`. Identity is checked against actual `CURRENT_USER()` and `DATABASE()` values; wrong credentials must yield the server authentication error code 1045, not merely any connection error. Error output must not contain the submitted wrong password.
- Production never serializes or logs options/passwords; existing connection-error wrapping is unchanged.

## Installed SQLx 0.8.6 source checks

`Cargo.lock:4922-4925,5013-5016` locks SQLx and sqlx-mysql to 0.8.6. Read installed registry sources under `/home/vietis/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`:

- `sqlx-mysql-0.8.6/src/options/mod.rs:117-177`: setters store strings as fields and explicitly support selecting the SSL mode. Defaults otherwise prefer TLS with plaintext fallback, so retaining the explicit mapping is necessary.
- `sqlx-core-0.8.6/src/pool/options.rs:528-557`: `connect(url)` parses the URL, whereas `connect_with(options)` consumes typed options and immediately opens a connection within the acquire deadline.
- `sqlx-mysql-0.8.6/src/options/ssl_mode.rs:8-31`: documents all four selected policies.
- `sqlx-mysql-0.8.6/src/connection/tls.rs:24-69`: Disabled avoids TLS; Required/VerifyCa/VerifyIdentity reject a server without TLS. VerifyCa and VerifyIdentity validate certificates, with hostname checks additionally enabled only for VerifyIdentity.

Versioned reference URLs: https://docs.rs/sqlx/0.8.6/sqlx/mysql/struct.MySqlConnectOptions.html and https://docs.rs/sqlx/0.8.6/sqlx/mysql/enum.MySqlSslMode.html (API decisions verified against the installed source above, not a fetched web page).

## Adjacent-provider source audit

At baseline `bceb735727f81180dad678defee1000bc547d97b`:

- PostgreSQL does **not** share this URL-interpolation defect. `crates/infrastructure/src/postgres/connection_string.rs:5-19` sets separate `PgConnectOptions` fields. Both public paths consume those options via `connect_with` in `postgres/connector.rs:322-330,355-363`.
- SQL Server does **not** share this URL-interpolation defect. `crates/infrastructure/src/sqlserver/connector.rs:146-166` builds Tiberius `Config` with separate host/port/database and `AuthMethod::sql_server(username, password)` fields. `connect_client` consumes that config directly, and both public connection paths share it (`:169-180,185-186,199-201`).
- This is a bounded source audit, not a claim of live PostgreSQL/SQL Server verification or broader credential/security correctness. Neither provider was edited.

## Fixture contract and isolation

The parent owns a disposable MySQL **8.4.11** fixture bound to loopback port **25433**, started with `--tls-version=` (empty value); secure-transport enforcement must be off for Disable to succeed. MySQL 8.4 removed `--skip-ssl`: the parent reported that the first container exited and restarted the fixture with the supported empty TLS-version setting. Official reference supplied by parent: https://dev.mysql.com/doc/refman/8.4/en/encrypted-connection-protocols-ciphers.html . Provision a dedicated SQL-authenticated account with host `%` and a dedicated database; grant only the access needed to select/use that database. No production data or credentials.

Export these separate environment fields without URL encoding:

| Variable | Required value/meaning |
|---|---|
| `DB_PRO_MYSQL_452_HOST` | `127.0.0.1`; activates the dedicated cases |
| `DB_PRO_MYSQL_452_PORT` | `25433` for this parent fixture |
| `DB_PRO_MYSQL_452_USERNAME` | Exact fixture username containing `#`, `@`, `:`, `/`, `?`, `%` and non-ASCII characters |
| `DB_PRO_MYSQL_452_PASSWORD` | Exact fixture password containing the same reserved characters and non-ASCII characters; keep it external to source/docs/logs |
| `DB_PRO_MYSQL_452_DATABASE` | Exact fixture database containing `#`, `@`, `:`, `?`, `%` and non-ASCII characters |
| `DB_PRO_MYSQL_452_CURRENT_USER` | Exact expected authenticated account identity, username followed by `@%` for the dedicated `%` account |

MySQL prohibits `/`, backslash and period in database names; slash coverage therefore belongs to username/password, not an invalid database fixture. SQL delimiters/identifiers in fixture provisioning must be quoted correctly by the parent's provisioning mechanism. Do not use `DATABASE_URL` for these cases: the existing integration helper itself manually splits URLs and cannot represent this fixture safely.

Cases are ignored/provider-opt-in, following this integration file's conventions. If the activation variable is absent, they print a skip reason and return; this is **not** provider evidence. Once activated, every other field is required and malformed/missing fields fail. The parent must record that the fields were supplied when claiming PASS. The cases perform only SELECT/authentication probes, close successful pools before assertions where possible, and create/drop no objects. They can run concurrently without changing shared fixture state. Parent removes the dedicated account/database or disposable container/volume after integration verification; never point them at a shared production server.

## Exact verification selectors — NOT RUN by worker

From the repository root with all six fixture fields exported:

```sh
cargo test -p db-pro-infrastructure --test mysql_integration mysql_reserved_fields_preserve_authenticated_user_and_database -- --ignored --exact
cargo test -p db-pro-infrastructure --test mysql_integration mysql_reserved_fields_still_reject_wrong_password -- --ignored --exact
cargo test -p db-pro-infrastructure --test mysql_integration mysql_strict_ssl_modes_reject_a_server_without_tls -- --ignored --exact
```

Expected behavior, **not observed by this worker**: identity/test_connection succeed for exact reserved/Unicode values; connect and test_connection both reject a wrong password with 1045; all three strict modes reject the TLS-disabled server while Disable succeeds. The negative TLS fixture distinguishes strict encryption from plaintext fallback, but does not independently prove the difference between CA and hostname verification; that distinction is preserved by the explicit mapping and checked installed SQLx implementation.

Parent also owns independent before/after public-API probes against the immutable baseline/candidate, relevant adjacent unit tests, formatting, build/check, Clippy, workspace tests, release build, review, commits/push and issue closure. Every gate is **NOT RUN** by this worker per assignment. No shell commands, builds, tests, linters or formatters were run by this worker.

## Tổng kết bằng tiếng Việt

Đã thay URL nội suy bằng cấu hình MySQL có kiểu, giữ nguyên giới hạn pool/timeout và bốn chế độ SSL; xóa helper và kiểm thử chuỗi URL lỗi thời. Đã bổ sung ba ca kiểm thử DB thật cho danh tính, sai mật khẩu và chống hạ cấp TLS. PostgreSQL/SQL Server không có cùng lỗi nội suy theo kiểm tra mã nguồn. Trạng thái READY để parent chạy fixture, đối chiếu baseline/candidate và thực hiện toàn bộ gate; chưa tuyên bố PASS runtime hay đóng issue.
