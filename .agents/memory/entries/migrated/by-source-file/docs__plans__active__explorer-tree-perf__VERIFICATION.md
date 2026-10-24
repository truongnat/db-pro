---
schema_version: 1
id: "MEM-SRC-7cd23d9e2f6f17c8"
title: "Source mirror: docs/plans/active/explorer-tree-perf/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/explorer-tree-perf/VERIFICATION.md"
source_sha256: "7cd23d9e2f6f17c8d8e83ac31996ee89571196a59fa7d2f7f40a476f1fe61873"
mirror_body_sha256: "7cd23d9e2f6f17c8d8e83ac31996ee89571196a59fa7d2f7f40a476f1fe61873"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Explorer Tree Perf — Verification

## Commands

```bash
cargo check -p db-pro-ui --all-targets
cargo test -p db-pro-ui schema_matching_table_count -- --nocapture
cargo test -p db-pro-ui explorer_search_matches_table_names -- --nocapture
cargo clippy -p db-pro-ui --all-targets -- -D warnings
```

## Status

| Gate | Result |
|------|--------|
| `cargo check -p db-pro-ui --all-targets` | passed |
| `schema_matching_table_count` unit test | passed |
| `explorer_search_matches_table_names` unit test | passed |
| clippy `-D warnings` | passed |
| Runtime large-schema scroll | pending (app relaunch) |

## Runtime checklist (manual)

1. Connect to a schema with ≥500 tables.
2. Expand connection → schema: Tables should be closed; UI stays responsive.
3. Open Tables: first paint may allocate once; scroll should not hitch from off-screen row widgets.
4. Type a filter: Tables auto-opens, badge shows `matching/total`, truncate hint if >100.
5. Open Views only when needed — no clone while closed.
