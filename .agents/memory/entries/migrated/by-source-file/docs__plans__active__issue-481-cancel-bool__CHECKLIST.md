---
schema_version: 1
id: "MEM-SRC-eba4efbb7b5ebb29"
title: "Source mirror: docs/plans/active/issue-481-cancel-bool/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-481-cancel-bool/CHECKLIST.md"
source_sha256: "eba4efbb7b5ebb2918f7e0c199317f35850ada268a22c7f2d3641ab2d9b812cc"
mirror_body_sha256: "eba4efbb7b5ebb2918f7e0c199317f35850ada268a22c7f2d3641ab2d9b812cc"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #481 checklist

- [x] `cancel` uses `query_scalar` + `fetch_one` and checks the returned bool
- [x] refused pids aggregated into `cancel_failures` → `DbError` naming pids
- [x] `Ok(())` preserved for empty pid list (UI cancel button)
- [x] tests: pure-fn aggregation test + source-shape test in `tests/connector_tests.rs`
- [x] `cargo test -p db-pro-infrastructure --lib` green: 167 passed / 0 failed
- [x] `cargo check -p db-pro-infrastructure` clean
