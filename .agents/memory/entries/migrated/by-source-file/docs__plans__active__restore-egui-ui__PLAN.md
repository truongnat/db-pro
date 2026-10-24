---
schema_version: 1
id: "MEM-SRC-833dc91abed5df08"
title: "Source mirror: docs/plans/active/restore-egui-ui/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/restore-egui-ui/PLAN.md"
source_sha256: "833dc91abed5df08f31b63826a4050c0f81198048ad2b0d1d1d25945959d0c71"
mirror_body_sha256: "833dc91abed5df08f31b63826a4050c0f81198048ad2b0d1d1d25945959d0c71"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Restore egui UI

State: RUNTIME_VERIFY
Branch: main (owner override)

Owner request (2026-10-02): temporarily remove all rs-ui integration and use egui consistently with the Query editor. Restore the established egui source/manifests from `c0c1f5525b20a810913d1eee13c7ee2dd15b6664`; preserve domain, query, database, persistence and editor implementations. Remove the rs-ui adapters, Explorer texture renderer, experimental binary and dependencies. Keep the independent rs-ui repository intact.

Acceptance: no rs-ui references in executable Rust/manifests/lockfile; sidebar, tabs, resizing and grid return to egui; workspace gates and release build pass; capture normal/loading/error/empty at 1280×800, 1440×900 and 1920×1080.
