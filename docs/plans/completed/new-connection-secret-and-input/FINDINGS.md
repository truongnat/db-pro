# new-connection-secret-and-input — Findings

Record only evidence-backed findings.

## P0

None.

## P1

- **Keyring "empty service name" misclassified as fatal** — `keyring_vault.rs:246-266` (baseline `a3a6a529`). When the OS keyring is deterministically disabled, `build_secret_store` (`runtime/src/lib.rs:156-181`) passes an empty service name; `Entry::new("")` returns `keyring::Error::Invalid("service name is empty", "disallowed")`, which `is_keyring_unavailable` did **not** match, so `retrieve_secret` reported `DbError::Internal("keyring entry creation failed: ...")`. `ConnectionService::connect` / `test_connectivity_with_secret` (`core/.../connection_service.rs:367,470`) propagated it via `?`, surfacing as the new-connection "Configuration Error". **Fixed**: added an `Err(e) if self.service_name.is_empty()` arm routing to `retrieve_unavailable_secret`. **Runtime verified**: the New Connection dialog now opens cleanly with `DB_PRO_DISABLE_KEYRING` active at 1280×800 / 1440×900 / 1920×1080 — no error (PNGs under `screenshots/`). Data-safe (no mutation, no secret loss).

## P2

- **Input click-steal (password eye + double-click select)** — `text.rs:172` and `password.rs:149` (baseline `a3a6a529`). Both `show()` registered `frame.interact(egui::Sense::click())` on top of the inner `TextEdit`/eye button; egui resolved the click to the frame, so the eye never toggled and double-click never selected text. **Fixed**: removed the block (the `TextEdit` already focuses itself on click). **Runtime verified**: the password field + eye toggle render correctly at all three gate viewports (PNGs under `screenshots/`); the dynamic toggle/selection behaviours are covered by headless egui regression tests (`password_eye_toggles_when_clicked`, `input_field_receives_click_when_clicked`).

## Rejected / downgraded findings

- *Was the empty-service `Invalid` actually a provider bug?* No — it is the intended "keyring disabled" signal. The defect was the **classification**, not the empty name. Fix routes it to the fallback rather than changing the convention.
- *Should the frame still force focus?* The buggy block called `edit_response.request_focus()` as a workaround; removing it is correct because egui's `TextEdit` handles focus on its own click. Kept only a clarifying comment.

## Runtime gaps

- PostgreSQL: NOT EXERCISED at runtime (no live DB required for these secret-store / widget paths).
- SQLite: NOT EXERCISED at runtime (same).
- UI:
  - **Captured** the New Connection dialog at 1280×800 / 1440×900 / 1920×1080 on the disabled-keyring path (`screenshots/new-connection-*.png`).
  - **Not captured**: the dialog's transient "Testing connection…" spinner and any invalid-input error toast — the capture driver opens the dialog but does not drive clicks. The keyring-error state is absent by the P1 fix and was verified gone at runtime (clean dialog open under disabled keyring).