---
schema_version: 1
id: "MEM-SRC-a013ead541dafebe"
title: "Source mirror: docs/plans/active/issue-440-stale-schema-loaded/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-440-stale-schema-loaded/PLAN.md"
source_sha256: "a013ead541dafebeedf8727709f73c7c9bd867083354e8c2147e64953079b2a5"
mirror_body_sha256: "a013ead541dafebeedf8727709f73c7c9bd867083354e8c2147e64953079b2a5"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #440: Stale SchemaLoaded response applies after disconnect

Status: FIXED in this change

Changed `on_schema_loaded` guard from `is_some_and(|e| e != request_id)` to `schema_request != Some(request_id)` — a response whose pending request was cleared by disconnect/reset is now dropped.
