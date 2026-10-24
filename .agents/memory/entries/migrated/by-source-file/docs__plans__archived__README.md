---
schema_version: 1
id: "MEM-SRC-e750ad87d54ab2ce"
title: "Source mirror: docs/plans/archived/README.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/README.md"
source_sha256: "e750ad87d54ab2ce80cd19482c6f81a1cf91e1c2f4391e494bf584ec1a6f90f8"
mirror_body_sha256: "e750ad87d54ab2ce80cd19482c6f81a1cf91e1c2f4391e494bf584ec1a6f90f8"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Archived Plans

This directory contains historical feature plans, audit records, and QA remediation plans that targeted the **React 19 / TypeScript / Vite / Tauri-webview frontend** (`_archive/frontend/`), which was archived on 2026-09-11 in favor of the native Rust `eframe`/`egui` desktop IDE (`crates/ui` + `crates/native-app`).

## Disposition Policy

- **State:** `OBSOLETE (frontend-era / archived React)`
- These plans are preserved as historical reference only.
- They are not built, linted, or tested in CI.
- Features needing equivalent behavior in native egui are implemented under dedicated native plans in `docs/plans/active/` (e.g. `native-core-architecture`, `issue-300-ui14`, `native-visual-redesign`).
