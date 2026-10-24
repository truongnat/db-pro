---
schema_version: 1
id: "MEM-SRC-40e41b73d43ab886"
title: "Source mirror: docs/plans/active/settings-shell-visual-polish/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/settings-shell-visual-polish/CHECKLIST.md"
source_sha256: "40e41b73d43ab886117f0bf6d933222115cb2c7be9724f0efcbcabf5242607bb"
mirror_body_sha256: "40e41b73d43ab886117f0bf6d933222115cb2c7be9724f0efcbcabf5242607bb"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

## Planning

- [x] Inspect the supplied references as visual input only.
- [x] Read the feature lifecycle and native UI design guidance.
- [x] Confirm the existing Settings state/action pipeline is reusable.
- [x] Record the direct user workflow override: work on local `main`, no worktrees.

## Implementation

- [x] Render Settings as a dedicated central surface.
- [x] Hide normal topbar, tabs, sidebar, output dock, and statusbar in Settings mode.
- [x] Add grouped Settings navigation with active and hover states.
- [x] Keep existing section content, commands, and persistence behavior intact.
- [x] Add a light-theme capture switch for deterministic visual evidence.

## Verification

- [x] Capture dark Settings surface at 1280×800.
- [x] Capture light Settings surface at 1280×800.
- [x] Capture 1440×900 and 1920×1080 width checks; record the host height cap.
- [x] Run the full workspace quality gates and locked release build.
- [ ] Independent review remains pending.
