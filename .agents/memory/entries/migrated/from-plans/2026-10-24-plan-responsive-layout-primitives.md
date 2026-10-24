---
schema_version: 1
id: "MEM-PLAN-responsive-layout-primitives-8dc3ec04f5eb"
title: "Plan archive: Responsive Layout Primitives"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, responsive-layout-primitives]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "8dc3ec04f5ebc7d3bdd4c91e6913c87921fe951d889f7e511db07701ed11a652"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/responsive-layout-primitives/PLAN.md"
  - "docs/plans/active/responsive-layout-primitives/FINDINGS.md"
  - "docs/plans/active/responsive-layout-primitives/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/responsive-layout-primitives/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Responsive Layout Primitives

## State

- **State:** IMPLEMENTING
- **Branch:** `feature/responsive-layout-primitives`
- **Baseline SHA:** `44d157e2e4c6ad9d83be9ec69414f508f91d38d8`
- **Surface:** Native Rust `egui` UI (`crates/ui`), starting with Component Gallery forms.

## Goal

Provide a small, reusable native layout API—Container, Row, responsive Grid, and Col-like sizing—that keeps content inside the actual available `egui::Ui` rectangle and adapts column count/width when the workspace or sidebar changes size. Replace repeated view-local fixed breakpoints for the pilot surfaces.

## Evidence and failure scenario

- At baseline SHA `44d157e2e4c6ad9d83be9ec69414f508f91d38d8`, `component_gallery_inputs.rs:4,109-120` uses a local `680.0` breakpoint and repeats separate one-/two-column rendering paths.
- At the same SHA, `component_gallery_view.rs:230-248` owns a gallery-specific max-width/inset calculation instead of a shared layout contract.
- The supplied Component Gallery screenshot (2026-09-26) shows a connection form whose right column is clipped by the viewport after available workspace width is reduced. This motivates an explicit minimum column width/wrapping rule and container clipping/measurement contract.
- `crates/ui/src/components/mod.rs` exports individual primitives, but there is no shared responsive container/grid module. Existing layouts use local `ui.columns`, `horizontal_wrapped`, `egui::Grid`, and hand-written width checks.

## Severity and scope

- **Severity:** P2 UI consistency/responsiveness; promote to P1 only if a confirmed core workflow becomes unusable or controls become inaccessible.
- **In scope:** Native layout primitives; deterministic width/column calculations; unit tests; migrate the Component Gallery's connection form and its outer content container as the first consumers; document usage and constraints.
- **Out of scope:** Web/CSS/Tailwind/React implementation; copying every Bootstrap utility or CSS breakpoint; changing database behavior; migrating every view in one change; redesigning product surfaces unrelated to the pilot.

## Design direction

`egui` is immediate-mode and has no DOM/CSS cascade. Layout decisions must be based on the current `Ui`'s available width, in logical points, each frame. Components must not claim a minimum width larger than the parent or use `set_min_width` as a substitute for a responsive maximum.

Proposed responsibilities (exact public builder syntax is an implementation task and must be reviewed before broad migration):

- **Container:** fluid by default; default horizontal gutter uses `SPACE_LG`; optional max width applies to content after gutters; content is bounded by local available width and centered only when space remains.
- **Row:** owns a token-based/overridable horizontal gap and wrapping policy; children flow to a subsequent row rather than clip.
- **Col:** declares a preferred/minimum width and optional span within its Row; never exceeds the allocated cell. V1 does not ship the full Bootstrap 12-span named-breakpoint matrix.
- **ResponsiveGrid:** equal-width convenience built on Row/Col; chooses the largest column count fitting `min_cell_width` + default `SPACE_LG` gap, bounded by optional `max_columns`. Do not use `egui::Grid` for this, since that API is tabular.
- **Measurement core:** pure functions for available width → effective content width, column count, and cell width. Treat NaN/infinity/negative width or gaps as zero; allow zero min-width but always cap columns to at least one; clamp oversized gutter/gap to available geometry; test every boundary.

Do not introduce a general layout engine or store measured sizes in app state. Avoid per-frame allocations where a simple iterator/`ui.columns` or fixed-capacity calculation suffices. Tokens remain the source of standard spacing; callers may override gaps intentionally.

## Task decomposition / owners

1. **Layout API and measurement contract — Coder A (In Progress)**
   - Propose and implement Container + pure width/column calculations in a focused module.
   - Add boundary/unit tests, public exports, and API docs.
   - No gallery edits in this task.
