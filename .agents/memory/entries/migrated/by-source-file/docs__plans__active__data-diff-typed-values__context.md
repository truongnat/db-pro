---
schema_version: 1
id: "MEM-SRC-bd443d13d0ac5a5c"
title: "Source mirror: docs/plans/active/data-diff-typed-values/context.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/data-diff-typed-values/context.md"
source_sha256: "bd443d13d0ac5a5c8b5a01469495f68c27935b18879f0dd58313a896c18f48c7"
mirror_body_sha256: "bd443d13d0ac5a5c8b5a01469495f68c27935b18879f0dd58313a896c18f48c7"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Context — #457/#485

Source baseline: `76b120724a7542de8b587a002811ddacad28156a`.

Borrow typed comparison values/keys. INSERT/predicates use typed cells; UPDATE consumes exact nullable scalar change values (None vs Some), never sentinel decoding. Preserve DTO types and existing binary/MySQL limits.

Detailed source pointers/alternatives and inherited limits: PLAN.md, FINDINGS.md.
