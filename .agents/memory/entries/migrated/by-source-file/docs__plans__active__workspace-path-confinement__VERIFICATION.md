---
schema_version: 1
id: "MEM-SRC-8d66506de56f9c4d"
title: "Source mirror: docs/plans/active/workspace-path-confinement/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/workspace-path-confinement/VERIFICATION.md"
source_sha256: "8d66506de56f9c4d4f695db7f3f14639c2e2a391e2dcabf3762e0d5a030c3e88"
mirror_body_sha256: "8d66506de56f9c4d4f695db7f3f14639c2e2a391e2dcabf3762e0d5a030c3e88"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — Workspace Path Confinement

- `cargo test -p db-pro-app --lib ide_workspace::tests::`: 5 passed, 0 failed.
- `cargo check -p db-pro-app -p db-pro-native`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- Regression coverage verifies plain and root-qualified traversal, absolute paths, single-component create/rename names (including rejection of multi-component names), refusal to delete or rename the workspace root, and unlinking an external-target symlink without deleting its target, and refusing to traverse a symlinked parent to an outside target.

- `cargo test -p db-pro-app --lib workspace_paths_reject_traversal_and_absolute_paths`: passed (1 test), including empty and `.` root-alias delete/rename rejection.

- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: 15 checks pass, 0 failures; one heuristic clone-count warning (17 clones in this existing file). The added root-id/path snapshots are needed to validate paths without retaining workspace borrows across filesystem mutations and refresh.
