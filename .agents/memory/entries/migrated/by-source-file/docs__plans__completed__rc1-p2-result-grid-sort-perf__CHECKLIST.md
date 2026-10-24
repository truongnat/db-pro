---
schema_version: 1
id: "MEM-SRC-16a9255aa03a7da7"
title: "Source mirror: docs/plans/completed/rc1-p2-result-grid-sort-perf/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/rc1-p2-result-grid-sort-perf/CHECKLIST.md"
source_sha256: "16a9255aa03a7da70283d3954a922eae0a75a8fca17aaa7b711036a33e51d5d3"
mirror_body_sha256: "16a9255aa03a7da70283d3954a922eae0a75a8fca17aaa7b711036a33e51d5d3"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# CHECKLIST — Result Grid Sorting Allocation & Performance Optimization (#238)

- [x] Create `docs/plans/active/rc1-p2-result-grid-sort-perf/` directory and documentation
- [x] Implement `cell_text_as_str(&UiCell) -> &str` in `crates/ui/src/result_grid.rs`
- [x] Add `looks_like_iso_temporal` date/time heuristic check
- [x] Refactor `compare_ui_cells` to avoid `cell_text` allocations
- [x] Refactor `filtered_sorted_indexes` to use zero-allocation ASCII case-insensitive window filtering
- [x] Add unit and regression tests in `crates/ui/src/result_grid.rs` and `crates/ui/src/app_tests.rs`
- [x] Run Rust quality gates (`cargo check`, `cargo clippy`, `cargo test`)
