---
schema_version: 1
id: "MEM-PLAN-issue-478-connection-draft-valid-c491f9358f41"
title: "Plan archive: Issue 478 Connection Draft Validation"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-478-connection-draft-validation]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "c491f9358f410770a8b2f487548413f98cdd80d19f3e99a7922b4bc005539c66"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-478-connection-draft-validation/PLAN.md"
  - "docs/plans/active/issue-478-connection-draft-validation/FINDINGS.md"
  - "docs/plans/active/issue-478-connection-draft-validation/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-478-connection-draft-validation/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# #478 Findings

- `validate_connection_draft` (`crates/app/src/connection/logic.rs`) checked
  name/database, port, and SSH fields but never host/username. A Postgres
  draft with blank host therefore passed UI validation and only failed later
  in `ConnectionConfig::validate` (`crates/core/src/domain/connection.rs`),
  whose `ValidationError.field` attribution never reaches the form inputs.
- Domain requires host + username for `DriverType::Postgres | Mysql | SqlServer`
  (`validate_network_fields`); SQLite is exempt — mirrored in the fix.
- Port/database inputs already used `.required(true)`; host/username did not,
  so the form visually suggested they were optional.

## Root cause
Draft validation was written before host/username became mandatory upstream;
the network-driver branch only validated the port.


## Source: `VERIFICATION.md`

# #478 Verification

Commands run:

- `cargo test -p db-pro-app --lib connection::` — 31 passed, 0 failed.
  New tests:
  - `network_draft_requires_host` — blank host on Postgres draft → `"Host is required"`.
  - `network_draft_requires_username` — blank username → `"Username is required"`.
  - `network_draft_with_host_and_username_passes` — regression guard.
  - `sqlite_draft_is_exempt_from_host_and_username` — SQLite with blank host/username/port → `Ok(())`.
- `cargo test -p db-pro-app connection` — 68 passed, 0 failed.
- `cargo check -p db-pro-app` — clean.

Manual reasoning: `form_fields.rs` Host/Username inputs now render the
required indicator; validation errors name the missing field in the dialog
error text.

