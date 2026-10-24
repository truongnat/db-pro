---
schema_version: 1
id: "MEM-SRC-acb5a98a553b94d9"
title: "Source mirror: docs/plans/active/component-interaction-polish/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/component-interaction-polish/CHECKLIST.md"
source_sha256: "acb5a98a553b94d9c9e0bb1a0a30b4b6be660dbccf45ddd7fd939efdaea8df21"
mirror_body_sha256: "acb5a98a553b94d9c9e0bb1a0a30b4b6be660dbccf45ddd7fd939efdaea8df21"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

- [x] Shared Button whole-shape transform; geometry regression test covers centered/left alignment, icon, underline and untouched source galley.
- [x] Alert reserves full copy-column width; regression test covers short/wrapped copy at 280 and 800 points.
- [x] Theme owns blue/white pair; egui adapter recolors only selected glyphs after native selection finalizes.
- [x] Native Label drag and TextEdit partial-selection tests; both themes for TextEdit.
- [x] fmt, check, clippy, workspace tests, release build.
- [x] Native form-error screenshots at three requested widths; Alert dismiss clicked successfully.
- [x] Native selected Alert description and selected Database input screenshots.
- [ ] Full requested 900/1080 heights (host display clamps capture height).
- [ ] Native held-button animation recording; current coverage is geometry tests.
- [ ] Independent review (self-review is recorded separately).
