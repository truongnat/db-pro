# Verification — issue #494

- `cargo test -p db-pro-infrastructure --lib sqlite` — **36 passed, 0 failed**
- `cargo check -p db-pro-infrastructure` — **clean** (earlier failures were
  transient sibling edits to `postgres/monitoring.rs` and `core/safety_lexer.rs`,
  unrelated to this change)
  - `dispatch_guarded_converts_panic_into_internal_error` — unit-test of the
    wrapper: panic → `DbError::Internal` containing "panic" and the payload.
  - `actor_survives_panicking_command_and_keeps_serving` — end-to-end: sends
    `SqliteCommand::Poison` (test-only variant), asserts the responder gets an
    `Internal` error mentioning the panic, then issues a real `SELECT 1`
    through the handle to prove the loop is still serving.
