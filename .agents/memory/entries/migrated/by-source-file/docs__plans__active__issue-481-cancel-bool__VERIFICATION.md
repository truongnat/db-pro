---
schema_version: 1
id: "MEM-SRC-440d1f0957c223a8"
title: "Source mirror: docs/plans/active/issue-481-cancel-bool/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-481-cancel-bool/VERIFICATION.md"
source_sha256: "440d1f0957c223a87b510fbd11e10fd0f7f9a3640f6ff02d20d3ecd821144721"
mirror_body_sha256: "440d1f0957c223a87b510fbd11e10fd0f7f9a3640f6ff02d20d3ecd821144721"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #481 verification

- `cancel_reports_backends_pg_cancel_backend_refused` — `cancel_failures(vec![123,456])`
  errors and the message names both pids and "cancel".
- `cancel_with_no_refused_backends_succeeds` — empty refused list → `Ok(())`.
- `owned_cancel_reads_pg_cancel_backend_result` — shipped `cancel` body contains
  `query_scalar("SELECT pg_cancel_backend($1)")` → `fetch_one(&pool)` →
  `cancel_failures(refused)`, and no `.execute(&pool)`.
- `cargo check -p db-pro-infrastructure` — clean.
- `cargo test -p db-pro-infrastructure --lib` — 167 passed / 0 failed, including the three
  tests above.
