---
schema_version: 1
id: "MEM-SRC-4f27c2be1975e3a2"
title: "Source mirror: docs/plans/active/issue-381-workbench-stale-context/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-381-workbench-stale-context/FINDINGS.md"
source_sha256: "4f27c2be1975e3a26c11ff72b612236427a69c2268ae88619a82ce5a894bbcde"
mirror_body_sha256: "4f27c2be1975e3a26c11ff72b612236427a69c2268ae88619a82ce5a894bbcde"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — #381

schema_workbench.rs:407-430 + `clear_plan_state` :313-321 (clears preview_sql/safety/fingerprint/error/connection/pending_plan/apply_confirmation).
