---
schema_version: 1
id: "MEM-SRC-c12cec299f246dd2"
title: "Source mirror: docs/plans/active/safe-sample-sqlite-seed/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/safe-sample-sqlite-seed/CHECKLIST.md"
source_sha256: "c12cec299f246dd23d0a17c0c02b8f9c4e8f797bc5cf7c23624c7dacd16bfd9e"
mirror_body_sha256: "c12cec299f246dd23d0a17c0c02b8f9c4e8f797bc5cf7c23624c7dacd16bfd9e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — Safe SQLite Sample Seeding

- [x] Remove the shared `/tmp` fixture path from new startup seeding.
- [x] No-overwrite create and validate existing fixtures by app id and marker.
- [x] Reject symlinks and preserve unrecognized existing files.
- [x] Seed the sample connection read-only and migrate the legacy sample entry.
- [x] Add tests for idempotence, arbitrary existing files, and symlink safety.
- [x] Native test suite, native package check, formatting, and diff validation pass.
- [x] Rebase, commit, push, and open draft PR #529 linked to #492.
