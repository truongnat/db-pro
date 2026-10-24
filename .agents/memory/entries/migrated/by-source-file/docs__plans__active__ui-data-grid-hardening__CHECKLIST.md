---
schema_version: 1
id: "MEM-SRC-6dbfe2c0c54a78dc"
title: "Source mirror: docs/plans/active/ui-data-grid-hardening/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-data-grid-hardening/CHECKLIST.md"
source_sha256: "6dbfe2c0c54a78dc3f44787435f528669463086f36ad1037e3fb4ac05d6a0bab"
mirror_body_sha256: "6dbfe2c0c54a78dc3f44787435f528669463086f36ad1037e3fb4ac05d6a0bab"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — UI06 Data Grid Hardening

- [x] Standardize result grid toolbar actions (`Copy Cell`, `Copy Row`, `CSV`, `JSON`, `Record`, `Inspect`) in `result_grid_view.rs`.
- [x] Standardize table data toolbar actions (`Add Row`, `Apply`, `Discard`, `Reload Row`, `Discard Local Change`, `Resolve Conflict`, `Retry`) in `table_editor_view.rs`.
- [x] Standardize table data pagination controls (`First`, `Previous`, `Next`, `Last`) in `table_editor_view.rs`.
- [x] Extend non-regression tests in `crates/ui/src/components/mod.rs`.
- [x] Run full workspace quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, `cargo build --release --locked -p db-pro-native`).
- [x] Document findings and verification.
