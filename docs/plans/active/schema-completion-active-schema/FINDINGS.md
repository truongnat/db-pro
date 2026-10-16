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
