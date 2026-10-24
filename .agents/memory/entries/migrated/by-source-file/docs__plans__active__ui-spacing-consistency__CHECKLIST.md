---
schema_version: 1
id: "MEM-SRC-0aafd887a85e8e81"
title: "Source mirror: docs/plans/active/ui-spacing-consistency/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-spacing-consistency/CHECKLIST.md"
source_sha256: "0aafd887a85e8e817a6e13be57de32186f4d048a2462cfe1efece01bf4494a89"
mirror_body_sha256: "0aafd887a85e8e817a6e13be57de32186f4d048a2462cfe1efece01bf4494a89"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

- [x] Trace audit findings and tab geometry
- [x] Fix shared spacing ownership and width reservation
- [x] Apply the existing 38pt form-control height and semantic label/helper gaps
- [x] Make Gallery feedback, shortcut, and empty-state groups fit their local width
- [x] Pad workspace tabs and separate shared segmented-tab targets
- [x] Add regression tests and collect the geometry probe
- [x] Run formatting, workspace check, clippy, workspace tests, clean-code scan, and capture build
- [x] Capture affected native surfaces at requested widths; record available-height limit
- [x] Review captures and record known limitations in VERIFICATION.md
- [ ] Verify the native UI at the full requested logical heights on a display that supports them
