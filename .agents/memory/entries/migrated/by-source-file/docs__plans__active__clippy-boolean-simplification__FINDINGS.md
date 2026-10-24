---
schema_version: 1
id: "MEM-SRC-08ba526534a00877"
title: "Source mirror: docs/plans/active/clippy-boolean-simplification/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/clippy-boolean-simplification/FINDINGS.md"
source_sha256: "08ba526534a00877d0de47653853f5a5d938b111324c44b3fcbc4b440cc03096"
mirror_body_sha256: "08ba526534a00877d0de47653853f5a5d938b111324c44b3fcbc4b440cc03096"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
## Analysis
The nonminimal boolean `!pending.as_ref().is_some_and(...)` was introduced in a recent commit. Replacing it with `pending.as_ref().is_none_or(...)` satisfies Clippy while preserving identical logic semantics.
