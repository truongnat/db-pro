---
schema_version: 1
id: "MEM-SRC-11d73ed2d7dd6c97"
title: "Source mirror: docs/plans/archived/rc1-p2-grid-connection-correctness/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/rc1-p2-grid-connection-correctness/CHECKLIST.md"
source_sha256: "11d73ed2d7dd6c97b65e7f89a4b1c856720b79cae780e895b4cf2faa7bae1ede"
mirror_body_sha256: "11d73ed2d7dd6c97b65e7f89a4b1c856720b79cae780e895b4cf2faa7bae1ede"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# RC1 P2 Grid & Connection Correctness Checklist

- [ ] Data Grid Columns picker checkbox click calls `onToggleHiddenColumn` exactly once
- [ ] Editing host, port, database, username, password, or SSH fields in Connection Editor clears previous test status badge
- [ ] Connection test error displays specific backend error details (`userMessage`) when available
- [ ] Frontend typecheck passes (`npm run typecheck`)
- [ ] Frontend tests pass (`npm run test`)
- [ ] Rust tests pass (`cargo test -p db-pro-core -p db-pro-infrastructure`)
