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
- Delete symlink leaves without following their targets; disallow deleting the workspace root.
- Add regressions for traversal, absolute paths, and external symlink targets.
