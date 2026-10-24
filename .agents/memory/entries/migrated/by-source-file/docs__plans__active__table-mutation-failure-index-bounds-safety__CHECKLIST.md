---
schema_version: 1
id: "MEM-SRC-4d685fa7deda8d2f"
title: "Source mirror: docs/plans/active/table-mutation-failure-index-bounds-safety/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/table-mutation-failure-index-bounds-safety/CHECKLIST.md"
source_sha256: "4d685fa7deda8d2f6f8d6dd3da25e24b3279c15eba35d982520a761528e74e76"
mirror_body_sha256: "4d685fa7deda8d2f6f8d6dd3da25e24b3279c15eba35d982520a761528e74e76"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist: Table Mutation Transaction Failure Index Bounds Safety

- [x] Check bounds and failure phase in `TableMutationExecution::execute`
- [x] Add regression test in `table_data_service_tests.rs` for `Commit` phase failure
- [x] Pass `cargo check -p db-pro-core`
- [x] Pass `cargo test -p db-pro-core`
