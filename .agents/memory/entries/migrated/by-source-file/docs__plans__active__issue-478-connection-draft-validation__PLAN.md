---
schema_version: 1
id: "MEM-SRC-6e15da94b3da6d0a"
title: "Source mirror: docs/plans/active/issue-478-connection-draft-validation/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-478-connection-draft-validation/PLAN.md"
source_sha256: "6e15da94b3da6d0a668259837c19bbbb8848501de5cec28d41122d41c806ebdc"
mirror_body_sha256: "6e15da94b3da6d0a668259837c19bbbb8848501de5cec28d41122d41c806ebdc"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
