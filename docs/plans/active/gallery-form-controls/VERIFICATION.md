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
