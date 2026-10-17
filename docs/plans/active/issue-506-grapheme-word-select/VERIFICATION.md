# #506 Verification

- `cargo check -p db-pro-ui` → Finished, no errors (also `db-pro-app` for #510 caller).
- `cargo test -p db-pro-ui` → 687 passed, 0 failed.
- `cargo clippy -p db-pro-ui` → no warnings in touched files.
- `cargo fmt -p db-pro-ui -p db-pro-app -- --check` → clean.
- `bash scripts/check-ui-architecture.sh` → PASS.

## New regression tests
- `select_word_keeps_combining_marks_inside_the_word`
- `select_word_handles_decomposed_vietnamese_tone`
- `select_word_at_end_of_buffer_selects_last_word`
