---
schema_version: 1
id: "MEM-SRC-7fa40862f268afd6"
title: "Source mirror: docs/plans/active/issue-484-export-row-cap/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-484-export-row-cap/CHECKLIST.md"
source_sha256: "7fa40862f268afd62ee2739adcb1bf8d307c94d9c2b221abfe4c2c75ddb46a98"
mirror_body_sha256: "7fa40862f268afd62ee2739adcb1bf8d307c94d9c2b221abfe4c2c75ddb46a98"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #484 Checklist

- [x] PLAN/FINDINGS/CHECKLIST/VERIFICATION scaffold
- [x] `MAX_EXPORT_ROWS` + `ensure_within_export_cap` in export_formats.rs
- [x] Enforcement in `execute_for_export` (seam where row data enters export)
- [x] Defense-in-depth checks in render_csv / render_json / render_excel
- [x] Tests: boundary (== cap ok, cap+1 rejected), renderer-level, service-level
- [x] `cargo test -p db-pro-core` PASS (487) + `cargo check` PASS
- [ ] Comment/close issue (awaiting owner decision)