2. **Row/Col + responsive Grid — Coder B**
   - Build on the approved measurement contract; define wrap and span semantics and demonstrate 1→2→N columns without overflow.
   - Add unit tests for wrap, gap, min-width, max-columns and narrow/zero available space.
   - Coordinate on API before editing shared exports; do not parallel-edit `components/mod.rs` with Task 1.
3. **Pilot integration — Coder C**
   - Replace Component Gallery outer fixed-width logic and connection form's duplicated 680px branches with the new primitives.
   - Keep field ordering, validation/focus behavior, and existing controls unchanged.
   - Add a layout regression test for narrow viewport/sidebar-open geometry.
4. **Independent review — Reviewer**
   - Inspect API complexity, overflow paths, intrinsic child minimums, keyboard/focus behavior, allocations, and diff scope.
5. **Verification — Tester**
   - Run focused UI tests, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and the required native release build; record actual outcomes.
   - Capture the pilot UI with sidebar closed/open and at 1280×800, 1440×900, 1920×1080. Record unavailable viewport/runtime evidence honestly.

**Dependency/order:** Task 1 → API review → Task 2 → Task 3 → independent review/test/runtime verification. The feature owner approves the public API after Task 1; only then may Task 2 start. Do not run Coder A/B/C concurrently against the same checkout; parallelize only isolated research/test tasks or use isolated worktrees.

## Acceptance criteria

- [ ] Container content width never exceeds the available parent width; max width and gutters behave at narrow and wide widths.
- [ ] Responsive grid chooses cells that fit: no horizontal overflow at tested widths; wraps to fewer columns when available width falls below the required minimum.
- [ ] Row/Col API supports documented responsive layouts without repeated caller-side `if available_width >= ...` branches for the pilot.
- [ ] Gallery form field order is row-major at all widths: Display name → Database → Host → Password → Port → SSL; labels, validation, keyboard navigation, and actions remain intact while columns reflow.
- [ ] Regression tests measure actual child response bounds against the parent clip/available rect, including narrower-than-minimum and threshold-adjacent widths; sizing math alone is not accepted as proof of rendered bounds.
- [ ] No layout decision reads global screen width when local `Ui::available_width()` is the relevant constraint.
- [ ] No database/provider impact; feature is provider-independent.
- [ ] Required Rust gates and native UI evidence are recorded in `VERIFICATION.md`; no runtime claims without captures.

## Risks and decisions

- `egui::Ui` child allocation/layout direction can make a visually intuitive `Row/Col` wrapper overflow if it relies on intrinsic minimum sizes. Prototype and test the actual allocation contract before broad adoption.
- A 12-column Bootstrap clone could add ceremony without value for native desktop layouts. Keep span support optional; favor min-width-driven responsive grids by default.
- Existing uncommitted UI edits were present when this branch was created and remain carried in the worktree. They are not automatically part of this feature; inspect and separate them before any commit/PR.
- The older active plans `ui-shell-hierarchy` and `sidebar-dbeaver-codex-layout` concern shell/sidebar layout, not reusable responsive primitives. `component-gallery-redesign` is in RUNTIME_VERIFY; use Gallery only as pilot and avoid claiming that plan's completion gates here.

## PostgreSQL / SQLite impact

Not applicable: layout-only UI primitives; no provider/database calls or mutations.


## Source: `FINDINGS.md`

# Responsive Layout Primitives — Findings

## Baseline

- **Source SHA:** `44d157e2e4c6ad9d83be9ec69414f508f91d38d8`
- **Working branch:** `feature/responsive-layout-primitives`
- **Source evidence date:** 2026-09-26

## Findings

### P2 — Responsive layout rules are duplicated per view

- Evidence: `crates/ui/src/component_gallery_inputs.rs:4,109-120` defines a local `680.0` threshold and separate rendering branches for one and two columns.
- Failure scenario: when a sidebar reduces the local available width, caller-specific thresholds and widget intrinsic minimums can disagree; the right side may clip rather than reflow consistently.
- Scope decision: establish a reusable local-Ui width contract and migrate only the Gallery form as proof.

### P2 — Gallery content container has a bespoke width policy

- Evidence: `crates/ui/src/component_gallery_view.rs:230-248` applies local horizontal inset and a `1120.0` max width.
- Failure scenario: a fixed max-width policy detached from allocated local width can allow the content child to exceed its actual viewport if parent min/max constraints disagree.
- Scope decision: use a shared fluid bounded Container, but avoid changing unrelated gallery category navigation.

