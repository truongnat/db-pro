---
schema_version: 1
id: "MEM-SRC-ed08fcdd6cc00a3f"
title: "Source mirror: docs/plans/active/postgres-typed-query-description/AGENT_EVIDENCE.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/postgres-typed-query-description/AGENT_EVIDENCE.md"
source_sha256: "ed08fcdd6cc00a3f32f623e00ec834a8635f239c9359760cfd879aae9821ae82"
mirror_body_sha256: "ed08fcdd6cc00a3f32f623e00ec834a8635f239c9359760cfd879aae9821ae82"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Agent evidence — #533 PostgreSQL typed query description

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | PostgresTypedDescribe / implementation worker |
| Issue(s) | #533 — inherited P1 prerequisite discovered during #441/#442 runtime verification |
| Task state | Review |
| Baseline SHA | 76b120724a7542de8b587a002811ddacad28156a |
| Branch / PR | shared main; Main alone integrates, publishes and closes |
| Scope interpretation | Align query description and actual argument wire types without duplicating conversion or disabling caches. Add real PostgreSQL regressions and an external public runtime smoke recipe. |
| Out of scope | schema DDL, other-provider introspection, core data diff, ports, unrelated sessions, shared manifests/changelog/status |

## 2. Progress checkpoint

- Exact claimed baseline: `76b120724a7542de8b587a002811ddacad28156a`. No git command was run to rediscover shared HEAD. Candidate is uncommitted; Main must supply final SHA and account for independent shared-main changes.
- Completed implementation acceptance: opt-in capture of actual converted non-null argument wire types and unspecified hints for untyped NULL; public prepare_with before describe and streaming; nullable/empty metadata without tail draining; bounded retained rows; permanent INTEGER composite-PK public-service integration regression; BIGINT extrema; repeated same-typed SQL; execute/explain coverage; slow-tail regression; contextual NULL regression; captured-type unit test; standalone DbProRuntime smoke source/recipe.
- Remaining acceptance: actual candidate build/test/format/lint/provider execution and independent review are intentionally assigned to Main. No worker verification was authorized.
- Inherited P1: baseline `crates/infrastructure/src/postgres/connector.rs:376-381` describes without actual parameter types before binding binary i64. Parent-observed error and command preserved in VERIFICATION.md; separately tracked as #533, not attributed to candidate #441/#442.
- Tests already run: none by worker; commands/counts/exit statuses n/a. Parent inherited reproduction exited 1 as reported. Parent also reported an in-progress clippy compile failure E0309 in bind_value; worker added the required `T: 'q` bound without rerunning. Exact parent gate command/counts not supplied; final revised gates remain pending.
- Parent additionally observed the pre-NULL-fix live-PG run: 4 passed / 1 failed / 0 ignored; contextual NULL regression failed with QuerySyntaxAt integer = text, position 10 (bg51, parent artifact://147). Worker accepted this as ground truth and did not rerun; OID 0 fix still awaits Main verification.
- Dependency change: none. SQLx 0.8.6 official API and installed implementation inspected; ordinary statement caching retained.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Baseline 76b120724a7542de8b587a002811ddacad28156a; candidate uncommitted, final integration SHA pending Main |
| Commit list | n/a; worker does not commit |
| File / surface inventory | PostgreSQL connector.rs: typed prepare/describe/bounded-fetch helper; query_mapper.rs: opt-in type capture with unspecified NULL; adjacent mapper tests; pg_integration.rs: five opt-in real-PG regressions; canonical plan/evidence records. Temporary smoke source removed after proof. |
| Acceptance mapping | INTEGER PK/metadata/precision/execute/explain, row limits, capped slow tail and contextual NULL map to five real-PG tests; type hints map to mapper unit case; public DbProRuntime original INTEGER probe observed in VERIFICATION.md. |
| Commands and counts | No builds/tests/lint/format/smoke executed by worker; no pass counts claimed. Suggested focused command in VERIFICATION.md. |
| CI run IDs / status | not run |
| Known limitations | SQLx's existing SQL-only cache still cannot safely change binary parameter types for identical SQL. Candidate does not claim otherwise. Live verification pending. |
| Migrations / config implications | none; DATABASE_URL for opt-in existing-style integration tests; smoke uses disposable PROBE_STATE and local-only fixture credentials |
| Out-of-scope changes | none |

## 4. Review outcome

Independent review rejected the prior uncommitted execution-before-describe candidate with one introduced P2: Describe synchronously drained the unused tail after reaching max_rows. Revised prepare_with-before-execution design resolves that finding in source. Follow-up review identified another P2: untyped NULL captured as TEXT; worker corrected it to unspecified OID 0 and added regression coverage. Independent final re-review requested. Inherited P1 count: 1 (#533). No accepted final SHA or final verdict is available. Main must run gates, obtain revised review, pin the SHA and decide closure.

## 5. Research / audit handoff

- Source date: 2026-10-16.
- Source references: baseline connector.rs:218,344-469,689,814-819,932-979; query_mapper.rs:10-64,305-329; core TableDataService:103-120; Cargo.lock SQLx 0.8.6. Local installed SQLx executor.rs:168-248,445-485; arguments.rs:83-103; types/int.rs:134-151.
- Official API: https://docs.rs/sqlx/0.8.6/sqlx/trait.Executor.html#tymethod.prepare_with . Implementation references and root-cause chain are listed in FINDINGS.md.
- Factual finding: parent observed INTEGER/Int64 public-service failure; installed SQLx caches by SQL, describes with empty parameter hints, and normally executes with actual argument types.
- Inference: the INT4/i64 wire mismatch is the cause, grounded in that source chain and parent observation; the candidate remedy still awaits runtime proof.
- Decision: opt-in generic capture of actual converted non-null argument types, unspecified Parse OID 0 for domain NULL, then prepare_with before describe and streaming. Earlier execution-before-describe design was rejected for synchronous tail draining; a later TEXT-hint-for-NULL regression was corrected to preserve contextual inference. Other two describe callsites lack typed parameters; execute/explain keep the non-capturing bind path and are unchanged.
- Unresolved questions: none in implementation scope; final integrated SHA and verification outcomes pending Main.
- Downstream task activated: Main verification/review/closure of #533; then original #441/#442 runtime acceptance.

## 6. Tổng kết

Đã dùng generic helper để thu kiểu tham số không-null theo chuyển đổi hiện có; NULL không định kiểu giữ OID 0 để PostgreSQL suy luận theo ngữ cảnh. prepare_with và describe đều chạy trước streaming, không chờ slow tail sau max_rows. Hai P2 do reviewer phát hiện đã được xử lý ở nguồn: drain kết quả và hint TEXT cho NULL. Có tổng cộng 5 integration test PostgreSQL thật, unit test capture kiểu và smoke DbProRuntime độc lập. P1 kế thừa ghi riêng ở #533. Worker chưa chạy gates; Main cần chạy ứng viên mới, ghim SHA và lấy review cuối. Việc đổi kiểu binary cho SQL đã cache vẫn ngoài phạm vi.
