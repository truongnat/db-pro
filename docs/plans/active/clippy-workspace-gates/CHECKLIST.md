# Checklist — Restore Workspace Clippy Gate

- [x] Reproduce workspace Clippy failures on `origin/main`.
- [x] Fix all six diagnostics without changing credential classification or action behavior.
- [x] Run formatting and diff validation.
- [x] Run workspace Clippy with warnings denied.
- [x] Run focused monitoring, saved-task, and dialog modal-guard tests.
- [x] Fix an inherited unused closure argument that prevented CI test compilation under `-D warnings`.
- [x] Push and open draft PR #532; CI and independent review are pending.
