---
schema_version: 1
id: "MEM-SRC-e888ebb3d344f51c"
title: "Source mirror: docs/plans/active/provider-correct-generated-sql/requirements.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/provider-correct-generated-sql/requirements.md"
source_sha256: "e888ebb3d344f51cbf11c76d3cc9e18d70a6808498175491696e9f5068ebd851"
mirror_body_sha256: "e888ebb3d344f51cbf11c76d3cc9e18d70a6808498175491696e9f5068ebd851"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Requirements — #441/#442

Owner delegated continuous grouped issue fixes on 2026-10-16; no repeated approval required for this bug scope.

PK reload uses provider pagination/order; reconstructed DDL quotes every nested identifier with DriverDialect and preserves FK targets/actions/indexes. Close only provider acceptance actually exercised.

Full acceptance/non-goals: PLAN.md. Per-issue closure still requires actual evidence.
