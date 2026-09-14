# Editing or duplicating a connection no longer downgrades its stored TLS mode (#236)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 457579e` (clean at the start of the change); fix commit recorded in `LEDGER.md`
  (`fix(ui): preserve the stored TLS mode when editing or duplicating a connection (#236)`), pushed to
  `origin/main`
- Issue: **#236** ([P1][Security] Editing or duplicating a connection silently downgrades its stored
  TLS mode), filed during this session from the #144 triage
- Related: **#144** ([P1][RC1][Security] Make PostgreSQL remote connections secure by default) — this
  fix resolves one of the secondary blockers #144 lists and is **partial progress on the same theme**,
  not a fix for #144 itself

## 1. The defect, reproduced before the fix

Two tests were written first and run against the unmodified prefill code
(`crates/ui/src/connection_view.rs:147`, `:176` hardcoding `UiSslMode::Disable`). Both failed:

```
$ cargo test -p db-pro-ui --lib -- preserves_its_stored_ssl_mode
running 2 tests
test app::tests::duplicating_a_connection_preserves_its_stored_ssl_mode ... FAILED
test app::tests::editing_a_connection_preserves_its_stored_ssl_mode ... FAILED

---- app::tests::editing_a_connection_preserves_its_stored_ssl_mode stdout ----
panicked at crates/ui/src/app_tests.rs:288:5:
assertion `left == right` failed
  left: Disable
 right: Require

---- app::tests::duplicating_a_connection_preserves_its_stored_ssl_mode stdout ----
panicked at crates/ui/src/app_tests.rs:310:5:
assertion `left == right` failed
  left: Disable
 right: VerifyFull

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 359 filtered out; finished in 0.00s
EXIT_STATUS=101
```

The first assertion failure is on the dialog prefill; the second would also catch the value carried on
the save command, because the tests assert the mode on the dispatched `UiCommand` after calling
`dispatch_connection_command(true)`.

## 2. Root cause — the mode never reached the UI (by `file:line`, pre-fix)

| Step | Location (pre-fix) | Behaviour |
|---|---|---|
| stored | `crates/core/src/domain/connection.rs:88` | `ConnectionConfig.ssl_mode` exists and is persisted |
| dropped | `crates/runtime/src/api.rs:127-137`, `:757-767` | `ConnectionSummary` has no `ssl_mode`; `summary_from_connection` omits it |
| dropped | `crates/ui/src/runtime.rs:304-313`, `crates/native-app/src/translate.rs:1212-1221` | `UiConnectionSummary` has no `ssl_mode`; the mapping omits it |
| hardcoded | `crates/ui/src/connection_view.rs:147` (edit), `:176` (duplicate) | `ssl_mode: UiSslMode::Disable` in the prefill |
| written | `crates/ui/src/connection_view.rs:899-906` → `crates/native-app/src/translate.rs:11-16` | the whole draft, including `ssl_mode`, is written into `ConnectionConfig` and persisted by `ConnectionService::update` |

Net effect: `stored Require/VerifyCa/VerifyFull` → `Disable` on the first edit-and-save or
duplicate-and-save, with no user action and no warning. The same draft feeds **Test Connection**
(`crates/ui/src/connection_view.rs:897,909` sends `self.connection_draft`), so an edited connection was
also *tested* over the downgraded mode.

## 3. The fix (minimal, no new options)

The stored mode is carried through the summary DTOs into the prefill; the value written on save is now
the value that was stored.

| Change | Location (post-fix) |
|---|---|
| `ssl_mode: SslMode` added to the runtime DTO and populated from the stored config | `crates/runtime/src/api.rs:128-140`, `:766` |
| `ssl_mode: UiSslMode` added to the UI DTO | `crates/ui/src/runtime.rs:304-316` |
| domain→UI mapping `ui_ssl_mode` (reverse of the existing `draft_to_domain` arm) | `crates/native-app/src/translate.rs:55-63`, used at `:1231` |
| prefill uses the stored mode instead of a constant | `crates/ui/src/connection_view.rs:147`, `:176` |

Deliberately **not** changed: the new-connection default (`crates/ui/src/runtime.rs:81`), which stays
`Disable`; no `Prefer` variant; no CA/client-certificate fields. Choosing a safer default for *new*
connections is the owner-blocked decision tracked by #144, and the issue text for #236 scopes this fix
to the round-trip only.

## 4. Tests

| Test | File | Asserts |
|---|---|---|
| `app::tests::editing_a_connection_preserves_its_stored_ssl_mode` | `crates/ui/src/app_tests.rs:280-300` | prefill equals the stored `Require` **and** the dispatched `UpdateConnection` carries `Require` |
| `app::tests::duplicating_a_connection_preserves_its_stored_ssl_mode` | `crates/ui/src/app_tests.rs:302-319` | prefill equals the stored `VerifyFull`, the name is `... (Copy)`, **and** the dispatched `CreateConnection` carries `VerifyFull` |
| `connection_summary_with_ssl_mode` (fixture) | `crates/ui/src/app_tests.rs:266-278` | single fixture for both |

```
$ cargo test -p db-pro-ui --lib -- preserves_its_stored_ssl_mode
running 2 tests
test app::tests::duplicating_a_connection_preserves_its_stored_ssl_mode ... ok
test app::tests::editing_a_connection_preserves_its_stored_ssl_mode ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 359 filtered out; finished in 0.00s
EXIT_STATUS=0
```

The 21 pre-existing `UiConnectionSummary` literals in `crates/ui/src/app_tests.rs` gained an explicit
`ssl_mode: UiSslMode::Disable` so the fixtures keep describing what they described before (stored
`Disable`); no existing assertion changed meaning.

## 5. Full gate set (all six green, exit status 0)

| Gate | Command | Result |
|---|---|---|
| 1 | `cargo fmt --all -- --check` | `EXIT=0` |
| 2 | `cargo check --workspace` | `EXIT=0` |
| 3 | `cargo clippy --workspace --all-targets -- -D warnings` | `EXIT=0` |
| 4 | `cargo test --workspace` | **829 passed / 0 failed / 19 ignored** (19 suites) |
| 5 | `cargo build --release --locked -p db-pro-native` | `EXIT=0` |
| 6 | `bash .skills/perf-audit/scripts/perf-scan.sh` | `Passed 4 / Warnings 0 / Failed 0` — `Status: PASS`, `EXIT=0` |

Delta versus the recorded baseline **827 / 0 / 19**: **+2 passed** (the two new regression tests),
failed and ignored unchanged. The 19 ignored are the standing 18 `pg_integration` (need
`DATABASE_URL`) plus 1 SSH test (needs nine `DB_PRO_SSH_*` variables) — none of them touched here.

## 6. What this does not claim

- **Not** a fix for #144: the default TLS mode for *new* connections is still `Disable`, and
  `Require`/`VerifyCa`/`VerifyFull` remain downgrade-free but unqualified exactly as
  `docs/release/provider-capability-matrix.md:42` states.
- **Not** runtime-verified: no live TLS handshake against a remote PostgreSQL was performed; the proof
  here is source-level plus unit tests on the dialog/prefill/save round-trip.
- The stored mode is now preserved, but the dialog still offers no warning that `Disable` sends
  credentials in plaintext — that UI warning is part of #144's proposal, not this fix.
