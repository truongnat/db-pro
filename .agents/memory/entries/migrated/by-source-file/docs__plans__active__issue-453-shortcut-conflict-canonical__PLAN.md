---
schema_version: 1
id: "MEM-SRC-87232272da17e850"
title: "Source mirror: docs/plans/active/issue-453-shortcut-conflict-canonical/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-453-shortcut-conflict-canonical/PLAN.md"
source_sha256: "87232272da17e850a3f9ed815fd2b059795811d858ff530ad8f01204a1e1f638"
mirror_body_sha256: "87232272da17e850a3f9ed815fd2b059795811d858ff530ad8f01204a1e1f638"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #453 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/453
- Fix: KeybindingSettings::conflicts parses shortcut parts and groups by canonical (Key, want_mod, want_shift, want_alt) tuple so alias variations (ctrl vs mod vs cmd) detect conflicts properly.
