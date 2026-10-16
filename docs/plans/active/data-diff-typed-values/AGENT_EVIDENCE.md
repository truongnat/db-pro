# Agent evidence — typed data-diff values (#457 / #485)

## 1. Claim
| Field | Value |
|---|---|
| Agent identity | TypedDataDiff / core data-diff lane |
| Issue(s) | #457, #485 |
| Task state | Review |
| Baseline SHA | `76b120724a7542de8b587a002811ddacad28156a` |
| Branch / PR | main / no PR; Main owns integration and publication |
| Scope interpretation | Replace lossy NULL/text and composite-key identity in DataDiffService; generate scalar sync-preview SQL from typed rows. |
| Out of scope | SQL builders, schema DDL, ports, new dependencies, unrelated sampling/provider support, binary literal implementation, MySQL backslash modes, shared changelog/status. |

## 2. Progress checkpoint
- Current integration base: `76b120724a7542de8b587a002811ddacad28156a`; lane changes are uncommitted. No final candidate SHA exists in this lane.
- Implementation acceptance: [x] NULL/text comparison, [x] distinct NULL/text keys, [x] IS NULL predicates, [x] text `∅` INSERT/UPDATE, [x] structural U+001F tuples and exact WHERE boundaries, [x] adjacent regressions and independent provider probes.
- Remaining: [ ] executed focused tests, [ ] SQLite runtime evidence, [ ] PostgreSQL runtime evidence, [ ] integration review/gates/commit/closure.
- Baseline P1 findings at `76b120724a7542de8b587a002811ddacad28156a`: `crates/core/src/application/data_diff.rs:266-290,310-345`; details in FINDINGS.md. Revised implementation not runtime-verified.
- Tests already run: none (explicit lane prohibition). No build/lint/format/cargo execution.
- Dependencies/blockers: parent reported dedicated PostgreSQL fixture and second database provisioning; availability not observed here. No external code changes required.

## 3. Implementation handoff / review request
| Field | Value |
|---|---|
| Exact SHA | Baseline `76b120724a7542de8b587a002811ddacad28156a` + uncommitted lane changes; final SHA must be supplied by Main. |
| Commit list | none |
| File / surface inventory | `crates/core/src/application/data_diff.rs` — borrowed typed values/keys, null predicates, source-row SQL, display-only strings; `crates/core/src/application/tests/data_diff_tests.rs` — updated private typed fixture plus six public-service regressions; `crates/infrastructure/tests/data_diff_smoke.rs` — real SQLite and independent ignored PostgreSQL public-service round trips; this directory's PLAN.md, CHECKLIST.md, FINDINGS.md, VERIFICATION.md and AGENT_EVIDENCE.md — requirements, source evidence and handoff. |
| Acceptance mapping | NULL/text and NULL predicates → null_source/null_keys/empty_set tests; text literal preservation → empty_set test; key collision and intended WHERE → two composite tests; precision/quoting → typed_values test; existing behavior + real SQL execution → existing tests and provider round trips. |
| Commands and counts | No commands executed; exact selectors/recipes in VERIFICATION.md. Counts unavailable, exit status n/a. |
| CI run IDs / status | not run |
| Known limitations | Provider/automated execution unverified; binary literal correctness and MySQL backslash handling are inherited/out of scope. DTO NULL now uses None; keys are escaped typed display strings, never SQL identity. |
| Migrations / config implications | No persisted/public DTO type migration. PostgreSQL smoke requires two distinct disposable writable databases plus PG* credentials and DATA_DIFF_TARGET_DATABASE. |
| Out-of-scope changes | none |

## 4. Review outcome
| Field | Value |
|---|---|
| Reviewed SHA | n/a; no independent review performed |
| Verdict | n/a; request Main review |
| P0 / P1 / P2 counts | Not adjudicated by this lane; baseline findings recorded separately. |
| Findings | Baseline P1 NULL/key failures described in FINDINGS.md; candidate execution pending. |
| CI disposition | all execution gates not run in this lane |
| Next task(s) unblocked | Main's combined verification and provider smoke; close #457/#485 only after fully verified. |

## 5. Research / audit handoff
- Source date: 2026-10-16.
- Source references at baseline `76b120724a7542de8b587a002811ddacad28156a`: core application data_diff.rs and tests/data_diff_tests.rs; domain query.rs/cross_connection.rs; ports dialect.rs/db_connector.rs; runtime api.rs/worker.rs; app schema/schema_compare_view.rs; infrastructure postgres/connector.rs and existing integration/pg_integration.rs conventions.
- Factual findings: baseline string flattening, U+001F join/split and sentinel literal decoding are present; runtime/API forward DTOs and the UI displays keys/SQL. Exact pointers are in FINDINGS.md.
- Inference: baseline public-provider probe should fail on duplicate composite key; independent NULL-only fixture should falsely report equality. Neither reproduction was executed here.
- Decision: borrow typed values internally, use structural tuples, generate SQL from typed source/key data, retain DTO field types; leave provider-specific binary/backslash work untouched as authorized by Main.
- Unresolved questions: no implementation question; runtime/provider readiness and results belong to Main.
- Downstream tasks activated: Main integration/verification for #457/#485.

## 6. Tổng kết (Vietnamese summary)
Đã sửa cụm #457/#485 bằng giá trị có kiểu và khóa tuple: NULL không còn trùng chuỗi `∅`, khóa ghép chứa U+001F không bị gộp, điều kiện NULL dùng IS NULL, SQL lấy dữ liệu gốc thay vì giải mã chuỗi hiển thị. Giữ nguyên kiểu DTO; thêm 6 test hồi quy và smoke độc lập SQLite/PostgreSQL có chạy SQL preview rồi so sánh lại. Chưa chạy bất kỳ gate nào theo phân công; Main cần chạy test/provider smoke, ghi SHA và bằng chứng thật trước khi commit/push/đóng issue. Hành vi binary/MySQL backslash cũ không được mở rộng hoặc xác minh.
