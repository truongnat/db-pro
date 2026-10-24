---
schema_version: 1
id: "MEM-PLAN-issue-479-snippet-secret-clear-b5578bd9c4a7"
title: "Plan archive: Issue 479 Snippet Secret Clear"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-479-snippet-secret-clear]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "b5578bd9c4a7f380ebfe99b6decb936f5e2a52ce4260f9741557acec69f1a62c"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-479-snippet-secret-clear/PLAN.md"
  - "docs/plans/active/issue-479-snippet-secret-clear/FINDINGS.md"
  - "docs/plans/active/issue-479-snippet-secret-clear/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-479-snippet-secret-clear/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# #479 — Clear cloud_snippet after URI import (plaintext password leak)

## Goal
After a successful `apply_connection_snippet`, no plaintext password may
remain in `draft.cloud_snippet` — the unmasked `Input` renders it on screen
and the draft may be serialized while the dialog is open.

## Scope
- `crates/app/src/connection/mapper.rs`: `apply_connection_snippet` clears
  `draft.cloud_snippet` after the URI is consumed into fields. Clearing in
  the mapper (not the button handler) makes the guarantee hold for every
  caller; on parse error the field is left untouched so the user can fix it.
- Tests in `crates/app/src/connection/tests/mapper_tests.rs` (new file;
  `#[path]` module declared in `mapper.rs`).

## Non-goals
- No redaction scheme — clearing is the safe minimal path: `cloud_snippet`
  has no readers after import (grep confirms only the paste `Input` and the
  Import button touch it; `cloud_guidance` is the only retained output).
- No password masking on the paste input itself (out of scope; the paste
  field legitimately shows what the user typed until consumed).

## Acceptance
- `apply_connection_snippet` Ok → `draft.cloud_snippet == ""` while
  host/port/database/username/password/driver/ssl_mode are populated.
- Parse error → `cloud_snippet` unchanged.


## Source: `FINDINGS.md`

# #479 Findings

- `apply_connection_snippet` (`crates/app/src/connection/mapper.rs`) copied the
  URI's password into `draft.password` but left the full URI — including the
  plaintext password — in `draft.cloud_snippet`.
- `advanced_panels.rs:108-128` binds `cloud_snippet` to an unmasked `Input`
  (`.password(false)` semantics — plain text field), so the secret stays
  visible in the Cloud Presets panel after import.
- Grep audit of `cloud_snippet` across the workspace:
  - written to `String::new()` at draft construction sites
    (`mapper.rs:98`, `workspace_actions.rs:378`, runtime `Default`)
  - read only by the paste `Input` and cloned for the Import button call in
    `advanced_panels.rs`
  - never serialized to disk by `draft_to_domain_config` (no such field on
    `ConnectionConfig`), but the live draft keeps the secret for the dialog's
    lifetime — clearing is still required by the issue.
- Conclusion: nothing depends on the snippet after a successful import;
  `draft.cloud_snippet.clear()` inside `apply_connection_snippet` is the
  minimal safe fix. Done in the mapper so all callers get the guarantee; a
  failed parse keeps the user's input for correction.


## Source: `VERIFICATION.md`

# #479 Verification

Commands run:

- `cargo test -p db-pro-app --lib connection::` — 31 passed, 0 failed.
  New tests (`mapper_tests.rs`):
  - `apply_snippet_clears_cloud_snippet_after_success` — imports
    `postgresql://alice:s3cret@db.example.com:5432/appdb?sslmode=require`;
    asserts all fields populated (incl. `password == "s3cret"`,
    `ssl_mode == UiSslMode::Require`) and `cloud_snippet == ""`.
  - `apply_snippet_keeps_cloud_snippet_on_parse_error` — invalid URI keeps
    the original text so the user can correct it.
- `cargo test -p db-pro-app connection` — 68 passed, 0 failed.
- `cargo check -p db-pro-app` — clean.

