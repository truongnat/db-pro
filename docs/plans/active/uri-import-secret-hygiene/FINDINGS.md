# URI Import Secret Hygiene — FINDINGS

## #479 — CONFIRMED, P2 (on-screen hygiene)

`crates/app/src/connection/mapper.rs::apply_connection_snippet` copies parsed username/password and connection settings into `UiConnectionDraft`, but did not clear `cloud_snippet`. `crates/app/src/connection/advanced_panels.rs` binds that field to an ordinary text `Input`, so the URI password remains visible after a successful import.

The issue's additional assertion that this draft is serialized/persisted or reaches the backend/logging path was not supported in the inspected code: `UiConnectionDraft` has no serde derives; the native-app translator does not read `cloud_snippet`; repo-wide use is limited to UI, draft initialization, and the field definition. The established risk is plaintext visible on screen, including screenshots/shoulder-surfing, not a confirmed disk/backend leak.

## Fix

Clear `cloud_snippet` after parsing and applying a valid snippet. Leave it unchanged when parsing fails so the user can correct the input. The dedicated password field continues to hold the imported value and uses its existing masked input.

## UI review

- Surface job: paste/import a connection URI, then review/edit dedicated connection fields.
- Keep: current native egui form, compact layout, theme tokens, and actionable parse errors.
- Change: successful import consumes and clears the plaintext URI; invalid input stays visible for recovery.
- No layout/theme changes are intended; runtime captures verify the dialog remains legible in both themes and required viewports.
