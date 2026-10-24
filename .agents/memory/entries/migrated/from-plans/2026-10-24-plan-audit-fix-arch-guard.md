---
schema_version: 1
id: "MEM-AUDIT-fix-arch-guard-00137d339859"
title: "Plan archive: Audit Fix Arch Guard"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, audit-fix-arch-guard]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "00137d339859e855940e7fb1388d2cb2e3ed6190ae51a612c548ff2de6d6d151"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/_audit-2026-10-13/fix-arch-guard.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ audit slice `docs/plans/_audit-2026-10-13/fix-arch-guard.md`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

# Fix — Architecture boundary guard (slice 06 P1)

`bash scripts/check-ui-architecture.sh` failed on HEAD in two ways: the
`DbProApp` allowlist lacked `dev_tools`/`capture_evidence_seeded` (added by
`0bb549a5`), and — masked behind that exit — a `*_state.rs` module test
constructed `DbProApp`, tripping the state-module boundary check.

## Changes

- `scripts/check-ui-architecture.sh`
  - Extended `expected_fields` with `dev_tools` and `capture_evidence_seeded`
    (legitimate composition-root state: `dev_tools` is `#[cfg(debug_assertions)]`,
    `capture_evidence_seeded` is capture-provenance plumbing owned by the app root).
    Guard semantics unchanged — still a diffed allowlist.
  - Removed stale `explicit_state_modules` entry `shell_output_panel_view.rs`
    (file deleted in `3c662ba3`; caused an `rg` IO warning).
- `crates/ui/src/query_output_state.rs`
  - Moved `clicking_a_dock_tab_switches_the_output_pane` out — it is a full-app
    egui click-harness test, not state logic, and its `DbProApp` use violated the
    "feature state modules must not depend on DbProApp" rule. Dropped the
    now-unused `use crate::DbProTheme`.
- `crates/ui/src/app_tests.rs`
  - Added the moved test verbatim (all names already in scope via `super::*`).

## Verification

- `bash scripts/check-ui-architecture.sh` → `UI architecture boundary: PASS`, exit 0.
- `cargo check -p db-pro-ui` → clean (an earlier failure during this run was a
  sibling's in-flight `query_editor_surface_view.rs` edit; re-check passed).
- `cargo test -p db-pro-ui clicking_a_dock_tab_switches_the_output_pane` → 1 passed.
- `cargo test -p db-pro-ui query_output_state` → 3 passed.
- `cargo fmt --all -- --check` → only `crates/core/src/application/object_mutation_service.rs`
  diffs (sibling-owned, not mine).

## Notes / leftovers

- The moved test exercises the same assertion (`active_tab_for_document` ==
  `OutputTab::History` after clicking the dock History tab); behavior identical.
- CI step `ci.yml:116` should now pass for the guard.

