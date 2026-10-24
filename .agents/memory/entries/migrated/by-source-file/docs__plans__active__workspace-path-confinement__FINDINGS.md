---
schema_version: 1
id: "MEM-SRC-9803fb1154f652c9"
title: "Source mirror: docs/plans/active/workspace-path-confinement/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/workspace-path-confinement/FINDINGS.md"
source_sha256: "9803fb1154f652c998daadbffccae344f6002cba154f3d67f60ec54e6abff2f7"
mirror_body_sha256: "9803fb1154f652c998daadbffccae344f6002cba154f3d67f60ec54e6abff2f7"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — Workspace Path Confinement

## Root Cause

`absolute_for_relative` joined untrusted input directly to the workspace path. `Path::join` allows parent traversal, and an absolute operand replaces the root. File creation, patching, rename, deletion, and workspace path consumers inherited this behavior. Symlink leaves could also resolve to targets outside the root.

## Fix Strategy

Reject non-relative paths and `ParentDir`/root/prefix components. For candidate operations, canonicalize the nearest existing ancestor and require it to be beneath the canonical root. Restrict names to one normal component. For deletion, inspect with `symlink_metadata` and remove a symlink itself rather than following it; preserve the external target. Never permit deletion of the root directory.

## Scope Decision

This PR addresses the single P1 security boundary in #465. Workspace scan scheduling (#393) is a separate performance concern and is not coupled to path confinement.


Root alias review follow-up: the lexical path `.` resolves to the workspace root while remaining a distinct `PathBuf`. Delete and rename now reject empty/current-directory aliases explicitly.
