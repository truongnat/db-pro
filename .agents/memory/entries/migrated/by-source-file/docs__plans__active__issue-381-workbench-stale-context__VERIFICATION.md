---
schema_version: 1
id: "MEM-SRC-20e7b912c113e836"
title: "Source mirror: docs/plans/active/issue-381-workbench-stale-context/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-381-workbench-stale-context/VERIFICATION.md"
source_sha256: "20e7b912c113e836686486444894a73e65eb46ddddb881ca76b33968251da493"
mirror_body_sha256: "20e7b912c113e836686486444894a73e65eb46ddddb881ca76b33968251da493"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — #381

`cargo check -p db-pro-app` — clean. `cargo test -p db-pro-app` — 593 passed, 0 failed (includes schema-workbench/event regressions).

