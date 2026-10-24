---
schema_version: 1
id: "MEM-PLAN-ui-shell-hierarchy-cbdfb82901bf"
title: "Plan archive: Ui Shell Hierarchy"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, ui-shell-hierarchy]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "cbdfb82901bf2581d1a6998878e7105ecec7b0b4172d76ce0b6a9e199867492d"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/ui-shell-hierarchy/PLAN.md"
  - "docs/plans/active/ui-shell-hierarchy/FINDINGS.md"
  - "docs/plans/active/ui-shell-hierarchy/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/ui-shell-hierarchy/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Shell, Activity Bar, Top Bar, and Sidebar Hierarchy — #289

## Goal

Harden the native shell, top bar, activity bar, and sidebar hierarchy so that DB Pro feels like a coherent, restrained, and stable database IDE at a glance.

## Scope

- Top bar layout and information hierarchy (active connection breadcrumb, driver badge, quick search trigger, action buttons).
- Activity Bar layout, spacing, tooltips, active states, and keyboard/mouse ergonomics.
- Sidebar header (workspace/connection dropdown, new connection, palette search, new query button).
- Sidebar resize grip, min/max limits, smooth divider lines, and clean integration with the CentralPanel.
- Ensure all components use canonical tokens (`DbProTheme`, `tokens.rs`, `Button`, `Badge`).

## Non-goals

- Rewriting internal panels of all individual activities (Explorer tree in #290, Query workspace in #291, Data Grid in #292).
- Adding complex new window frame management.

## Acceptance

- Shell hierarchy is visually crisp, consistent, and uses canonical design tokens without hardcoded magic numbers where standard tokens exist.
- Sidebar header actions, breadcrumbs, search triggers, and new query button use canonical component styles.
- Rust quality gates (`cargo check --workspace`, `cargo clippy`, `cargo test`) pass.


## Source: `FINDINGS.md`

# Findings — UI03 Shell Hierarchy

## Audit Findings

1. `crates/ui/src/sidebar_view.rs`:
   - The "+ New query" button was custom painted with manual painter commands instead of using the canonical button tokens or standard button styling.
   - The workspace selector dropdown used raw `interact` with manual background repaint instead of canonical interactive component styling.
   - Hardcoded sizes and magic numbers (e.g. `FontId::proportional(13.5)`, `Rounding::same(6.0)`, etc.) where `tokens.rs` / `DbProTheme` provide unified design tokens.

2. `crates/ui/src/navigation_view.rs`:
   - The driver tag badge was hand-crafted with raw `egui::Frame` instead of using the canonical `Badge` component or standardized tokens.
   - Topbar search box center placeholder and sizing had slight alignment inconsistencies with standard inputs.
   - Statusbar buttons used legacy `compact_icon_button` where `Button::new(theme).icon(...).variant(ButtonVariant::Ghost).size(ButtonSize::IconSm)` is canonical.

3. `crates/ui/src/activity_bar_view.rs`:
   - Bottom settings button spacing had `ui.available_height() - 44.0` calculation that could jitter on resize.


## Source: `VERIFICATION.md`

# Verification — UI03 Shell Hierarchy

## Test Matrix

- `cargo fmt --all -- --check` -> PASS
- `cargo check --workspace` -> PASS
- `cargo clippy --workspace --all-targets -- -D warnings` -> PASS
- `cargo test --workspace` -> PASS (483 passed, 0 failed in db-pro-ui)
- `cargo build --release --locked -p db-pro-native` -> PASS

