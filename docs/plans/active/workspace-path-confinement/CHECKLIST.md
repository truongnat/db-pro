# Checklist — Workspace Path Confinement

- [x] Reject traversal and absolute paths in root-relative resolution.
- [x] Restrict create and rename names to one path component.
- [x] Prevent deletion of the workspace root.
- [x] Delete symlink leaves without following their targets.
- [x] Add regression tests for traversal, absolute paths, and symlink targets.
- [x] Focused workspace tests, app/native checks, formatting, and diff checks pass.
- [x] Rebase on `origin/main`, commit, push, and open draft PR #528 linked to #465.
