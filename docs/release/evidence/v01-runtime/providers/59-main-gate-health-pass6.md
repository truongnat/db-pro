# `main` gate health — restored in pass 6, with the before/after numbers

- Session: issue-queue pass 6, 2026-09-15 (evening)
- **Base head before the fixes:** `main @ 455f2718` (worktree clean, remote in sync)
- **Head after the fixes:** `d4df6373` (three fix commits) — every gate below was re-run on it
- **Outcome:** `main` is gate-clean again. Three genuine failures were found (not one): the reported
  formatting violation, a `--all-targets` compile break in a bench target, and a CI-mirroring test
  failure plus three dead-code lints in the merged MySQL test file. Each was fixed minimally and
  separately; no merged feature was refactored and no one else's work was reverted.

## 1. Before/after

| Gate | `main @ 455f2718` (before) | `main @ d4df6373` (after) |
|---|---|---|
| `cargo fmt --all -- --check` | **FAIL**, exit 1 — one diff, `crates/infrastructure/src/meta/saved_query_repo.rs:115` | **PASS**, exit 0 |
| `cargo check --workspace` | PASS, exit 0 | PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | **FAIL**, exit 101 — `E0432` unresolved import `db_pro_ui::GridSelectionLookup` in `crates/ui/benches/result_grid_benchmarks.rs:2` | **PASS**, exit 0 |
| `cargo test --workspace` | PASS, exit 0 — **931 passed / 0 failed / 35 ignored** | PASS, exit 0 — **932 passed / 0 failed / 35 ignored** (+1 = the new re-export test) |
| `cargo build --release --locked -p db-pro-native` | PASS, exit 0 | PASS, exit 0 |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | **FAIL**, exit 1 — 3 passed / 1 failed (the clippy check), binary 23.7 MB `a80cb4cf60be3f28…` | **PASS (partial)**, exit 0 — 4 passed / 0 warnings / 0 failed / 4 not executed, binary 23.7 MB `9da5cfa6948a61a6…` |
| `cargo test --all -- --include-ignored` (CI mirror, PostgreSQL fixture up) | **FAIL**, exit 101 — `mysql_integration` 0 passed / **5 failed** | **PASS**, exit 0 — **967 passed / 0 failed / 0 ignored** |

The CI-mirroring run used the disposable PostgreSQL 16.15 fixture on `127.0.0.1:55432`
(`dbpro-v01-pg-fixture`, credentials per the repo runbook, redacted here). It was stopped after the
run. No `qltx-*` container was touched.

**A note on the before-run's shape, for honesty:** in the before state, `cargo test --all` aborted the
`mysql_integration` target, so the targets after it alphabetically (including `pg_integration`, whose
28 tests are `#[ignore]`d and only run under `--include-ignored`) never executed. The before number is
therefore "5 failed and the rest of the suite not reached", not "5 failed out of everything". The
after run executes the full set: 967 passed, 0 failed, 0 ignored.

## 2. What was actually broken, and the fix for each

### 2.1 Formatting — `saved_query_repo.rs:115` (`29027814`)

`cargo fmt --all -- --check` failed on the `delete_folder` `raw_query(...)` call, which was wrapped on
one line. Fixed with `cargo fmt --all` (formatting only), commit `style(meta): apply rustfmt to the
saved-query store`. This is the violation the pass was told to expect; the diff is exactly that hunk
and nothing else.

### 2.2 A merged cleanup broke a bench target — `E0432` (`6afc5ba6`)

`cargo clippy --workspace --all-targets` could not compile `crates/ui/benches/result_grid_benchmarks.rs`:
the bench imports `db_pro_ui::GridSelectionLookup`, and the re-export that made that import valid was
gone. The history matters, because two commits are involved and neither message is quite right:

- `71ca86c1` (#246) added `pub use app::result_grid_view::GridSelectionLookup;` to `lib.rs`. That line
  **could not compile** — `result_grid_view` was a private module of `app`, so the path produced
  `E0603 module result_grid_view is private`. The commit was verified with `cargo test -p db-pro-ui --lib`
  and `cargo build --release -p db-pro-native`, neither of which compiles the bench target.
- `bac29442` ("resolve merge conflicts and compilation errors") removed the line to clear that error,
  describing it as an "unnecessary public re-export". That fixed the `E0603` and created the `E0432`:
  a bench is a **separate crate**, so it can only reach the type through the crate root, and plain
  `cargo check`/`cargo test` never build benches — only `--all-targets` does.

Fix: make the module `pub(crate) mod result_grid_view;` so the crate root can re-export the item, and
restore the re-export with a comment naming the bench. `selection_lookup_is_reachable_from_the_crate_root`
(`crates/ui/src/result_grid_view.rs` tests) resolves the type through `crate::GridSelectionLookup`, so
dropping the re-export now fails `cargo test` rather than only a bench build.

### 2.3 The merged MySQL test file failed CI and `-D warnings` (`f2a36b4b`, `d4df6373`)

Two distinct defects in the file added by #235:

1. **CI-mirroring failure.** `cargo test --all -- --include-ignored` runs the `#[ignore]`d MySQL tests
   in **every** environment, and CI's `DATABASE_URL` is `postgres://…` (`.github/workflows/ci.yml:34`),
   so `setup()`'s `.expect("DATABASE_URL must be a mysql:// URL for MySQL integration tests")` panicked
   at `crates/infrastructure/tests/mysql_integration.rs:44` for all five tests. Fixed by making
   `setup()` return `Option` (`mysql_config()?`) and having each test skip with a stated reason —
   the idiom `ssh_backup_runtime_verification.rs:83` already uses. The skip is honest: it makes the
   live acceptance visibly *not run* instead of pretending it passed, which is why #235's acceptance
   item 1 is still recorded as unverified.
2. **Three `-D warnings` errors** that only became visible once the bench target compiled:
   unused imports `ConnectionRegistry` and `DbError`, and an unused `password` binding in
   `mysql_config` (the config carries no password; `connect` takes it separately). Removed.

## 3. What was *not* changed, and why

- **No merged feature was refactored.** The chart engine, provider SDK, MySQL adapter, selection cache
  and projection cache are untouched except for the visibility word that makes the public path
  compilable.
- **No gate was weakened.** No `#[allow]` was added, no test was deleted, no lint level was lowered.
  The only test-level change is the MySQL skip path, which is a fixture gate the file already
  documented as its intent.
- **No substantive failure was papered over.** The failures here were a formatting diff, a visibility
  regression, a fixture-gated test that panicked outside its fixture, and dead code — all mechanical.
  Nothing in this pass required guessing at a public interface or persisted data, so nothing had to be
  escalated instead of fixed.
