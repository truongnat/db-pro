---
schema_version: 1
id: "MEM-SRC-3a7089055f14aea9"
title: "Source mirror: docs/plans/active/issue-490-audit-event-ids/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-490-audit-event-ids/CHECKLIST.md"
source_sha256: "3a7089055f14aea9c1fa21992bfd33c76ee34424d5d1cc360746e37b67b5ec4f"
mirror_body_sha256: "3a7089055f14aea9c1fa21992bfd33c76ee34424d5d1cc360746e37b67b5ec4f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #490 — Checklist

- [x] `parse_postgres_csv_line`: id = `csvlog:{hash(raw line)}`
- [x] `read_csvlog_page`: per-line file byte offset appended to id
- [x] Removed now-unused `pid`/`session_line` locals
- [x] Test: shared prefix / different tail → distinct ids
- [x] Test: identical raw line → stable id
- [x] `cargo check -p db-pro-core` — clean
- [x] `cargo test -p db-pro-core --lib domain::audit` — 8 passed, 0 failed
