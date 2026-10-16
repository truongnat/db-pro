# Findings — Workspace Path Confinement

## Root Cause

`absolute_for_relative` joined untrusted input directly to the workspace path. `Path::join` allows parent traversal, and an absolute operand replaces the root. File creation, patching, rename, deletion, and workspace path consumers inherited this behavior. Symlink leaves could also resolve to targets outside the root.

## Fix Strategy

Reject non-relative paths and `ParentDir`/root/prefix components. For candidate operations, canonicalize the nearest existing ancestor and require it to be beneath the canonical root. Restrict names to one normal component. For deletion, inspect with `symlink_metadata` and remove a symlink itself rather than following it; preserve the external target. Never permit deletion of the root directory.

## Scope Decision

This PR addresses the single P1 security boundary in #465. Workspace scan scheduling (#393) is a separate performance concern and is not coupled to path confinement.
