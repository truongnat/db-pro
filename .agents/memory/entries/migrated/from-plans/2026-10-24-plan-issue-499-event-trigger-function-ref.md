---
schema_version: 1
id: "MEM-PLAN-issue-499-event-trigger-function-a6b648c75378"
title: "Plan archive: Issue 499 Event Trigger Function Ref"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-499-event-trigger-function-ref]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "a6b648c75378ef71908f1da69dc949756c28cb23a2c943461151ccbdcd91d6f2"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-499-event-trigger-function-ref/PLAN.md"
  - "docs/plans/active/issue-499-event-trigger-function-ref/FINDINGS.md"
  - "docs/plans/active/issue-499-event-trigger-function-ref/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-499-event-trigger-function-ref/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# #499 Findings

## Root cause (tại thời điểm issue được viết)
`validate_function_ref` chỉ whitelist ký tự `[A-Za-z0-9_().,]` nên `func(,)`,
`func()()`, `func(a)` đều pass; `preview_create_event_trigger` emit
`EXECUTE FUNCTION {function_ref};` verbatim → `public.on_ddl` render thành
`EXECUTE FUNCTION public.on_ddl;` (thiếu `()` → syntax error trên Postgres).

## Trạng thái hiện tại
Commit `55dc35a4` (wave `backend-four-issue-wave`) đã fix:
- `validate_function_ref` (event_trigger.rs:59-68): strip suffix `()` một lần,
  split `schema.func`, `validate_ident` từng phần → mọi input có `(`, `)`, `,`,
  space, nhiều `.`, ident rỗng/số đầu đều reject. `,` đã ra khỏi whitelist.
- `preview_create_event_trigger` (:94-113): normalize — strip `()` nếu có,
  `to_ascii_lowercase` (giữ name resolution của Postgres cho unquoted input),
  `quote_ident` từng component, emit `EXECUTE FUNCTION {function_name}();`.

## Gap còn lại
Test coverage cho preview path — acceptance issue yêu cầu assert output
`EXECUTE FUNCTION on_ddl();`; code thực tế emit identifier đã quote
(`EXECUTE FUNCTION "on_ddl"();`) — hợp lệ SQL và consistent với `quote_ident`
convention của file. Đã bổ sung test assert exact SQL.

## Severity
P2 — trước fix: SQL preview thiếu `()` (syntax error khi apply) và input
malformed được chấp nhận.


## Source: `VERIFICATION.md`

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

