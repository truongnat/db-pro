# new-connection-secret-and-input — Plan

State: RUNTIME_VERIFY
Branch: `fix/new-connection-secret-and-input`

## Goal

Fix three user-reported defects on the new-connection flow:

1. Creating a new connection fails with `Configuration Error` / `internal: keyring entry creation failed: Attribute service name is empty is invalid: disallowed`.
2. The password field's eye toggle does not reveal the password.
3. Double-clicking into a text input does not select its text.

## Evidence / baseline

- `crates/infrastructure/src/secret/keyring_vault.rs:123` — `keyring::Entry::new` returns `keyring::Error::Invalid("service name is empty", "disallowed")` when the service name is empty.
- `crates/infrastructure/src/secret/keyring_vault.rs:246-266` — `retrieve_secret` matched only `is_keyring_unavailable` (`NoStorageAccess | PlatformFailure`) and otherwise reported the empty-service `Invalid` as a fatal `DbError::Internal`.
- `crates/runtime/src/lib.rs:156-181` — `build_secret_store` sets an **empty** service name whenever the OS keyring is off (`DB_PRO_DISABLE_KEYRING` / local debug), i.e. a deterministically *disabled* keyring, not an unavailable one.
- `crates/core/src/application/connection_service.rs:367,470` — `connect` / `test_connectivity_with_secret` propagate `retrieve_secret(...)?.ok_or_else(AuthFailed(...))`, so the fatal `Internal` surfaced as the new-connection "Configuration Error".
- `crates/ui/src/components/input/text.rs:172` and `crates/ui/src/components/input/password.rs:149` — both `show()` registered `frame_output.response.interact(egui::Sense::click())` *after* the inner `TextEdit`/eye button, so egui resolved the click to the frame and the inner widgets never saw it (eye never toggled, double-click never selected).
- Runtime logs already showed `saved Agent API key could not be loaded error=internal: keyring entry creation failed: Attribute service name is empty is invalid: disallowed`.

## Scope

- `keyring_vault.rs`: add an `Err(e) if self.service_name.is_empty()` arm routing to `retrieve_unavailable_secret` (same handling as an unavailable keyring).
- `text.rs` / `password.rs`: delete the frame `interact(Sense::click())` block; the `TextEdit` focuses itself on its own click.
- `tests.rs`: replace the weak focus guard with two revert-verifiable guards.

## Non-goals

- Changing `build_secret_store`'s empty-service-name convention (intentional, documented at `runtime/src/lib.rs`).
- Touching the provider connectors, the Tauri host, or any other input variant (`search.rs`, `textarea.rs`).
- Fixing the separate pre-existing `SearchInput` guessed-width reservation (recorded in `sidebar-content-full-width` FINDINGS).

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | yes | Provider-independent (secret store + egui widget); covered by headless unit tests, not a provider path |
| SQLite | yes | Provider-independent; same as above |

## Architecture

Secret store (infrastructure) → `ConnectionService` (core) → UI `Input`/`PasswordInput` (egui). Both defects are confined to their own layer and require no cross-layer contract change.

## Acceptance criteria

- [x] New-connection flow no longer fatal-errors when the OS keyring is disabled.
- [x] Password eye toggle reveals/hides the password.
- [x] Double-click into a text input selects its text.
- [x] Regression tests added and revert-verified for all three guards.
- [ ] UI runtime screenshots at 1280×800 / 1440×900 / 1920×1080 (SKIPPED — see VERIFICATION.md).
