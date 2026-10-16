# Agent evidence — provider-correct generated SQL (#441 / #442)

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | ProviderSqlDialect — worker slice |
| Issues | #441, #442 |
| Task state | Review (implementation complete; gates delegated) |
| Baseline SHA | `76b120724a7542de8b587a002811ddacad28156a` — baseline source observations in FINDINGS.md |
| Branch / PR | shared `main`; no PR; parent alone commits/pushes/closes |
| Scope interpretation | PK reload SQL and provider-correct reconstructed table DDL, adjacent core tests and this plan directory |
| Out of scope | dialect ports, data_diff, manifests/dependencies, shared changelog/status, trigger redesign, expression/identity syntax translation |

## 2. Progress checkpoint

- Current candidate: uncommitted worker patch layered on baseline `76b120724a7542de8b587a002811ddacad28156a`; no final commit SHA assigned by worker.
- Completed acceptance: implementation, composite PK SQL/ordered-value regression cases, public SchemaService special/reserved identifier output matrix, PG native-index/deferred-FK guard, independent real-provider probe recipe.
- Remaining acceptance: parent executes tests and public-service/provider smoke; independent review; final integration evidence and issue closure.
- Findings/risks: inherited P1 #441 and P2 #442 plus related schema-qualified index/missing MySQL-SQL Server FK paths documented with baseline anchors in FINDINGS.md. Proposed patch addresses them; no executed proof yet.
- Tests run: none. Build/test/lint/format/cargo/smoke execution prohibited for this worker.
- Prerequisites: parent is preparing live fixtures; no database availability/credentials claim by worker. LSP was reported unavailable in the assignment, so literal source navigation was used. Semantic find also failed due unavailable judge account; literal fallback used.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Baseline `76b120724a7542de8b587a002811ddacad28156a`; candidate is uncommitted (parent must supply exact final SHA) |
| Commit list | none by worker |
| File / surface inventory | `crates/core/src/application/sql_builder/mutation.rs` — preserve row cap via existing dialect pagination/order hooks; `driver_dialect.rs` — crate-visible typed constructor reused by existing name constructor; `schema_ddl.rs` — shared dialect quoting, inline non-PG FKs, valid index-name qualification, preserved native PG/trigger behavior; `schema_service.rs` — remove obsolete test-only quote import; `tests/sql_builder_tests.rs` — four-driver composite PK SQL/ordered parameters and malformed-key tests; `schema_service_tests.rs` — special/reserved identifier public-service matrix, native index/deferred composite FK guard, corrected reconstructed PG index expectation; this directory's PLAN/CHECKLIST/FINDINGS/VERIFICATION/AGENT_EVIDENCE |
| Expanded provider inventory | `crates/infrastructure/src/mysql/introspect.rs` — actual referenced schema and database/table-scoped catalog joins; `crates/infrastructure/src/sqlserver/introspect.rs` — canonical FK actions and removal of literal non-SQL comment text from FK query; `crates/infrastructure/src/sqlserver/tests/introspect_tests.rs` — catalog-action regression; `crates/infrastructure/tests/mysql_integration.rs` — cross-database SchemaService reconstruction and actual target-key enforcement; `crates/infrastructure/tests/sqlserver_integration.rs` — action metadata plus SchemaService reconstruction; `schema_ddl.rs`/`schema_service_tests.rs` additionally preserve MySQL supporting indexes inline before FK creation |
| Acceptance mapping | #441 → composite PK builder/value tests plus live `PROBE_CASE=pk`; #442 → public schema output matrix plus live `PROBE_CASE=ddl`, original SQLite/trigger suites retained |
| Commands and counts | NOT RUN; exact focused selectors and independent public-service probe are in VERIFICATION.md |
| CI run IDs / status | not run |
| Known limitations | no live-provider evidence; raw expressions/identity syntax remain provider-authored and untranslated; parent must execute and remove temporary probe source |
| Migrations / config implications | none; generated output changes, no persisted-format/schema migration |
| Out-of-scope changes | none |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | baseline `76b120724a7542de8b587a002811ddacad28156a`; first uncommitted DDL candidate blob `6a56b45d9a14ed06d4eec068a32fbc21ed2efe2e` reviewed by ReviewDataSql |
| Verdict | BLOCK on first candidate; corrections implemented, independent recheck pending |
| P0 / P1 / P2 counts | reviewer reported 0/0/2 introduced; parent upgraded cross-database wrong-target risk to P1 |
| Findings | SQL Server raw catalog action tokens and MySQL wrong referenced database; producer fixes and direct/runtime regression cases added, not executed by worker |
| CI disposition | all execution gates delegated, none reported passed |
| Next tasks unblocked | parent integration gates and separate provider-runtime verification for #441/#442 |

## 5. Research / audit handoff

- Source date: 2026-10-16.
- Source references: baseline `76b120724a7542de8b587a002811ddacad28156a`, `crates/core/src/application/{driver_dialect.rs,schema_ddl.rs,schema_service.rs,table_data_service.rs,sql_builder.rs,sql_builder/read.rs,sql_builder/mutation.rs}`, adjacent test files, `docs/architecture/provider-contract.md`, canonical lifecycle/evidence templates. Complete external links and the exact baseline source anchors are in FINDINGS.md.
- Factual findings: baseline hardcoded reload LIMIT and DDL double quotes; DDL reconstruction has a DriverType; shared DriverDialect already implements provider escaping; reconstructed index names are qualified; MySQL/SQL Server FKs are omitted.
- Inference: core output tests cannot establish live-provider acceptance. Existing validation does not establish catalog uniqueness; retaining the row cap avoids introducing that assumption.
- Decision: reuse existing quoting and dialect pagination/order hooks without extending ports; retain provider-specific DDL representation and native SQL bodies. After independent review, normalize SQL Server action metadata, preserve actual MySQL referenced databases with scoped joins, and declare MySQL supporting indexes before FKs. Parent owns final review/runtime decision.
- Unresolved questions: no implementation question; live provider availability/TLS/configuration and execution results are pending parent.
- Downstream tasks: parent verifies #441 and #442 before closure.

## 6. Tổng kết

Đã sửa SQL tải lại hàng theo khóa chính để không phát `LIMIT` không hợp lệ trên SQL Server; DDL dùng chung `DriverDialect` để quote/escape đúng cho bốn provider, bỏ schema khỏi tên index và giữ FK của MySQL/SQL Server. Đã thêm test cho PK ghép, thứ tự tham số, định danh đặc biệt, CHECK/FK/index và giữ hành vi trigger. Chưa chạy bất kỳ gate hay provider thật nào theo phân công; cha cần chạy các selector và probe public-service trong VERIFICATION.md, ghi SHA/kết quả thật rồi mới đóng issue. Không sửa port, manifest, data_diff hoặc file dùng chung.

Review follow-up: Đã sửa hợp đồng metadata FK: SQL Server chuyển action có dấu gạch dưới sang cú pháp SQL chuẩn; MySQL giữ đúng database đích và giới hạn JOIN catalog theo database/bảng/constraint. Index MySQL được khai báo trước FK để tránh trùng index hỗ trợ. Đã thêm unit test và hai test provider thật có ignore; chưa chạy gate nào, cha cần chạy lại review và smoke cross-database.