### No database/provider risk identified

This feature affects layout only. PostgreSQL and SQLite behavior are both N/A; no SQL, persistence, runtime worker or provider capability changes are planned.

## Open questions for API owner

1. Can the Row/Col closure API support heterogeneous one-shot children without boxed per-frame allocations?
2. Can its allocated child `Response::rect` be tested against the local parent clip rect at narrow widths?

## Decisions

- V1 includes the requested Container, Row, Col and equal-width ResponsiveGrid concepts, but **not** Bootstrap's full 12-column/named-breakpoint utility system.
- Default horizontal gutter and row/grid gap use `SPACE_LG`; callers can override deliberately.
- Gallery form order is row-major and remains constant across breakpoints: Display name → Database → Host → Password → Port → SSL.
- API owner must show no-overflow behavior for actual rendered child response bounds before broad migration.


## Source: `VERIFICATION.md`

# Responsive Layout Primitives — Verification

## State

IMPLEMENTING. Primitive implementation has started; feature gates and runtime evidence remain pending.

## Historical planning snapshot commands

The following entries describe the earlier planning snapshot only; they are superseded by the integrated verification below.

## Commands actually executed at planning snapshot

- `git status --short --branch` — branch and existing dirty worktree inspected; feature branch created.
- Source inspection of Component Gallery and input layout modules — planning evidence only, not runtime evidence.

## Automated gates

- `cargo fmt --all -- --check` — BLOCKED by pre-existing unresolved `crates/ui/src/components/form/config.rs` module reference.
- `cargo check --workspace` — NOT RUN for this feature.
- `cargo clippy --workspace --all-targets -- -D warnings` — NOT RUN.
- `cargo test -p db-pro-ui responsive_layout` — FAILED on pre-existing unrelated missing modules/type inference in `form`, `input`, `overlay`, `selection`, `table`, `toggle`, `tree`, and other UI files; no responsive-layout diagnostics remained.
- `cargo build --release --locked -p db-pro-native` — NOT RUN.

## Runtime evidence

Not collected. Required native captures are sidebar open/closed at 1280×800, 1440×900 and 1920×1080. Source inspection is not runtime evidence.

## Provider matrix

- PostgreSQL: N/A — layout-only feature.
- SQLite: N/A — layout-only feature.

## Integrated component-refactor verification

Executed in the working tree based on HEAD `5b38eb6543d5fa66783fe7f772e52b97665183a6` plus uncommitted changes:

- `cargo test -p db-pro-ui responsive_layout --lib`: PASS (6 passed, 0 failed, 905 filtered out).
- `cargo test -p db-pro-ui --lib`: PASS (911 passed, 0 failed, 0 ignored).
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace --quiet`: PASS (1,566 passed, 0 failed, 41 ignored; doc-test suites contained 0 tests).
- `cargo fmt --all -- --check`: PASS.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `git diff --check`: PASS.

## Limitations / blockers

- Native runtime captures/accessibility traversal at 1280×800, 1440×900, and 1920×1080 remain pending.
- Existing uncommitted changes from prior UI work remain in the worktree and must be reviewed/separated before commit.
- Public API and responsive geometry behavior are source/test verified; visual acceptance still requires runtime evidence.

## Audit evidence update — 2026-10-13

`docs/plans/_audit-2026-10-13/03-data-grid-table-editor.md` verdict: **STALE-DOC** —
the primitives are **fully landed on `main`** (no feature branch remains):
`Container` + `ResponsiveGrid` implemented and exported
(`components/responsive_layout/ui.rs:69-195`, `components/mod.rs:88`), consumed at
6+ gallery call sites (`component_gallery_view.rs:257`,
`component_gallery_inputs.rs:112`, …); 7 responsive-layout unit tests pass at HEAD.
There are no separate Row/Col types — `ResponsiveGrid` covers the approved
"equivalent responsive Grid API". The CHECKLIST task list, PLAN branch name and the
"implementation has started" state note above are stale on the landing facts.

Fresh captures exercising the primitives: `docs/plans/_audit-2026-10-13/captures/gallery_*.png`
(3 sizes, Container/ResponsiveGrid consumers visible; heights capped at 838).
Remaining per audit: independent review sign-off and sidebar open/closed captures
at 3 widths around column transitions. State not changed by this note.

