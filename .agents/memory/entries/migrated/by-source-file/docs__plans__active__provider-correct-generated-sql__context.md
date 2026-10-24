---
schema_version: 1
id: "MEM-SRC-f72972acd1a5c17b"
title: "Source mirror: docs/plans/active/provider-correct-generated-sql/context.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/provider-correct-generated-sql/context.md"
source_sha256: "f72972acd1a5c17b74ceebf170be184f3a151dd57c8e071c609371c91ae8f514"
mirror_body_sha256: "f72972acd1a5c17b74ceebf170be184f3a151dd57c8e071c609371c91ae8f514"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Context — #441/#442

Source baseline: `76b120724a7542de8b587a002811ddacad28156a`.

Reuse existing dialect, no ports change. Repair newly consumed MySQL referenced-schema joins and SQLServer action spellings; declare MySQL supporting indexes before FKs. PostgreSQL typed binding prerequisite tracked separately as #533.

Detailed source pointers/alternatives and inherited limits: PLAN.md, FINDINGS.md.
