---
schema_version: 1
id: "MEM-SRC-0ed720033ab51ced"
title: "Source mirror: docs/plans/active/issue-499-event-trigger-function-ref/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-499-event-trigger-function-ref/CHECKLIST.md"
source_sha256: "0ed720033ab51ced5d246527c212142c93547012ea670b55905139b7f8508ff6"
mirror_body_sha256: "0ed720033ab51ced5d246527c212142c93547012ea670b55905139b7f8508ff6"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #499 Checklist

- [x] Verify fix đã land (`55dc35a4`) vs acceptance criteria
- [x] Test preview-normalization mới (emit `EXECUTE FUNCTION <fn>();`)
- [x] `cargo test -p db-pro-core event_trigger` PASS
- [x] `cargo check -p db-pro-core` PASS
- [ ] Comment/close issue (chờ owner quyết định)
