# Checklist — Restore Workspace Clippy Gate

- [x] Reproduce workspace Clippy failures on immutable PR base `beaa1882a6253d902748c27fba53b99d9969b635`.
- [x] Fix all six diagnostics without changing credential classification or action behavior.
- [x] Run formatting and diff validation.
- [x] Run workspace Clippy with warnings denied.
- [x] Run focused monitoring, saved-task, and dialog modal-guard tests.
- [x] Fix an inherited unused closure argument that prevented CI test compilation under `-D warnings`.
- [x] Push and open draft PR #532; CI and independent review are pending.

## Follow-up — zero-warning gate at `f3fe9708`

- [x] Baseline the gate: `cargo clippy --workspace --all-targets -- -D warnings` fails with 73 diagnostics (65 core / 2 each app, infrastructure, runtime, ui).
- [x] Bump async-trait 0.1.91 → 0.1.92 in Cargo.lock; verify `cargo check --workspace` clean afterward.
- [x] Confirm all 64 `double_must_use` diagnostics clear with the bump alone (no source `allow` added).
- [x] Box `DbError::ConstraintViolation` payload (`ConstraintViolationDetails`); keep Display output byte-identical.
- [x] Update the 13 `ConstraintViolation` construction/pattern sites (infra error adapter, core+infra tests, sqlite integrity test).
- [x] Remove 4 redundant `&` in `format!`/`assert_eq!` (app x2, ui, pg test).
- [x] Replace constant-size `chunks_exact(_mut)` with `as_chunks(_mut)` in theme code + test.
- [x] Rewrite fixture-parser `if let … return None` with `?` in contract conformance test.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` exits 0 with zero diagnostics.
- [x] `cargo +nightly fmt --all -- --check` exits 0.
- [x] `cargo test --workspace` passes (numbers in VERIFICATION.md).
- [x] No new `#[allow]` attributes; pre-existing `DbConnector` `result_large_err` allow untouched.
