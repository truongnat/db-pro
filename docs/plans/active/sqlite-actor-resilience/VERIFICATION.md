# Verification

State: REVIEW.
Issues: truongnat/db-pro#493 and #494.

- `cargo test -p db-pro-infrastructure --lib sqlite::actor::tests`: 6 passed, 0 failed.
- `cargo check -p db-pro-core -p db-pro-infrastructure`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- Clean-code scan: 100/100, no findings.
