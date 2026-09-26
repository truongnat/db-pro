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
