---
schema_version: 1
id: "MEM-SRC-00b030c41d990ad8"
title: "Source mirror: docs/plans/archived/connection-list-reconnect-sideeffect/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/connection-list-reconnect-sideeffect/CHECKLIST.md"
source_sha256: "00b030c41d990ad893cabe79c503a11c472addb9451b5f4b6391c9d8ec96214f"
mirror_body_sha256: "00b030c41d990ad893cabe79c503a11c472addb9451b5f4b6391c9d8ec96214f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — QA-P1-09 Connection-list query reconnect side-effect

- [ ] PLAN.md, CHECKLIST.md, FINDINGS.md, VERIFICATION.md created
- [ ] `useConnectionList` queryFn does not trigger `restoreSession` on generic refetches
- [ ] One-shot session restoration logic is cleanly separated
- [ ] Frontend unit tests verify refetch safety
- [ ] Quality gates (FE + Rust) executed and passing
- [ ] PR published against main
