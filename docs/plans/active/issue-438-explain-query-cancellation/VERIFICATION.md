# Issue #438 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/438
- Fix: ExplainQuery inserts request_id into query_cancellations and selects on cancel_rx, allowing long-running EXPLAIN ANALYZE queries to be cancelled cleanly.
