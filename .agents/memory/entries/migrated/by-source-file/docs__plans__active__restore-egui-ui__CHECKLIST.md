---
schema_version: 1
id: "MEM-SRC-dd64bd6c98fee742"
title: "Source mirror: docs/plans/active/restore-egui-ui/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/restore-egui-ui/CHECKLIST.md"
source_sha256: "dd64bd6c98fee7425ec786b92e5af0e72fb5e5ce60da8bf7b104edc9d778e194"
mirror_body_sha256: "dd64bd6c98fee7425ec786b92e5af0e72fb5e5ce60da8bf7b104edc9d778e194"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

- [x] Restore pre-integration egui implementations and dependencies.
- [x] Remove rs-ui code paths and experimental binary.
- [x] Preserve Query font/rendering and business logic.
- [x] Format, check, clippy and workspace tests.
- [x] Release build; collect 15 native framebuffer captures.
- [x] Review diff; record evidence; code commit.
- [ ] Capture full logical heights 900/1080 on a display supporting them (current host clamps to 838).
- [x] Commit code and evidence; push authorized by owner (remote result recorded in final handoff).
