---
schema_version: 1
id: "MEM-SRC-e3d5321c237efa35"
title: "Source mirror: docs/plans/active/issue-493-sqlite-busy-timeout/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-493-sqlite-busy-timeout/FINDINGS.md"
source_sha256: "e3d5321c237efa35d133316a718e63af3cdb732ee8b09ff87f741bec4a044bb5"
mirror_body_sha256: "e3d5321c237efa35d133316a718e63af3cdb732ee8b09ff87f741bec4a044bb5"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — issue #493

- `rusqlite::Connection::open` defaults to `busy_timeout = 0`: `sqlite3_step`
  returns `SQLITE_BUSY` on the first lock collision.
- `actor.rs` had exactly one `Connection::open` site (`spawn`, old line 425);
  no other open site needed the fix.
- Precedent: `backup/sqlite_backup.rs` propagates `busy_timeout` errors via
  `map_err(crate::error::from_rusqlite)` — same pattern used here (stricter than
  the issue's `.ok()` suggestion, consistent with the codebase).
