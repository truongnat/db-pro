---
schema_version: 1
id: "MEM-SRC-7b2ff59f632a330f"
title: "Source mirror: docs/plans/active/backend-four-issue-wave/requirements.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/backend-four-issue-wave/requirements.md"
source_sha256: "7b2ff59f632a330f3eae5259ea9d4629c4f89b430debcdb627e1baad222415eb"
mirror_body_sha256: "7b2ff59f632a330f3eae5259ea9d4629c4f89b430debcdb627e1baad222415eb"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Requirements

- [ ] R1 #499: valid bare/qualified zero-argument function references create a working PG event trigger; reject malformed references.
- [ ] R2 #508: exact requested PG relation dependency identity, unaffected by search_path/wildcards/quotes/decoys.
- [ ] R3 #452: MySQL credentials and database names containing URL-reserved characters connect/test correctly without changing TLS policy.
- [ ] R4 #486: verify BOM/non-BOM delimited import header/data semantics; repair only a reproduced failure.

Owner approved bounded four-ticket continuation. Out of scope: other PR ownership, UI redesign, unrelated global lint debt, SQLServer runtime.
