# Checklist — #505

- [x] `row_position` is Option-aware; no 0-default for arrows
- [x] ArrowDown → first visible row when selection not in projection
- [x] ArrowUp → last visible row when selection not in projection
- [x] Home/End/Left/Right keep prior fallback behavior
- [x] Tests in `result_grid/tests/mod_tests.rs`
- [x] `cargo test -p db-pro-app`
- [x] `cargo check -p db-pro-app`
