# Verification — #501

- `cargo check` clean for db-pro-app, db-pro-core, db-pro-infrastructure,
  db-pro-runtime, db-pro-ui, db-pro-native.
- `cargo test -p db-pro-app` monitoring tests: confirm actions dispatch
  `UiCommand::{MonitoringCancelBackend,MonitoringTerminateBackend}` carrying
  `expected_backend_start`; `backend_start: None` dispatches nothing and sets
  feedback.
- `cargo test -p db-pro-core` monitoring_service: positive-pid validation kept,
  `cancel`/`terminate` reject empty backend_start, mock asserts the arg reaches
  the port.
- `cargo test -p db-pro-infrastructure` monitoring: SQL contract asserts
  `a.pid = $1 AND a.backend_start = $2::timestamptz` in both statements.
- Live-PG negative path (mismatched backend_start → Ok(false), no signal) rests
  on SQL semantics; no DB fixture in this repo's unit layer.
