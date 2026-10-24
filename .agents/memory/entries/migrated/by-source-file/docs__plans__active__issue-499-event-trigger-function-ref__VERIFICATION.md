---
schema_version: 1
id: "MEM-SRC-ed8746f692f6b87d"
title: "Source mirror: docs/plans/active/issue-499-event-trigger-function-ref/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-499-event-trigger-function-ref/VERIFICATION.md"
source_sha256: "ed8746f692f6b87dccdfdad1a5c6239645b3bab60c83a094db6049607466aff6"
mirror_body_sha256: "ed8746f692f6b87dccdfdad1a5c6239645b3bab60c83a094db6049607466aff6"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #499 Verification

## Pass (2026-10-17)
- `cargo test -p db-pro-core event_trigger` → **6 passed / 0 failed**
  - `create_preview_normalizes_and_quotes_function_call` (mới): assert exact
    `CREATE EVENT TRIGGER "audit_ddl" ON ddl_command_end EXECUTE FUNCTION "on_ddl"();`
    cho `on_ddl`, `on_ddl()`, `public.on_ddl`, `public.on_ddl()`, `PUBLIC.On_Ddl`.
  - `rejects_arguments_and_malformed_function_references` (sẵn có): `func(,)`,
    `func(arg)`, `func(a,b)`, `func()()`, `func(())`, `func ()`, `func;DROP`,
    `func--comment`, `.on_ddl`, `public.`, `a.b.c`, `1func`, `()` … đều Err,
    cả ở `validate_function_ref` lẫn `preview_create_event_trigger`.
- `cargo check -p db-pro-core` → Finished OK.

## Note về literal acceptance trong issue
Issue mô tả `EXECUTE FUNCTION on_ddl();` không quote; implementation quote
identifier (`"on_ddl"()`) qua `quote_ident` — đúng convention safety của repo
(never concat unquoted idents) và Postgres chấp nhận ngang nhau. Fix đã có sẵn
trong `55dc35a4`; phiên này chỉ thêm test + docs, không đổi source.

## Chưa làm
- Không chạy full `cargo test --workspace` (phạm vi thay đổi chỉ là 1 test file
  trong db-pro-core).
- Issue #499 để OPEN chờ owner quyết định.
