# Findings — #501

- `MonitorSession.backend_start` already exists and `LIST_SESSIONS_SQL` already
  returns `backend_start::text` — no query change needed upstream.
- Race surface: snapshot → user reads dialog → clicks confirm → runtime spawns
  `pg_terminate_backend(pid)`. A fast client reconnect can reuse the pid.
- Server-side guard is the only correct level: any UI/service-side re-check is
  still check-then-act. The WHERE clause folds the verification into the same
  statement as the signal.
- `pg_cancel_backend`/`pg_terminate_backend` only accept a pid; filtering
  `pg_stat_activity` rows first means a mismatched `backend_start` returns zero
  rows → `Ok(false)` (refused), never a signal to the wrong backend.
- `backend_start` for `client backend` rows is never NULL, but the code treats
  it as missing-information anyway and refuses rather than silently signalling.
- Struct-variant payloads on `MonitoringConfirmationAction` dropped the `Copy`
  derive; no other site relied on it.
