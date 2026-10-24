---
schema_version: 1
id: "MEM-SRC-1bacb741ae709929"
title: "Source mirror: docs/plans/active/backend-four-issue-wave/tasks.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/backend-four-issue-wave/tasks.md"
source_sha256: "1bacb741ae7099292fc0b99a932f69c16f3bce23b6f77355ad33af67492abc03"
mirror_body_sha256: "1bacb741ae7099292fc0b99a932f69c16f3bce23b6f77355ad33af67492abc03"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Tasks / dependencies

- [ ] T1 R1/R2 D1/D2: PG builder/dependency fixes plus isolated provider cases (independent of T2/T3).
- [ ] T2 R3 D3: typed MySQL options, preserve TLS settings, real credential smoke (independent).
- [ ] T3 R4 D4: baseline BOM behavior, source→target import smoke and necessary regression; avoid redundant production change.
- [ ] T4 all/D5: parent-owned gates, review, evidence and published issue closures after T1–T3 stable.
