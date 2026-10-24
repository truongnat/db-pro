---
schema_version: 1
id: "MEM-SRC-1a03adbc0e64c2cc"
title: "Source mirror: docs/plans/active/issue-453-shortcut-conflict-canonical/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-453-shortcut-conflict-canonical/FINDINGS.md"
source_sha256: "1a03adbc0e64c2cc9d752f11ee026468df1718c56109b5517096d0cec8e6ba79"
mirror_body_sha256: "1a03adbc0e64c2cc9d752f11ee026468df1718c56109b5517096d0cec8e6ba79"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #453 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/453
- Fix: KeybindingSettings::conflicts parses shortcut parts and groups by canonical (Key, want_mod, want_shift, want_alt) tuple so alias variations (ctrl vs mod vs cmd) detect conflicts properly.
