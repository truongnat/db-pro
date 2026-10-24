---
schema_version: 1
id: "MEM-SRC-214af71d87fb47a0"
title: "Source mirror: docs/plans/active/ssh-transport-lifecycle/dod.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ssh-transport-lifecycle/dod.md"
source_sha256: "214af71d87fb47a0b38f886ee47a1db807928546c0de8492afa5fec945a318d7"
mirror_body_sha256: "214af71d87fb47a0b38f886ee47a1db807928546c0de8492afa5fec945a318d7"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Definition of Done — #435/#451/#475; #400 partial

Process environment and real socket regressions; independent public tunnel smoke; real PostgreSQL in-flight query drains over local fixture transport. Native intent/domain validation tests required. This does not prove real SSH authentication.

CHECKLIST.md tracks actual states; VERIFICATION.md is authoritative for executed commands, failures/skips and runtime limits. A historical worker NOT RUN statement is not a parent PASS.
