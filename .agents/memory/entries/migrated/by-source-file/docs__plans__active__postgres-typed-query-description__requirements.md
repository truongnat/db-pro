---
schema_version: 1
id: "MEM-SRC-e44a69b0ee420ea4"
title: "Source mirror: docs/plans/active/postgres-typed-query-description/requirements.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/postgres-typed-query-description/requirements.md"
source_sha256: "e44a69b0ee420ea437dde980df45194e2f63620b1ae3268c69768e7260d0af48"
mirror_body_sha256: "e44a69b0ee420ea437dde980df45194e2f63620b1ae3268c69768e7260d0af48"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Requirements — #533

Owner-authorized continuous issue work includes this reproduced prerequisite. INTEGER PK lookup with Int64 inputs must succeed; retain BIGINT precision, empty/nullable metadata, contextual NULL inference and bounded-stream completion. No SQL cache suppression, type-special-case query rewrite or new dependency. Full acceptance: PLAN.md.
