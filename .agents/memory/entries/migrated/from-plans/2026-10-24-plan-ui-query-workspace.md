---
schema_version: 1
id: "MEM-PLAN-ui-query-workspace-5e93e92fd904"
title: "Plan archive: Ui Query Workspace"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, ui-query-workspace]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "5e93e92fd90422f78576c51ea42f7923724265d9229b3fefcd20734805bb6727"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/ui-query-workspace/PLAN.md"
  - "docs/plans/active/ui-query-workspace/FINDINGS.md"
  - "docs/plans/active/ui-query-workspace/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/ui-query-workspace/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Query Workspace Visual Hierarchy and Output Composition — #291

## Goal

Refine the Query workspace (editor header, transaction bar, search bar, and output tabs) into a clean, restrained, high-density developer tool layout with canonical button controls and unified design tokens.

## Scope

- Query header action buttons (`Run`, `Stop`, `Builder`, `More actions`) migrated to canonical `Button` primitives with proper variants/sizes.
- Query search bar controls (`ChevronUp`, `ChevronDown`, `Close`) migrated to canonical `Button` primitives.
- Built-in SQL snippet inserters migrated to canonical `Button` primitives.
- Non-regression tests updated to ensure query view surfaces do not re-introduce raw/ad-hoc controls.

## Non-goals

- Rewriting text buffer/cursor logic in `crates/ui/src/editor/`.
- Changing query execution worker architecture.

## Acceptance

- Query header and toolbar buttons use canonical `Button` components.
- Output tab bar and search bar have clean spacing aligned with design tokens.
- All workspace Rust quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`) pass.


## Source: `FINDINGS.md`

# Findings — UI05 Query Workspace

## Audit Findings

1. `crates/ui/src/query_view.rs`:
   - Header `Run` and `Stop` used legacy `primary_button_with_icon` and `secondary_button_with_icon`.
   - `Builder` used legacy `secondary_button`.
   - `More actions` used `compact_icon_button`.
   - Editor find bar used `compact_icon_button` for previous/next match and close.
   - SQL snippets list used legacy `compact_button`.


## Source: `VERIFICATION.md`

# Verification — UI05 Query Workspace

## Test Matrix

- `cargo fmt --all -- --check` -> PASS
- `cargo check --workspace` -> PASS
- `cargo clippy --workspace --all-targets -- -D warnings` -> PASS
- `cargo test -p db-pro-ui` -> PASS (483 passed, 0 failed)
- `cargo build --release --locked -p db-pro-native` -> PASS

## Audit evidence update — 2026-10-13

`docs/plans/_audit-2026-10-13/02-query-editor-workspace.md` verdict: **SUPPORTS** —
canonical `Button` confirmed across all query surfaces (`query_run_control_view.rs`,
`query_search_view.rs`, `query_snippets_surface_view.rs`, `query_output_tabs_view.rs`);
zero legacy button helpers remain in `query_*.rs` (grep-verified); the
non-regression include test covers `query_view.rs`.

Fresh native captures (this audit, `docs/plans/_audit-2026-10-13/captures/`):
`query_*.png`, `query-light_*.png`, `query-row-limit_*.png`, `results_*.png`,
`results-light_*.png`, `table-completion_*.png` — workspace chrome, results dock
with seeded rows (blank-grid fix `294ab089` confirmed visually), and completion
popup at 3 sizes. Remaining: interactive runtime evidence (typing, focus, scroll)
not covered by static captures.

Note: the 483-test count above is a historical total; the db-pro-ui suite is much
larger now (~1,126 lib tests measured 2026-10-13).

