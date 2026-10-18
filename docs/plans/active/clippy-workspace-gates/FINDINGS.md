# Findings — Restore Workspace Clippy Gate

CI failed because `cargo clippy --workspace --all-targets -- -D warnings` reported:
1. `unused_imports`: `db_pro_core::ports::DbConnector` in `crates/infrastructure/src/sqlserver/introspect.rs`.
2. `manual_div_ceil`: manual division arithmetic in `crates/ui/src/diagram/spatial.rs`.
3. `needless_borrows_for_generic_args`: string reference borrow in `crates/ui/src/editor/tests/buffer_tests.rs`.

Since CI treats Clippy warnings as errors (`-D warnings`), any open PR or CI run failed.

The fixes are mechanical:
- Removed the unused import `DbConnector` in `sqlserver/introspect.rs`.
- Converted manual division arithmetic to `.div_ceil(...)` in `diagram/spatial.rs`.
- Removed unnecessary reference borrow `&format!(...)` in `editor/tests/buffer_tests.rs`.
