---
schema_version: 1
id: "MEM-SRC-44bd938d2a77d3f0"
title: "Source mirror: docs/plans/active/issue-480-select-stale-index/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-480-select-stale-index/FINDINGS.md"
source_sha256: "44bd938d2a77d3f03a6b07c01e0d1cf0804493439a536ba6ed27e03704178f5e"
mirror_body_sha256: "44bd938d2a77d3f03a6b07c01e0d1cf0804493439a536ba6ed27e03704178f5e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #480 Findings

## Evidence
- `components/select/handler.rs:112-125`: `navigate_selection` applied
  `selected.saturating_add(1) < option_count` to an already-stale index —
  with `selected=5, option_count=2` both guards failed and 5 was returned
  forever ("ArrowDown stuck at 5").
- `ui/select.rs:332-339` writes the return value back to `*self.selected`
  each frame the menu is open → self-healing once clamped here.
- Popup render is safe: options iterated by `enumerate`, icons/disabled via
  `.get(idx)` (`ui/select.rs:365-377`); trigger label via
  `selected_label` → `options.get` (`handler.rs:59-64`).

## Severity
P2 — stuck highlight, wrong "current" option perception; no data corruption.
