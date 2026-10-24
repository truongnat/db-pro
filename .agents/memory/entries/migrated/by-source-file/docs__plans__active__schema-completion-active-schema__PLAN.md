---
schema_version: 1
id: "MEM-SRC-ec85f07416377b3a"
title: "Source mirror: docs/plans/active/schema-completion-active-schema/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/schema-completion-active-schema/PLAN.md"
source_sha256: "ec85f07416377b3aabe8a510065db5106b760fddd90fd407ab88fa2c1afc6a5c"
mirror_body_sha256: "ec85f07416377b3aabe8a510065db5106b760fddd90fd407ab88fa2c1afc6a5c"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
