# Plan: Enable `vendored` Feature Flag for `keyring` Dependency

## Context
When building DB Pro on Linux container environments without system `libdbus-1-dev` development headers, building `libdbus-sys` fails during `cargo check` / `cargo test` because `pkg-config` cannot locate `dbus-1.pc`.

## Goal
Add the `"vendored"` feature flag to `keyring` in `Cargo.toml`. This allows `libdbus-sys` to build its vendored C source cleanly without requiring pre-installed system headers in Linux container environments.

## Steps
1. Add `"vendored"` to `keyring` dependency features in `Cargo.toml`.
2. Verify `Cargo.lock` is updated and workspace builds cleanly.
3. Validate all Rust workspace quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`).
