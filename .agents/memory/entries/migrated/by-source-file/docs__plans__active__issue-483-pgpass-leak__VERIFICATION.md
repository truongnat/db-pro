---
schema_version: 1
id: "MEM-SRC-1a190573014aa0a0"
title: "Source mirror: docs/plans/active/issue-483-pgpass-leak/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-483-pgpass-leak/VERIFICATION.md"
source_sha256: "1a190573014aa0a0af522a34a96d6857047c3324144d73c700108ac87d2e491b"
mirror_body_sha256: "1a190573014aa0a0af522a34a96d6857047c3324144d73c700108ac87d2e491b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #483 verification

- `failed_pgpass_write_removes_the_partial_file` — injected `Write` impl that always
  errors on `write()`; asserts `DbError::Internal` is returned and the pre-created
  target file no longer exists.
- `successful_pgpass_write_leaves_file_for_guard` — real file round-trip through
  `write_pgpass`; contents verified, fixture removed.
- Existing `pgpass_file_is_owner_only_and_removed_on_drop` still covers the guard's
  drop-removal path.
- `cargo check -p db-pro-infrastructure` — clean.
- `cargo test -p db-pro-infrastructure --lib` — 167 passed / 0 failed, including both new
  tests above and the pre-existing pgpass drop test.
