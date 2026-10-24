---
schema_version: 1
id: "MEM-PLAN-issue-494-sqlite-actor-panic-b36750bdab51"
title: "Plan archive: Issue 494 Sqlite Actor Panic"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-494-sqlite-actor-panic]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "b36750bdab5127990461d23de7b111f16b60f80d68bd905a1d8b675e925236c4"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-494-sqlite-actor-panic/PLAN.md"
  - "docs/plans/active/issue-494-sqlite-actor-panic/FINDINGS.md"
  - "docs/plans/active/issue-494-sqlite-actor-panic/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-494-sqlite-actor-panic/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #494 — SqliteActor panic kills the command loop

## Problem
`SqliteActor::run` called `handle_*` methods directly. A panic in any handler
unwound out of the loop, killed the actor thread, and every queued responder
silently returned a generic oneshot `SendError`/`RecvError` — the connection
looked alive but was dead.

## Fix
In `crates/infrastructure/src/sqlite/actor.rs`:

- Every handler dispatch in `run()` is wrapped in
  `std::panic::catch_unwind(std::panic::AssertUnwindSafe(...))`.
- New free helpers:
  - `dispatch_guarded(&actor, |actor| ...)` for handlers returning
    `Result<T, DbError>` — panic becomes
    `Err(DbError::Internal("sqlite actor handler panicked: <msg>"))`.
  - `dispatch_transaction_guarded(&actor, stmt_count, |actor| ...)` for the two
    transaction handlers returning `Result<_, TransactionFailure>` — panic
    becomes a `TransactionFailure` with `phase: Validation`,
    `statement_index: statements.len()`, `outcome: Unknown` (post-panic
    transaction state is indeterminate), and the panic message in `error`.
  - `panic_message` extracts `&str`/`String` panic payloads.
- `Interrupt` and `Shutdown` arms unchanged (no handler call).
- `#[cfg(test)]`-gated `SqliteCommand::Poison` added so a test can inject a
  panic through the real dispatch loop.

## Unwrap/expect audit (issue asks: fix if trivial)
- `handle_execute`, `handle_introspect` (`introspect.rs`),
  `handle_explain`, `handle_raw_query`, `handle_execute_statement*`:
  no `unwrap`/`expect` on data paths — only `unwrap_or*`/`unwrap_or_default`
  safe fallbacks. Nothing to convert.
- `handle_execute_parameterized_transaction` uses `unwrap_or_default` inside an
  already-built error message — safe.


## Source: `FINDINGS.md`

# Findings — issue #494

- `SqliteActor::run` dispatched 9 handler variants directly; a panic in any of
  them killed the dedicated thread while `SqliteHandle` stayed clonable —
  callers then saw only `oneshot recv error`/`SendError` with no context.
- Handlers are `&self` methods, so `AssertUnwindSafe` is required
  (`rusqlite::Connection` is `!RefUnwindSafe`). Post-panic we only send an error
  through the responder and keep looping; the connection object is never
  destructively replaced.
- No data-path `unwrap`/`expect` found in `handle_execute`, `handle_introspect`
  (and its `introspect.rs` helpers), or `handle_explain`/`handle_raw_query` —
  all use `unwrap_or`-style fallbacks. The audit required no conversions.
- `TransactionFailure` contract: `statement_index = statements.len()` is the
  documented sentinel for transaction-level failures; `outcome: Unknown` is the
  honest state after a mid-transaction panic (may have left an open
  transaction — dropping the `tx` borrows doesn't run since `&self` methods
  can't outlive; any open transaction on `conn` remains open, next command can
  still proceed as `is_autocommit` checks handle it in `handle_execute_transaction`).


## Source: `VERIFICATION.md`

# Verification — issue #494

- `cargo test -p db-pro-infrastructure --lib sqlite` — **36 passed, 0 failed**
- `cargo check -p db-pro-infrastructure` — **clean** (earlier failures were
  transient sibling edits to `postgres/monitoring.rs` and `core/safety_lexer.rs`,
  unrelated to this change)
  - `dispatch_guarded_converts_panic_into_internal_error` — unit-test of the
    wrapper: panic → `DbError::Internal` containing "panic" and the payload.
  - `actor_survives_panicking_command_and_keeps_serving` — end-to-end: sends
    `SqliteCommand::Poison` (test-only variant), asserts the responder gets an
    `Internal` error mentioning the panic, then issues a real `SELECT 1`
    through the handle to prove the loop is still serving.

