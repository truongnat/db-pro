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
