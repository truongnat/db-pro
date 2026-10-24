---
schema_version: 1
id: "MEM-SRC-4007b21ac0b6fe7d"
title: "Source mirror: docs/plans/active/issue-453-shortcut-conflict-canonical/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-453-shortcut-conflict-canonical/CHECKLIST.md"
source_sha256: "4007b21ac0b6fe7d65c44983d11c18511dfd0baccd6c4587182e9fa024918af2"
mirror_body_sha256: "4007b21ac0b6fe7d65c44983d11c18511dfd0baccd6c4587182e9fa024918af2"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #453 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/453
- Fix: KeybindingSettings::conflicts parses shortcut parts and groups by canonical (Key, want_mod, want_shift, want_alt) tuple so alias variations (ctrl vs mod vs cmd) detect conflicts properly.
