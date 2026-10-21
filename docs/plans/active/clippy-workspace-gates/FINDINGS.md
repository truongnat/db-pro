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
