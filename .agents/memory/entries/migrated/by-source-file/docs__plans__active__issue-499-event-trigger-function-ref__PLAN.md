---
schema_version: 1
id: "MEM-SRC-d251f58557b61567"
title: "Source mirror: docs/plans/active/issue-499-event-trigger-function-ref/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-499-event-trigger-function-ref/PLAN.md"
source_sha256: "d251f58557b61567a1683f3f5a5b565838a0bdaa1ec84800cdf74cc4d5d1f237"
mirror_body_sha256: "d251f58557b61567a1683f3f5a5b565838a0bdaa1ec84800cdf74cc4d5d1f237"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #499 — Event trigger function ref validation/normalization

## Goal
`EXECUTE FUNCTION {ref}` phải luôn emit `name()` hợp lệ cho PostgreSQL event
triggers: `on_ddl` → `EXECUTE FUNCTION on_ddl();`, `public.on_ddl` →
`EXECUTE FUNCTION public.on_ddl();`; `func(,)`, `func(a)`, `func()()` bị reject.

## Findings
- Fix đã land trong commit `55dc35a4` (`fix: commit temp`, wave
  `backend-four-issue-wave`): `validate_function_ref` bỏ char-whitelist cũ
  (cho phép `(`/`)`/`,`) → strict-parse `name` hoặc `name()` + `validate_ident`
  cho từng phần `schema.func`; `preview_create_event_trigger` lowercase + quote
  từng ident rồi emit `{function_name}()` đúng chuẩn `EXECUTE FUNCTION`.
- Không còn code nào để sửa; phần còn thiếu là test cho *output preview*
  (existing tests chỉ cover validate ok/err + alter/drop preview).

## Scope
- Thêm test `create_preview_normalizes_and_quotes_function_call` trong
  `crates/core/src/domain/tests/event_trigger_tests.rs` assert exact SQL.
- Không đổi `event_trigger.rs` (hành vi đã đúng acceptance).

## Acceptance
- Test mới pass: `on_ddl`, `on_ddl()`, `public.on_ddl`, `public.on_ddl()`,
  `PUBLIC.On_Ddl` → `EXECUTE FUNCTION "..."()` (identifier quoting là
  hardening đúng chuẩn repo — "quoted" thay vì literal `on_ddl()` trong issue).
- `func(,)`, `func(arg)`, `func()()` … đã reject (test
  `rejects_arguments_and_malformed_function_references` sẵn có).
