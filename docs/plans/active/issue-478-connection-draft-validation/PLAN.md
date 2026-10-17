# #478 — Validate host/username in connection draft before domain layer

## Goal
`validate_connection_draft` (UI layer) must reject empty host/username for
network drivers so errors surface at the field level instead of late, inside
`ConnectionConfig::validate`.

## Scope
- `crates/app/src/connection/logic.rs`: inside the existing network-driver
  branch (Postgres/MySQL/SqlServer), return `"Host is required"` when
  `draft.host.trim().is_empty()` and `"Username is required"` when
  `draft.username.trim().is_empty()`. SQLite exempt (unchanged).
- `crates/app/src/connection/form_fields.rs`: `.required(true)` on the Host
  and Username `Input`s (Port/Database already required).
- Tests in `crates/app/src/connection/tests/logic_tests.rs`.

## Non-goals
- No restructuring of `validate_connection_draft`'s `Result<(), String>` API
  or localization of its messages (existing messages are plain strings).
- No change to domain `ConnectionConfig::validate`.

## Acceptance
- Empty host on Postgres draft → `Err("Host is required")`.
- Empty username → `Err("Username is required")`.
- SQLite draft with empty host/username/port → `Ok(())`.
