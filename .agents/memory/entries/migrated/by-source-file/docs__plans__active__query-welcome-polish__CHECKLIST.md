---
schema_version: 1
id: "MEM-SRC-5dd1c56d0692edf2"
title: "Source mirror: docs/plans/active/query-welcome-polish/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/query-welcome-polish/CHECKLIST.md"
source_sha256: "5dd1c56d0692edf2f53241fcbddc7d7ff2f171d4ab17bab3474476f37d71c9ef"
mirror_body_sha256: "5dd1c56d0692edf2f53241fcbddc7d7ff2f171d4ab17bab3474476f37d71c9ef"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

- [x] Inspect exact Query and Welcome source paths and shared Select behavior.
- [x] Replace legacy row-cap ComboBox with shared Select; add compact size/Ghost variant and geometry regressions.
- [x] Simplify Welcome; remove only private obsolete handlers/state.
- [x] Review diff, fmt/check/clippy and existing Select tests.
- [x] Native release build and light/dark native captures.
- [x] Full release workspace suite executed; two inherited failures reported.
- [ ] Interactive selection/keyboard verification and exact taller viewport heights.
- [ ] Independent review before release; no PR requested.
