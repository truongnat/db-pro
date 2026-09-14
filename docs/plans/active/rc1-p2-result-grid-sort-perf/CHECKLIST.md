# CHECKLIST — Result Grid Sorting Allocation & Performance Optimization (#238)

- [x] Create `docs/plans/active/rc1-p2-result-grid-sort-perf/` directory and documentation
- [x] Implement `cell_text_as_str(&UiCell) -> &str` in `crates/ui/src/result_grid.rs`
- [x] Add `looks_like_iso_temporal` date/time heuristic check
- [x] Refactor `compare_ui_cells` to avoid `cell_text` allocations
- [x] Refactor `filtered_sorted_indexes` to use zero-allocation ASCII case-insensitive window filtering
- [x] Add unit and regression tests in `crates/ui/src/result_grid.rs` and `crates/ui/src/app_tests.rs`
- [x] Run Rust quality gates (`cargo check`, `cargo clippy`, `cargo test`)
