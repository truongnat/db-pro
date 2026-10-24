---
schema_version: 1
id: "MEM-SRC-9ed7978ca93d15dc"
title: "Source mirror: docs/plans/active/issue-440-stale-schema-loaded/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-440-stale-schema-loaded/VERIFICATION.md"
source_sha256: "9ed7978ca93d15dcb3f58bec09c8b582007b2814140bccdf5f8a31e78f79c497"
mirror_body_sha256: "9ed7978ca93d15dcb3f58bec09c8b582007b2814140bccdf5f8a31e78f79c497"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — #440

`cargo check -p db-pro-app` — clean. `cargo test -p db-pro-app` — 593 passed, 0 failed (includes schema-workbench/event regressions).
New regression test `schema_loaded_after_disconnect_is_dropped` passes.

