# Issue #439 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/439
- Fix: AuditState::reset_connection_scope clears audit_page, filters, selections, and bookmarks upon disconnect and connect, preventing audit events from leaking across connections.
