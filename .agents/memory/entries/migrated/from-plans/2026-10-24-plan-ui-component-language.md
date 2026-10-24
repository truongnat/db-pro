---
schema_version: 1
id: "MEM-PLAN-ui-component-language-3ef1a95ff9e4"
title: "Plan archive: Ui Component Language"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, ui-component-language]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "3ef1a95ff9e4bd2331338f74609234567d1e38be32ad2b7667a60d259d9c6333"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/ui-component-language/PLAN.md"
  - "docs/plans/active/ui-component-language/FINDINGS.md"
  - "docs/plans/active/ui-component-language/VERIFICATION.md"
  - "docs/plans/active/ui-component-language/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/ui-component-language/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# UI Component Language — #288 slice

## Goal

Reduce visible control drift on two primary native surfaces by routing their
actions and search through the existing canonical egui components.

## Scope

- Value inspector mode switching and actions in `result_grid_edit.rs`;
- Explorer search and refresh context action in `explorer_view.rs`;
- no new UI framework, token system, or component abstraction.

## Non-goals

- full `legacy.rs` removal;
- every native surface in #288;
- #287 runtime screenshot matrix and light/dark viewport qualification;
- changes to data editing semantics or provider behavior.

## Acceptance

- inspector modes use `SegmentedTabs`;
- inspector actions use canonical `Button` variants/sizes;
- Explorer search uses `SearchInput` and refresh menu uses `ctx_menu_item`;
- existing behavior remains unchanged;
- Rust quality gates pass; UI runtime evidence remains explicitly pending per #287.


## Source: `FINDINGS.md`

# Findings

## P2 — Primary native controls still bypassed canonical components

At baseline `8da3c108ce237811eef6931d6c081cef9bf85447`, the value inspector
used raw `ui.selectable_label` and `ui.button` controls, while the Explorer
toolbar hand-built a search frame/TextEdit and a raw refresh context button.
The repository already provides `SegmentedTabs`, `Button`, `SearchInput`, and
`ctx_menu_item` for these patterns.

This is a contained visual-consistency gap, not a data or provider defect. The
minimal fix is to reuse those primitives on the two named surfaces and leave
the broader #288 inventory for later focused slices.

## P2 — Runtime screenshot provider is unavailable in this environment

At follow-up implementation SHA `78eb8c34506f82c0e8e9221f6a028771e15c2fd7`,
the release native binary launched and introspected a 977-table local fixture,
but the desktop provider reported `screenshot=false`; Orca could not identify
the native window for accessibility capture. This prevents the required
1280×800, 1440×900 and 1920×1080 visual evidence.

The clean-code scan also reports inherited long methods in
`schema_workbench.rs`/`schema_workbench_form.rs`; the follow-up only changes
button composition and does not claim those methods were refactored.

## Follow-up slice — Tasks, Settings Sessions, and Query Output

- `tasks_view.rs`: Migrated manual action buttons (Run, Disable schedule, Schedule 60s, Delete, Save task, Cancel) to canonical `Button` with appropriate size `ButtonSize::Sm` and variants (`Default`, `Secondary`, `Destructive`, `Ghost`).
- `settings_view.rs`: Migrated workspace session management buttons (Save workspace, Restore, Duplicate, Delete) to canonical `Button`.
- `query_output_view.rs`: Migrated EXPLAIN ANALYZE button to canonical `Button` with `enabled` binding.
- `components/mod.rs`: Expanded `primary_native_surfaces_do_not_reintroduce_raw_buttons` test to prevent raw `ui.button` regressions in these files.


## Source: `VERIFICATION.md`

# Verification

## Baseline

- Baseline SHA: `8da3c108ce237811eef6931d6c081cef9bf85447`
- Branch: `fix/ui-component-language`

## Commands executed

- `cargo fmt --all -- --check` — PASS, exit 0.
- `cargo check --workspace` — PASS, exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS, exit 0.
- `cargo test -p db-pro-ui primary_native_surfaces_do_not_reintroduce_raw_buttons` — 1 passed / 0 failed / 482 filtered, exit 0.
- `cargo test --workspace --no-fail-fast 2>&1 | rg '^test result:'` — 1147 passed / 0 failed / 38 ignored across test binaries, exit 0.
- `cargo build --release --locked -p db-pro-native` — PASS, exit 0.
- `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` — 13 pass / 2 warn / 1 fail; inherited long Schema Workbench methods are reported, not introduced by the button slice.

