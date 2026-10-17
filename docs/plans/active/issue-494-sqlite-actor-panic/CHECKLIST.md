# Checklist — issue #494

- [x] All handler calls in `run()` wrapped in `catch_unwind(AssertUnwindSafe)`
- [x] Panic → `Err(DbError::Internal(...panic...))` to the responder
- [x] Transaction handlers mapped to `TransactionFailure` (phase Validation, outcome Unknown)
- [x] Actor loop continues after a panic (loop keeps running; `Shutdown` still honored)
- [x] `#[cfg(test)] SqliteCommand::Poison` drives the panic end-to-end
- [x] Test: `dispatch_guarded_converts_panic_into_internal_error`
- [x] `cargo test -p db-pro-infrastructure --lib sqlite` — 36 passed, 0 failed
- [x] `cargo check -p db-pro-infrastructure` — clean
- [x] Test: `actor_survives_panicking_command_and_keeps_serving`
