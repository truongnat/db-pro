---
schema_version: 1
id: "MEM-PLAN-workspace-path-confinement-2971450160bd"
title: "Plan archive: Workspace Path Confinement"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, workspace-path-confinement]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "2971450160bd99eb4a98fff84f945eef6f0c3e68193bdb319b81431dbf625788"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/workspace-path-confinement/PLAN.md"
  - "docs/plans/active/workspace-path-confinement/FINDINGS.md"
  - "docs/plans/active/workspace-path-confinement/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/workspace-path-confinement/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Findings — Workspace Path Confinement

## Root Cause

`absolute_for_relative` joined untrusted input directly to the workspace path. `Path::join` allows parent traversal, and an absolute operand replaces the root. File creation, patching, rename, deletion, and workspace path consumers inherited this behavior. Symlink leaves could also resolve to targets outside the root.

## Fix Strategy

Reject non-relative paths and `ParentDir`/root/prefix components. For candidate operations, canonicalize the nearest existing ancestor and require it to be beneath the canonical root. Restrict names to one normal component. For deletion, inspect with `symlink_metadata` and remove a symlink itself rather than following it; preserve the external target. Never permit deletion of the root directory.

## Scope Decision

This PR addresses the single P1 security boundary in #465. Workspace scan scheduling (#393) is a separate performance concern and is not coupled to path confinement.


Root alias review follow-up: the lexical path `.` resolves to the workspace root while remaining a distinct `PathBuf`. Delete and rename now reject empty/current-directory aliases explicitly.


## Source: `VERIFICATION.md`

# Verification — Workspace Path Confinement

- `cargo test -p db-pro-app --lib ide_workspace::tests::`: 5 passed, 0 failed.
- `cargo check -p db-pro-app -p db-pro-native`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- Regression coverage verifies plain and root-qualified traversal, absolute paths, single-component create/rename names (including rejection of multi-component names), refusal to delete or rename the workspace root, and unlinking an external-target symlink without deleting its target, and refusing to traverse a symlinked parent to an outside target.

- `cargo test -p db-pro-app --lib workspace_paths_reject_traversal_and_absolute_paths`: passed (1 test), including empty and `.` root-alias delete/rename rejection.

- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: 15 checks pass, 0 failures; one heuristic clone-count warning (17 clones in this existing file). The added root-id/path snapshots are needed to validate paths without retaining workspace borrows across filesystem mutations and refresh.

