---
schema_version: 1
id: "MEM-SRC-42d27a1af79ae7b9"
title: "Source mirror: docs/plans/active/issue-446-save-query-version-check/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-446-save-query-version-check/PLAN.md"
source_sha256: "42d27a1af79ae7b9530a2c5abf56ad9c3d420834f9b7c30aa09512f859694ad2"
mirror_body_sha256: "42d27a1af79ae7b9530a2c5abf56ad9c3d420834f9b7c30aa09512f859694ad2"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #446 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/446
- Fix: commit_dispatched records the document's buffer version along with the request. on_query_saved only calls mark_saved if the buffer version still matches, preventing edits made while save was in flight from being marked as saved.
