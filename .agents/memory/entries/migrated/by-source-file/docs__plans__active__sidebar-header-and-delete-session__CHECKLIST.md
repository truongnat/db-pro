---
schema_version: 1
id: "MEM-SRC-96ea6b02c0986c3e"
title: "Source mirror: docs/plans/active/sidebar-header-and-delete-session/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sidebar-header-and-delete-session/CHECKLIST.md"
source_sha256: "96ea6b02c0986c3e7dd53ef3112dd24a8f7dad0c5ba9ac4ef6ee8df3c24df3db"
mirror_body_sha256: "96ea6b02c0986c3e7dd53ef3112dd24a8f7dad0c5ba9ac4ef6ee8df3c24df3db"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

- [x] Header name + search merged into one launcher pill
- [x] Plus stays separate (new connection)
- [x] Sidebar content hosted in `egui::Area` for wheel hit-testing
- [x] Explorer + other-activity ScrollAreas bound with `max_height`
- [x] `connection.deleted` only resets session when deleted id was active
- [x] Unit tests for delete + header palette (4/4)
