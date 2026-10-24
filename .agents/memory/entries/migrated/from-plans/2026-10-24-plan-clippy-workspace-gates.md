---
schema_version: 1
id: "MEM-PLAN-clippy-workspace-gates-eddc6f13916a"
title: "Plan archive: Clippy Workspace Gates"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, clippy-workspace-gates]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "eddc6f13916a1ebbee18cf12346e5cfb15bc2d5e72ef1e88dd268c698c074ad5"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/clippy-workspace-gates/PLAN.md"
  - "docs/plans/completed/clippy-workspace-gates/FINDINGS.md"
  - "docs/plans/completed/clippy-workspace-gates/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/clippy-workspace-gates/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Findings — Restore Workspace Clippy Gate

CI failed because the current Rust Clippy version reports three collapsible-match warnings in `saved_task.rs`, two needless borrows in dialog titles, and one large enum variant in the monitoring surface. Since CI promotes warnings to errors, every PR was blocked before its own code could pass the required Rust check.

The fixes are mechanical: express credential checks as guarded match arms, pass owned formatted titles directly, and box the large monitoring action payload at the internal enum boundary.

The first required CI run then exposed an additional test-compilation blocker: an unused closure argument under `RUSTFLAGS=-Dwarnings`. Renaming it `_ctx` preserves the test and lets the workspace test target compile.

## Follow-up findings at `f3fe9708` (73 inherited diagnostics)

- 64 of the 73 were `clippy::double_must_use` originating inside the
  `#[async_trait]` macro expansion: async-trait 0.1.91 emitted a bare
  `#[must_use]` on expanded trait methods whose return type
  (`Pin<Box<dyn Future>>` / `Result`) is already must-use. Upstream fixed this
  in async-trait 0.1.92 (dtolnay removed the redundant attribute), so a
  lockfile-only bump cleared all 64 with no source edits — a scoped `allow`
  would have masked a fixed-not-avoided upstream bug.
- 3 `clippy::result_large_err` remained: `TransactionFailure` measured ≥144
  bytes purely because `DbError`'s `ConstraintViolation` variant inlined five
  payloads (112 bytes). Boxing that one variant behind
  `ConstraintViolationDetails` shrinks `DbError` to the next-largest variant
  (40 bytes) and drops every flagged `Result<_, TransactionFailure>` under
  the threshold — smaller and more broadly beneficial than boxing the error
  at the three call sites or extending the existing `#[allow]` on
  `DbConnector`.
- 6 mechanical polish lints: 4 redundant `&` in `format!`/`assert_eq!`,
  2 constant-size `chunks_exact(_mut)` with `as_chunks(_mut)` equivalents,
  1 `question_mark` rewrite in a test fixture URL parser. The `?` rewrite is
  exactly equivalent: the replaced `else { return None; }` arm only fired when
  `strip_prefix("mysql://")` returned `None`, which `?` now propagates.
- No new `#[allow]` attributes were added; the pre-existing scoped
  `#[allow(clippy::result_large_err)]` on the `DbConnector` trait
  (`crates/core/src/ports/db_connector.rs:61`) was left in place — it covers
  trait-method signatures that still return `DbError`/`TransactionFailure`
  directly and remains justified by its inline comment.


## Source: `VERIFICATION.md`

# Verification — Restore Workspace Clippy Gate

- `cargo +nightly fmt --all -- --check`: passed (exit 0).
- `git diff --check`: passed.
- `cargo clippy --all -- -D warnings`: passed.
- `cargo test -p db-pro-app --lib monitoring_activity_view`: 5 passed / 0 failed / 0 ignored; exit 0.
- `cargo test -p db-pro-core --lib saved_task`: 8 passed / 0 failed / 0 ignored; exit 0.

- `cargo test -p db-pro-ui --lib reopened_modal_is_topmost_immediately`: 1 passed / 0 failed / 0 ignored; exit 0.

## Follow-up gate evidence at `f3fe9708` (session 2026-10)

- Baseline: `cargo clippy --workspace --all-targets -- -D warnings` failed with 73 diagnostics
  (65 `db-pro-core` — 64 `double_must_use` from async-trait expansion + 1 `result_large_err` —
  and 2 each in `db-pro-app`, `db-pro-infrastructure`, `db-pro-runtime`, `db-pro-ui`).
- `cargo clippy --workspace --all-targets -- -D warnings`: 0 diagnostics, exit 0 (final run).
- `cargo +nightly fmt --all -- --check`: exit 0.
- `cargo test --workspace`: 2,176 passed / 0 failed / 56 ignored (matches baseline), exit 0;
  includes the live `pg_integration`, `mysql_integration`, `mysql_fixture_matrix`, and
  `provider_contract_conformance` suites — all `ok`.
- No `#[allow]` was added in the follow-up.

