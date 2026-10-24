---
schema_version: 1
id: "MEM-SRC-6ef8433d37ad6900"
title: "Source mirror: docs/plans/completed/clippy-workspace-gates/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/clippy-workspace-gates/PLAN.md"
source_sha256: "6ef8433d37ad690000cd8e165de7aa64e1ddc5f12478a7c5146f1c3740b7dd12"
mirror_body_sha256: "6ef8433d37ad690000cd8e165de7aa64e1ddc5f12478a7c5146f1c3740b7dd12"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Plan — Restore Workspace Clippy Gate

## Objective

Fix pre-existing warnings that made the required `cargo clippy --all -- -D warnings` CI job fail on unrelated PRs.

## Scope

- Collapse match arms in saved-task SQL credential detection without changing classification behavior.
- Remove needless string-reference borrows in foreign-server/publication dialogs.
- Box the oversized monitoring-session action in its internal surface enum.
- Keep this maintenance change isolated from product-ticket PRs.

## Review Baseline

PR base SHA: `beaa1882a6253d902748c27fba53b99d9969b635`.

## Follow-up — Full workspace gate closure (2026-10, HEAD `f3fe9708`)

PR #532 (merged) fixed six diagnostics, but the mandatory gate
`cargo clippy --workspace --all-targets -- -D warnings` still failed with 73
inherited diagnostics at `f3fe9708`: 65 in `db-pro-core`, 2 each in
`db-pro-app`, `db-pro-infrastructure`, `db-pro-runtime`, `db-pro-ui`.

Scope of this follow-up (extends, not replaces, the plan above):

- Bump `async-trait` 0.1.91 → 0.1.92, which removes the bare `#[must_use]`
  the macro emitted on expanded trait methods. This eliminates all 64
  `clippy::double_must_use` false positives (`db-pro-core` ports + runtime +
  worker `agent.rs`/`agent_executor.rs`) with zero source churn.
- Box the oversized `DbError::ConstraintViolation` payload behind a new
  `ConstraintViolationDetails` struct, shrinking `DbError` so the three
  remaining `clippy::result_large_err` sites
  (`table_mutation_execution.rs`, mysql/postgres `execute_transaction_joined`)
  fall under the size threshold instead of needing new `allow`s.
- Remove four `needless_pass_by_value`-style redundant `&` references in
  `format!`/`assert_eq!` arguments (app monitoring views, disclosure UI, pg
  integration test).
- Replace `chunks_exact(4)`/`chunks_exact_mut(4)` with `as_chunks`/
  `as_chunks_mut` in the text-selection style theme code and its test.
- Rewrite one `if let … else return None` block with the `?` operator in the
  provider contract conformance test fixture parser.

No behavior changes are intended; the `ConstraintViolation` Display output is
byte-identical (`constraint violation (<TYPE>) on <table>: <message>`).
