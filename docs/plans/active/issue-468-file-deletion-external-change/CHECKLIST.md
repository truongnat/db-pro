# Issue #468 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/468
- Fix: check_external_file_changes checks if a file known in workspace_file_mtimes no longer returns an mtime (deleted on disk) and flags it into workspace_external_changes.
