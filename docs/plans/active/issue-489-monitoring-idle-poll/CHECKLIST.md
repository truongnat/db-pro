# Checklist — #489

- [x] `prepare_frame` calls `request_repaint_after(remaining.max(50ms))` when
      Monitor + poll enabled + connected
- [x] Single cadence source `MONITORING_POLL_INTERVAL` shared by poll + schedule
- [x] `monitoring_poll_remaining` unit-tested (never/just/mid/overdue/future)
- [x] Code-path test asserts prepare_frame schedules the repaint
- [x] Audited other refresh loops (security/audit/editor) — none periodic
- [x] `cargo check`/`cargo test` on db-pro-app
