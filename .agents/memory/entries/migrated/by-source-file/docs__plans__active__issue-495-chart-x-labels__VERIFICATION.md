---
schema_version: 1
id: "MEM-SRC-4dcda11107e4fad8"
title: "Source mirror: docs/plans/active/issue-495-chart-x-labels/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-495-chart-x-labels/VERIFICATION.md"
source_sha256: "4dcda11107e4fad8686458d4d66dd8ddd7bb5cb335e328fd78f9214cf74e54d6"
mirror_body_sha256: "4dcda11107e4fad8686458d4d66dd8ddd7bb5cb335e328fd78f9214cf74e54d6"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #495 Verification

- `cargo check -p db-pro-ui` → Finished, no errors (also `db-pro-app` for #510 caller).
- `cargo test -p db-pro-ui` → 687 passed, 0 failed.
- `cargo clippy -p db-pro-ui` → no warnings in touched files.
- `cargo fmt -p db-pro-ui -p db-pro-app -- --check` → clean.
- `bash scripts/check-ui-architecture.sh` → PASS.

## New regression tests
- `x_axis_labels_keep_first_appearance_order_and_data_index`
- `x_axis_labels_empty_points`
- `evenly_spread_indices_include_first_and_last`
