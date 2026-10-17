# Issue #473 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/473
- Fix: MonitoringState::reset_connection_scope resets snapshot, error, last poll, and confirms on disconnect and connect to prevent monitoring state from lingering across connection switches.
