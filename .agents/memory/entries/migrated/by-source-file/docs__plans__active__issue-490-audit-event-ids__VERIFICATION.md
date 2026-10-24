---
schema_version: 1
id: "MEM-SRC-21781f19fadc4af2"
title: "Source mirror: docs/plans/active/issue-490-audit-event-ids/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-490-audit-event-ids/VERIFICATION.md"
source_sha256: "21781f19fadc4af244edc7e6b92229e5ba749d9941abb669d85db33866ebd898"
mirror_body_sha256: "21781f19fadc4af244edc7e6b92229e5ba749d9941abb669d85db33866ebd898"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
