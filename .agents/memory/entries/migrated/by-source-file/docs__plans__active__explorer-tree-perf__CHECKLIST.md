---
schema_version: 1
id: "MEM-SRC-fb8e626fc7f0151e"
title: "Source mirror: docs/plans/active/explorer-tree-perf/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/explorer-tree-perf/CHECKLIST.md"
source_sha256: "fb8e626fc7f0151e58bbe3c518c678ca99a891f1cc131ac8ad1b99d7d28a76de"
mirror_body_sha256: "fb8e626fc7f0151e58bbe3c518c678ca99a891f1cc131ac8ad1b99d7d28a76de"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Explorer Tree Perf — Checklist

- [x] Add `ExplorerNavCache` keyed by connection/schema/search
- [x] Clear cache on schema load and schema activation
- [x] Schema-scoped table count / matching count without name Vec alloc
- [x] Tables folder default closed; auto-open on search
- [x] Viewport cull + `EXPLORER_MAX_TABLES` truncate hint
- [x] Defer Views/Functions/Triggers filter until folder open
- [x] Schema-scoped folder persistence IDs
- [x] Unit test for matching count
- [ ] Runtime verify on large schema (PostgreSQL + SQLite)
- [x] Quality gates: check / clippy / targeted tests
