---
schema_version: 1
id: "MEM-PLAN-issue-490-audit-event-ids-f868f1977504"
title: "Plan archive: Issue 490 Audit Event Ids"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-490-audit-event-ids]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "f868f19775040bcd4a173e1316b3c879a0a0df1d6998c60b8c3f14e5f3f9682f"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-490-audit-event-ids/PLAN.md"
  - "docs/plans/active/issue-490-audit-event-ids/FINDINGS.md"
  - "docs/plans/active/issue-490-audit-event-ids/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-490-audit-event-ids/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# #490 — Findings

- `AuditEvent.id` consumers: `audit_state.rs` HashSets (bookmarks/selected),
  `audit_surface_view.rs` actions (`SetSelected`, `ToggleBookmark`), export
  preview filtering — all in-session, no cross-restart persistence requirement.
- `parse_postgres_csv_line` is `pub` but has exactly one production caller:
  `read_csvlog_page` (via `AuditService::load_events`). Keeping its signature
  unchanged and letting the page reader enrich the id avoids an API change.
- `DefaultHasher` is not specified cross-build, but the type is `std`'s stable
  constructor and ids are local/per-load; good enough per "Stable local
  identity (not a server id)" doc on the field.
- Byte offsets are file-absolute (`start + body_offset + line_in_body`), so a
  larger `max_scan_bytes` page keeps identical ids for the same lines — better
  than body-relative offsets.
- `String::from_utf8_lossy` may replace bytes, shifting nothing: offsets are
  computed over the *lossy text* which is what is parsed; consistent within a
  page since cursor walks the same `body` string.


## Source: `VERIFICATION.md`

# #490 — Verification

Ran 2026-10-17 on branch `main` (uncommitted worktree):

- `cargo check -p db-pro-core` — clean (after sibling WIP settled).
- `cargo test -p db-pro-core --lib domain::audit` — 8 passed, 0 failed:
  - `event_ids_differ_when_only_the_message_tail_differs` ✔
  - `event_id_is_stable_for_the_same_line` ✔
  - `identical_log_lines_get_distinct_ids_within_a_file` ✔
  - existing `page_reads_newest_without_full_materialization`,
    `parse_csv_quoted_commas`, `redact_password_and_uri`,
    `filter_and_tag_parse`, `export_csv_includes_header` — all still pass.

Acceptance mapping:
- Same ts/pid/session/prefix, different tails → distinct ids (content hash). ✔
- Byte-identical lines in one file → distinct ids (`@byte_offset` suffix). ✔

