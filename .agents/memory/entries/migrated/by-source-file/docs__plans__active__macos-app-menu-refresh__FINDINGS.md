---
schema_version: 1
id: "MEM-SRC-120e9ec5b77ed84e"
title: "Source mirror: docs/plans/active/macos-app-menu-refresh/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/macos-app-menu-refresh/FINDINGS.md"
source_sha256: "120e9ec5b77ed84ebf555db8c61b496bcf3c2b310242472c173c296627246ddc"
mirror_body_sha256: "120e9ec5b77ed84ebf555db8c61b496bcf3c2b310242472c173c296627246ddc"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings

Baseline inspected at `main@3dd988e2ff06ac71942c7ec331acf66389d3c5cb`.

- **P2 — Native app menu was absent.** Fixed in the working tree by installing the native app/File/Edit/View/Window/Help menus and dispatching their actions to the UI. Live menu behavior remains pending.
- **P2 — Explicit cache refresh was not exposed.** Fixed in the working tree by adding a typed cache-invalidation flag and invalidating before forced introspection. Automated UI dispatch tests pass; SQLite/PostgreSQL runtime behavior remains pending.

No P0/P1 finding is in scope. These are source findings, not live UI observations.
