# Verification

State: REVIEW.
Issues: truongnat/db-pro#480 and #505.

- `cargo test -p db-pro-ui --lib keyboard_navigation_recovers_when_option_list_shrinks`: 1 passed.
- `cargo test -p db-pro-app --lib grid_keyboard_navigation`: 3 passed.
- `cargo check -p db-pro-ui -p db-pro-app -p db-pro-native`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- Clean-code scan: 100/100, no findings.
