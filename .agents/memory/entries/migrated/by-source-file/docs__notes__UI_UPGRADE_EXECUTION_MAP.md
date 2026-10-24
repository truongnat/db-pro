---
schema_version: 1
id: "MEM-SRC-db0d937b92b4782e"
title: "Source mirror: docs/notes/UI_UPGRADE_EXECUTION_MAP.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/notes/UI_UPGRADE_EXECUTION_MAP.md"
source_sha256: "db0d937b92b4782e1ceb76631da3459b6511031643752e4ee38fdfe6d466bc8d"
mirror_body_sha256: "db0d937b92b4782e1ceb76631da3459b6511031643752e4ee38fdfe6d466bc8d"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Native UI Upgrade Execution Map

Parent goal issue: to be created from this baseline.

Recommended sequence:

1. UI visual QA harness and screenshot evidence contract
2. Design-system consolidation / raw egui cleanup
3. Shell + Activity Bar + Sidebar hierarchy
4. Explorer tree + search + context menus
5. Query workspace/editor/output composition
6. Data/result grid + editing states
7. Table/object workbench + schema forms
8. Dialogs/forms/confirmations
9. Files/IDE workspace
10. Agent workspace
11. ER workspace
12. Settings + keyboard/accessibility
13. Empty/loading/error/status feedback
14. Responsive density + overflow hardening

The core rule is runtime-first: each ticket must be manually traversed in the native app and include visual evidence. No UI ticket closes on source inspection alone.
