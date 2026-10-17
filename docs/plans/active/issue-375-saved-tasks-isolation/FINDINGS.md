# Issue #375 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/375
- Fix: dispatch_sql_task dispatches directly via UiCommand::RunQuery on the task's connection without opening a document, changing active tabs, or hijacking the user's active editor session.
