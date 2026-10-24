---
schema_version: 1
id: "MEM-PLAN-schema-completion-active-schema-d7bfa222f87a"
title: "Plan archive: Schema Completion Active Schema"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, schema-completion-active-schema]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "d7bfa222f87a68cd551f426a2fe1a3785b13dbc64d7d7a652853d81c1f7ace13"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/schema-completion-active-schema/PLAN.md"
  - "docs/plans/active/schema-completion-active-schema/FINDINGS.md"
  - "docs/plans/active/schema-completion-active-schema/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/schema-completion-active-schema/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# #511 — Schema completion prefers active_schema for duplicate table names

## Goal
`SELECT ... FROM users u` với `active_schema=tenant_b` phải gợi ý cột của
`tenant_b.users`, không phải `tenant_a.users` (đứng trước trong catalog order).

## Scope
- Helper mới `find_completion_table(tables, name, active_schema)` trong
  `crates/ui/src/query/schema_completion.rs`: match `name` + `schema == active_schema`
  trước, fallback `find` cũ (giữ hành vi khi active schema không có bảng đó).
- Thay 3 call-site `.find(|t| t.name.eq_ignore_ascii_case(..))`:
  1. `provide()` col frequencies (~L440),
  2. `provide()` column suggestions (~L454),
  3. `build_ai_sql_context()` relevant_columns (~L1045).
- Không đổi label qualification, ranking, hay scope parsing.

## Non-goals
- `consume_qualified` drop schema prefix (`completion_scope.rs:380-391`) — scope
  `referenced_tables` chỉ chứa bare name; giữ nguyên, fix ở tầng lookup là đủ
  cho kịch bản trong issue. Ghi nhận follow-up riêng nếu cần qualified refs.
- Qualified label khi ambiguous — machinery hiện có đã qualify khi 2+ bảng
  *resolved* chia sẻ cột; sau fix single-resolution nên không còn false ambiguity.

## Architecture
- `CompletionContext.active_schema: &str` đã có sẵn ở cả 2 site trong `provide()`;
  `build_ai_sql_context(doc, cursor, active_schema, summary)` đã nhận param.
- `UiTableSummary { schema, name, .. }` (`runtime_schema_types.rs:17`).

## Acceptance
- tenant_a.users + tenant_b.users (tenant_a đứng trước), active=tenant_b,
  `SELECT  FROM users u` → items chứa cột tenant_b, không chứa cột tenant_a.
- active=tenant_c (không có users) → fallback match đầu (tenant_a), giữ hành vi cũ.
- `build_ai_sql_context` tương tự cho `relevant_columns`.


## Source: `FINDINGS.md`

# #511 Findings

## Evidence (source)
- `crates/ui/src/query/schema_completion.rs:440-444` (frequencies) và `:454-458`
  (suggestions): `.find(|t| t.name.eq_ignore_ascii_case(t_name))` — catalog order,
  không xét `ctx.active_schema` (dù struct đã có field `:48`).
- `:1045-1048` (`build_ai_sql_context`): cùng pattern cho `relevant_columns`
  (AI context cũng gợi ý cột sai schema).
- `completion_scope.rs:380-391` (`consume_qualified`): `FROM tenant_b.users` vẫn
  cho `referenced_tables=["users"]` (drop prefix) → lookup bare-name là điểm
  duy nhất cần fix cho kịch bản issue.

## Failure scenario
Catalog `[tenant_a.users(id, tenant_a_only), tenant_b.users(id, tenant_b_only)]`,
`active_schema=tenant_b`, SQL `SELECT <cursor> FROM users u` → resolve ra
`tenant_a.users` → dropdown chứa `tenant_a_only` (không tồn tại ở schema đang chạy);
user insert → query lỗi ở runtime.

## Severity
P2 (theo issue): gợi ý sai, không mất/sai dữ liệu.

## Scope decision
Ưu tiên `schema == active_schema` (case-insensitive) rồi fallback find cũ —
giữ hành vi khi bảng chỉ tồn tại ở schema khác hoặc active_schema rỗng.

## Provider impact
Cả PostgreSQL (multi-schema) và SQLite (`main` schema): cùng code path,
cùng hành vi (prefer-then-fallback). SQLite thường 1 bảng/trùng tên → fallback.

## Runtime testability
Unit test `provide()` + `build_ai_sql_context()` với summary 2-schema, không cần DB.


## Source: `VERIFICATION.md`

# #511 Verification (append-only)

## Fail-before (chưa fix, đã có test mới)
- `cargo test -p db-pro-ui --lib 511` → **2 failed / 1 passed**:
  - `completion_prefers_active_schema_for_duplicate_table_511` FAILED —
    gợi ý `["id", "tenant_a_only", ...]`, thiếu `tenant_b_only` (đúng bug issue).
  - `ai_context_prefers_active_schema_for_duplicate_table_511` FAILED (cùng nguyên nhân).
  - `completion_falls_back_when_active_schema_has_no_match_511` passed (guard giữ hành vi cũ).

## Pass-after (sau helper `find_completion_table` + 3 call-site)
- `cargo test -p db-pro-ui --lib 511` → **3/3 PASS**.
- `cargo test -p db-pro-ui --lib query` → **89 passed / 0 failed**.
- `cargo +nightly fmt --all -- --check` → exit 0 (đã `fmt` 1 lần cho `if let` mới).
- `cargo clippy -p db-pro-ui --all-targets` → không warning nào trong `schema_completion`.
- `python3 tools/cc-scan.py schema_completion.rs schema_completion_tests.rs --fail-on error` → **100/100, error=0**.
- `cargo check -p db-pro-ui` → Finished OK.

## Chưa làm (khai báo trung thực)
- `consume_qualified` drop schema prefix (`completion_scope.rs:380-391`) giữ nguyên —
  `FROM tenant_b.users` vẫn resolve qua bare name + active_schema; qualified-ref
  triệt để là follow-up riêng.
- Không chạy full `cargo test --workspace`; không có screenshot dropdown completion.
- Issue #511 để OPEN chờ owner quyết định.

