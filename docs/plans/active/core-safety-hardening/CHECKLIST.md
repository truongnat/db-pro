# Checklist

- [x] Add Explain safety regression test for read-only `EXPLAIN ANALYZE DELETE`.
- [x] Enforce zero-row table mutations as a domain error.
- [x] Add SQLite interrupt handle and timeout-aware query execution.
- [x] Cover SQLite timeout and post-timeout actor recovery.
- [x] Remove SSH host-key verification bypass from tunnel and test commands.
- [x] Preserve SSH configuration through PostgreSQL backup/restore and route commands via the tunnel.
- [x] Make SSH tunnel startup wait for a listening local forward and report early exit.
- [x] Reject primary-key updates at the core mutation boundary.
- [x] Add timeout coverage for external PostgreSQL backup commands.
- [x] Preserve connection handles when disconnect fails.
- [x] Reuse SSH-routed effective configuration for connection testing.
- [x] Run formatting, check, clippy, targeted tests, and workspace tests.
- [x] Self-review changed files and confirm no UI/native files changed.
- [ ] Record PostgreSQL live evidence as pending unless an actual provider is exercised.
