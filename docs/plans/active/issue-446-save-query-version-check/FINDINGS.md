# Issue #446 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/446
- Fix: commit_dispatched records the document's buffer version along with the request. on_query_saved only calls mark_saved if the buffer version still matches, preventing edits made while save was in flight from being marked as saved.
