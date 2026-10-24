---
schema_version: 1
id: "MEM-SRC-9b0011f5aba92e04"
title: "Source mirror: docs/plans/active/issue-479-snippet-secret-clear/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-479-snippet-secret-clear/VERIFICATION.md"
source_sha256: "9b0011f5aba92e0431436b357d2aa07127a7199d1b292794c9603acc80b612ee"
mirror_body_sha256: "9b0011f5aba92e0431436b357d2aa07127a7199d1b292794c9603acc80b612ee"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #479 Verification

Commands run:

- `cargo test -p db-pro-app --lib connection::` — 31 passed, 0 failed.
  New tests (`mapper_tests.rs`):
  - `apply_snippet_clears_cloud_snippet_after_success` — imports
    `postgresql://alice:s3cret@db.example.com:5432/appdb?sslmode=require`;
    asserts all fields populated (incl. `password == "s3cret"`,
    `ssl_mode == UiSslMode::Require`) and `cloud_snippet == ""`.
  - `apply_snippet_keeps_cloud_snippet_on_parse_error` — invalid URI keeps
    the original text so the user can correct it.
- `cargo test -p db-pro-app connection` — 68 passed, 0 failed.
- `cargo check -p db-pro-app` — clean.
