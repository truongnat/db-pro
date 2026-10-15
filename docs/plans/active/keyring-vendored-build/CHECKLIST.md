# Checklist: keyring Vendored Feature Flag

- [x] Add `"vendored"` to `keyring` features in `Cargo.toml`
- [x] `cargo check --workspace` passes cleanly
- [x] `cargo fmt --all -- --check` passes
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [x] `cargo test --workspace` passes (1113 tests pass)
