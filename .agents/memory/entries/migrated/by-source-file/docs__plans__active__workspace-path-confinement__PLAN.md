---
schema_version: 1
id: "MEM-SRC-1dff23b5e63f97e2"
title: "Source mirror: docs/plans/active/workspace-path-confinement/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/workspace-path-confinement/PLAN.md"
source_sha256: "1dff23b5e63f97e29bd245e9caf02f1d9409e524be89fa81f05b7dd66b80d380"
mirror_body_sha256: "1dff23b5e63f97e29bd245e9caf02f1d9409e524be89fa81f05b7dd66b80d380"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Plan — Workspace Path Confinement

## Objective

Fix issue #465 so workspace file operations cannot escape configured workspace roots through traversal, absolute paths, or symlinks.

## Target Branch

`fix/workspace-path-confinement`

## Severity

P1 — workspace APIs could read, write, rename, or delete paths outside the workspace.

## Scope

- Validate plain and root-qualified relative paths; reject absolute paths and parent traversal.
- Resolve the nearest existing ancestor canonically and require it to remain within the canonical root.
- Require create/rename names to be a single path component.
- Delete through no-follow directory handles so parent symlink swaps cannot escape the root; delete symlink leaves without following their targets.
- Disallow deleting or renaming the workspace root, including empty and `.` aliases.
- Add regressions for traversal, absolute paths, and external symlink targets.
