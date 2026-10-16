# Verification

State: REVIEW.
Issues: truongnat/db-pro#445 and #509.

- `cargo test -p db-pro-ui --lib unicode_search`: 2 passed.
- `cargo test -p db-pro-ui --lib query_document`: 10 passed.
- `cargo test -p db-pro-ui --lib query_document_search_keeps_unicode_expansion_offsets_in_original_text`: 1 passed.
- `cargo check -p db-pro-ui -p db-pro-app -p db-pro-native`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- Clean-code scan: 99/100, 0 errors; one pre-existing LONG_FUNCTION warning for `renderer.rs::paint_search_and_execution`.
