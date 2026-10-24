---
schema_version: 1
id: "MEM-SRC-eaa8143d87d72b7b"
title: "Source mirror: docs/plans/active/issue-365-auto-increment-type/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-365-auto-increment-type/VERIFICATION.md"
source_sha256: "eaa8143d87d72b7b1bb707feaf2d926ace6594b55332a57e411a4b407c18a8bd"
mirror_body_sha256: "eaa8143d87d72b7b1bb707feaf2d926ace6594b55332a57e411a4b407c18a8bd"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — #365

`cargo check -p db-pro-app` — clean. `cargo test -p db-pro-app` — 593 passed, 0 failed (includes schema-workbench/event regressions).