Implementation commits: `ae25aeff6c3e6a9f2edd5d8ac877d55c5db738a0`, `78eb8c34506f82c0e8e9221f6a028771e15c2fd7`.

## Runtime evidence

The native release binary launched and completed local schema introspection
against a 977-table fixture. Screenshot/accessibility capture was attempted but
is blocked: Orca reports no screenshot support and cannot identify the native
window. Therefore the required native screenshots for 1280×800, 1440×900 and
1920×1080, both themes and required states remain pending under #287. No UI
visual pass is claimed.

## Provider matrix

Not applicable; this slice changes only native control composition and does not
change database/provider behavior.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — #288 UI component language slice

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · focused native UI refactor lane |
| Issue(s) | #288, dependent on #287 |
| Task state | Review |
| Baseline SHA | `8da3c108ce237811eef6931d6c081cef9bf85447` |
| Branch / PR | `fix/ui-component-language` · PR not opened |
| Scope interpretation | Migrate value inspector and Explorer toolbar controls to existing canonical components. |
| Out of scope | full legacy cleanup, other UI surfaces, runtime screenshots, provider behavior |

## 2. Progress checkpoint

- Current HEAD: `78eb8c34506f82c0e8e9221f6a028771e15c2fd7`.
- Completed acceptance rows: component inventory; value inspector migration; Explorer migration; Schema Workbench action migration; raw-button guard; Rust gates; native release build.
- Remaining acceptance rows: clean-code scan clean, runtime evidence and broader #288 closure.
- Findings / risks: P2 runtime screenshot blocker and inherited long Schema Workbench methods; see `FINDINGS.md`.
- Tests already run: UI guard → 1 passed / 0 failed / 482 filtered; workspace → 1147 passed / 0 failed / 38 ignored; all exit 0.
- Dependency / blocker changes: #287 runtime evidence remains blocked/skipped by explicit user direction.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `78eb8c34506f82c0e8e9221f6a028771e15c2fd7` |
| Commit list | `ae25aef refactor(ui): consolidate primary native controls`; `78eb8c3 refactor(ui): standardize schema workbench actions` |
| File / surface inventory | `result_grid_edit.rs` — SegmentedTabs and canonical Button actions; `explorer_view.rs` — SearchInput and canonical refresh context row; `schema_workbench.rs`/`schema_workbench_form.rs` — canonical docs and DDL confirmation actions; `components/mod.rs` — raw-button regression guard |
| Acceptance mapping | #288 control-language slice → four primary surfaces and guard; quality/runtime evidence → `VERIFICATION.md` |
| Commands and counts | fmt/check/clippy pass; workspace 1147/0/38; native release build pass; clean-code 13/2/1 with inherited method warnings |
| CI run IDs / status | not run |
| Known limitations | runtime evidence and broader #288 inventory remain open |
| Migrations / config implications | none expected |
| Out-of-scope changes | #287 runtime screenshots; full legacy cleanup; provider behavior |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `78eb8c34506f82c0e8e9221f6a028771e15c2fd7` |
| Verdict | `ACCEPT WITH P2` — implementation slice is green; #288 runtime/remaining-surface work stays open |
| P0 / P1 / P2 counts | introduced-by-this-SHA: 0 / 0 / 0; inherited/open P2: 2 |
| Findings | raw-button guard passes; runtime screenshot provider unavailable; inherited long methods remain |
| CI disposition | CI not run; local gates recorded in `VERIFICATION.md` |
| Next task(s) unblocked | none until runtime gate is addressed |

## 5. Research / audit handoff

- Source date: 2026-09-16
- Source URLs / references: issue #288; `crates/ui/src/components/{button,input,tabs,overlay}.rs`; `crates/ui/src/{result_grid_edit,explorer_view}.rs`
- Factual findings: recorded in `FINDINGS.md` against the baseline SHA.
- Inference: existing primitives are sufficient for this bounded migration slice.
- Decision / recommendation: reuse existing primitives; do not create a new abstraction.
- Unresolved questions: runtime screenshots and remaining raw-control inventory.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã hoàn tất thêm Schema Workbench vào slice #288 và thêm guard chống raw
`ui.button`; workspace 1147/0/38 và release build pass. Runtime screenshot
không lấy được do desktop provider thiếu screenshot/native-window accessibility,
nên #288 chưa thể đóng; còn inherited long methods cần xử lý riêng.

