# #490 — Audit event ids collide on shared timestamp/pid/session/message prefix

## Goal
Every `AuditEvent` parsed from a PostgreSQL csvlog line must have a unique id:
selection, bookmark and export key on `event.id`
(`audit_state.rs`, `audit_surface_view.rs`).

## Root cause
`parse_postgres_csv_line` built `id = timestamp|pid|session_line|message[..48]`.
Two log rows sharing those fields (common for the same session retrying a
statement) got identical ids → UI selection/export hit the wrong record.

## Fix
- `id = "csvlog:" + hex(DefaultHasher(raw line))` in
  `parse_postgres_csv_line`: content hash of the entire raw line, so differing
  tails/quoting produce distinct ids and identical content stays stable.
- `read_csvlog_page` tracks each line's file-absolute byte offset
  (`split_inclusive('\n')` + cumulative cursor) and rewrites
  `id = "{hash}@{offset}"`, so even byte-identical lines in one file are unique.
  Position info was available only in this function — preferred over a
  collision-sequence counter, which would be page-order dependent.
- Dead `pid`/`session_line` bindings removed (nothing else used them).

## Scope
- `crates/core/src/domain/audit.rs` only.
- Tests appended in `crates/core/src/domain/tests/audit_tests.rs`.

## Acceptance
- Lines differing only in message tail → distinct ids.
- Byte-identical repeated lines in one file → distinct ids (offset suffix).
- Same raw line parsed twice → same id (stable local identity).
