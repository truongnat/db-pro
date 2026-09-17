# new-connection-secret-and-input — Findings

Record only evidence-backed findings.

## P0

None.

## P1

- **Keyring "empty service name" misclassified as fatal** — `keyring_vault.rs:246-266` (baseline `a3a6a529`). When the OS keyring is deterministically disabled, `build_secret_store` (`runtime/src/lib.rs:156-181`) passes an empty service name; `Entry::new("")` returns `keyring::Error::Invalid("service name is empty", "disallowed")`, which `is_keyring_unavailable` did **not** match, so `retrieve_secret` reported `DbError::Internal("keyring entry creation failed: ...")`. `ConnectionService::connect` / `test_connectivity_with_secret` (`core/.../connection_service.rs:367,470`) propagated it via `?`, surfacing as the new-connection "Configuration Error". **Fixed**: added an `Err(e) if self.service_name.is_empty()` arm routing to `retrieve_unavailable_secret`. Data-safe (no mutation, no secret loss).

## P2

- **Input click-steal (password eye + double-click select)** — `text.rs:172` and `password.rs:149` (baseline `a3a6a529`). Both `show()` registered `frame.interact(Sense::click())` on top of the inner `TextEdit`/eye button; egui resolved the click to the frame, so the eye never toggled and double-click never selected text. **Fixed**: removed the block (the `TextEdit` already focuses itself on click).

## Rejected / downgraded findings

- *Was the empty-service `Invalid` actually a provider bug?* No — it is the intended "keyring disabled" signal. The defect was the **classification**, not the empty name. Fix routes it to the fallback rather than changing the convention.
- *Should the frame still force focus?* The buggy block called `edit_response.request_focus()` as a workaround; removing it is correct because egui's `TextEdit` handles focus on its own click. Kept only a clarifying comment.

## Runtime gaps

- PostgreSQL: NOT VERIFIED at runtime (no live DB exercised; headless unit tests cover the secret-store path deterministically).
- SQLite: NOT VERIFIED at runtime (same as above).
- UI: **SKIPPED** — no `crates/native-app/src/capture.rs` capture driver exists, and `screencapture` is TCC-blocked on this host, so screenshots at 1280×800 / 1440×900 / 1920×1080 cannot be auto-captured. Guard correctness is covered by headless egui regression tests instead.
