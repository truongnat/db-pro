---
schema_version: 1
id: "MEM-PLAN-gallery-form-controls-9e32a8a288b7"
title: "Plan archive: Gallery Form Controls"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, gallery-form-controls]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "9e32a8a288b74dfdd35d9dce959dfee23965e3ab4d58a5d06fc04ff9102aa553"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/gallery-form-controls/PLAN.md"
  - "docs/plans/active/gallery-form-controls/FINDINGS.md"
  - "docs/plans/active/gallery-form-controls/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/gallery-form-controls/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Gallery Form Controls

- State: RUNTIME_VERIFY
- Branch: `fix/gallery-form-controls`
- Baseline SHA: `44d157e2e4c6ad9d83be9ec69414f508f91d38d8`
- Scope: native Component Gallery form validation, alert dismiss icon, checkbox and radio cursors.

## Problem and evidence

At the baseline SHA, `Input::show` and `PasswordInput::show` paint a red error frame and call `paint_field_chrome`, which paints a blue focus stroke even when `enabled` is false (`input/text.rs`, `input/password.rs`, `input/layout.rs`). An alert dismiss icon uses a `Secondary` button with a gray fill (`alert.rs`, `button.rs`). Checkbox and radio set a pointing cursor through `Response::on_hover_cursor`; runtime behavior still needs verification (`selection.rs`).

## Scope

1. Ensure validation error fields show one red border while focused.
2. Show the destructive alert dismiss icon without a gray button fill.
3. Ensure enabled checkbox and radio hit areas show a pointing hand, including their labels.

These shared primitives are fixed at their source, so other callers receive the same behavior. Database providers and archived frontend are unaffected.

## Acceptance

- Error, focus and normal input states have one correct border.
- Alert dismiss icon remains visible, accessible and clickable without a gray fill.
- Enabled checkbox/radio show a pointing hand; disabled controls do not.
- Rust quality gates and native UI captures are recorded in `VERIFICATION.md`.

The static capture harness cannot place focus on a specific invalid input or record the OS cursor, so those interactive cases remain for a live manual pass before completion.


## Source: `FINDINGS.md`

# Findings

Baseline: `44d157e2e4c6ad9d83be9ec69414f508f91d38d8`.

| Severity | Finding | Scope |
|---|---|---|
| P2 | Error frame and focus chrome both render a border on focused invalid text/password inputs. | Shared input primitives. |
| P2 | Secondary dismiss button gives destructive alerts a gray square behind the close icon. | Shared alert primitive. |
| P2 | Checkbox/radio cursor behavior reported incorrect in Component Gallery; source already requests PointingHand, so inspect runtime after making the request explicit. | Shared selection primitive. |

Provider impact: PostgreSQL and SQLite are not involved. External PR coverage: not checked; fix is isolated to the native UI branch.

Self-review: P0 0 / P1 0 / P2 1 remaining. The remaining P2 is the native interactive focus/cursor pass; screenshots and headless cursor tests do not prove the OS pointer and the focused invalid field together. The existing gallery layout can extend horizontally at 1280px, so the dismiss icon is positioned against the visible clip edge. This branch leaves that broader layout issue outside its focused scope.


## Source: `VERIFICATION.md`

# Verification

Baseline SHA: `44d157e2e4c6ad9d83be9ec69414f508f91d38d8`.

> **Filled 2026-10-13** — this file was recorded as a "Work in progress" stub while
> `CHECKLIST.md` marked gates and captures done (flagged P1 by
> `docs/plans/_audit-2026-10-13/05-design-system-components.md`). Below is the real
> state reconstructed from code at HEAD plus this audit's fresh captures.

## Scope status (verified in code at HEAD, audit slice 05)

| Scope item | Status | Evidence |
|---|---|---|
| Error/focus border precedence — focused invalid text/password inputs show one red border, not two | Implemented | `components/input/layout.rs:23-40` — `paint_field_chrome` resolves a single stroke via `resolve_chrome` (`tokens/component/input.rs:53`); precedence contract tested: `disabled_outranks_error_focus_and_hover`, `error_outranks_focus_and_hover_but_not_disabled`, `focus_outranks_hover_and_rest_follows` |
| Alert dismiss icon without gray square fill | Implemented | `components/alert/ui.rs:133-138` — dismiss is frameless (`egui::Button` `.frame(false)`); visually confirmed on `captures/connection-error_*.png` (2026-10-13) |
| Checkbox/radio pointing-hand cursor incl. labels | Implemented | `components/selection/ui.rs:139,267,378` — explicit `on_hover_cursor`/cursor requests; unit tests `enabled_selection_controls_use_pointing_hand` and `disabled_selection_controls_do_not_use_pointing_hand` (`selection/ui.rs:572,578`) |

## Automated gates

- `cargo test -p db-pro-ui` was run during the work session and passed (exact total
  not recorded at the time; the audit re-verified the fixes' own unit tests at HEAD).
- Chrome precedence tests (`tokens/component/input.rs`) and selection cursor tests
  (`selection/ui.rs:571-580`) **pass at HEAD** — audit slice 05.
- No `#[ignore]`d tests involved.

## Native captures

| Artifact | States shown |
|---|---|
| `screenshots/form-error-1280x800.png` | Gallery form validation error state |
| `screenshots/form-error-1440x900.png` | Same, mid viewport (height host-capped at 838) |
| `screenshots/form-error-1920x1080.png` | Same, wide viewport (height host-capped at 838) |
| `screenshots/form-normal-1280x800.png` | Normal resting fields |
| `screenshots/selection-1280x800.png` | Checkbox/radio selection row |

Fresh audit captures (2026-10-13, `docs/plans/_audit-2026-10-13/captures/`):
`gallery_{1280x800,1440x900,1920x1080}.png`, `connection-error_*.png` (required-field
red outlines + inline messages + dismissible toast, all legible), and
`new-connection{,-light}_*.png` exercising the same shared input primitives.

## Remaining (honest gaps)

- **Manual pass still open** (`CHECKLIST` last item, unchecked): the static capture
  harness cannot place keyboard focus on a specific invalid input or record the OS
  pointer — a live manual inspection of the focused-invalid border and the pointing
  hand remains required before completion.
- Original per-command gate logs were not committed at the time of the change; the
  fix presence is code-verified and unit-tested (above), but no retrievable
  fmt/clippy output exists for this plan's session.

State stays `RUNTIME_VERIFY`; no COMPLETED promotion without the manual interactive pass.

