# Verification

State: REVIEW.
Issues: truongnat/db-pro#496 and #447.

- `cargo test -p db-pro-app --lib table_fk_lookup_tests`: 2 passed, 0 failed.
- `cargo check -p db-pro-app -p db-pro-native`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- Clean-code scan: 100/100, no findings.
