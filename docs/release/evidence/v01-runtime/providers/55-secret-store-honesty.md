# Secret-store honesty: the vault no longer reports success it did not achieve (#243)

- Session: issue-queue pass 5, 2026-09-15
- Issue: **#243** ([P2][RC1][Trust] Secret store reports success when nothing was stored, orphans
  are invisible, and the agent key cannot be deleted), filed from the #126 keyring-lifecycle audit
  (`docs/release/audit-keyring-secret-lifecycle.md`, findings K-1/K-2/K-3/K-5)
- **Base head:** `main @ 19fe465` (worktree clean at session start)
- **Outcome:** the **vault half** of K-1 and K-2 is fixed, with deterministic tests and two probes.
  The issue is **left open**: the agent-key lifecycle (K-3), the K-5 test list and the
  warning-vs-hard-failure decision for a session-only store are named in §5 as the remainders, and
  one of them is a product decision this pass does not make on the owner's behalf.

## 1. Reproduction before the change

Read from the code, then confirmed by test (§3, both probes):

| Finding | Code before | Consequence |
|---|---|---|
| K-1, store | `if let Ok(entry) = self.keyring_entry(key) { let _ = entry.set_password(value); } Ok(())` — the keyring result was discarded and the function returned `Ok(())` unconditionally | In a release build (no file fallback) a denied keyring meant the UI reported the connection as saved, the password worked for the session through the in-memory map, and after a restart connecting failed with `authentication failed: password not found in secret store` |
| K-1/K-2, delete | `let _ = store.delete(key); let _ = self.delete_session_secret(key); let _ = entry.delete_credential();` | A delete that did not take effect was indistinguishable from one that did; the surviving entry is an orphan, and the `SecretStore` port has no enumeration (`crates/core/src/ports/secret_store.rs:9-15`), so nothing could ever see it |

## 2. What was implemented

`crates/infrastructure/src/secret/keyring_vault.rs`, two methods, no port change, no new
component:

- **`store_secret`** now tracks whether *any* layer actually accepted the value (`file fallback`,
  `session store`, OS keyring) and returns `Err(DbError::Internal("the secret was not stored: …"))`
  when none did. A keyring refusal is logged at `warn` with the consequence spelled out
  ("will not survive a restart unless another store holds it") instead of being dropped.
- **`delete_secret`** collects per-layer failures and returns
  `Err(DbError::Internal("the secret could not be deleted from every credential store (…) — an
  orphan entry may remain"))` when any layer that could hold the secret failed to delete it.
  `keyring::Error::NoEntry` is treated as success, because "the key is gone" is the goal state of a
  delete, not a failure.

Both changes are visible to a caller through existing paths: every `store_secret` call site in
`connection_service` already propagates with `?` or runs a rollback, and `delete` already returns the
error to the UI (`crates/core/src/application/connection_service.rs:294-303`). Nothing about the
layer order, the session fallback or the encrypted-file gating (#142) changed.

## 3. Deterministic tests, and the seam that makes them deterministic

`keyring::Entry::new("", "k")` fails with `Invalid("service", "cannot be empty")` on this host —
verified with a throwaway probe, deleted before the commit. **A vault constructed with an empty
service name therefore has no usable keyring layer, with no production change and no test-only
injection**, which is what makes the failure paths testable at all. (The host smoke recorded in
`providers/14-install-smoke.txt` had to fake a key with a placeholder `GROQ_API_KEY`; this seam
removes that kind of environment dependence from the vault tests.)

| Test | Pins |
|---|---|
| `store_secret_reports_when_no_store_accepts_the_value` | K-1: no keyring, no fallbacks → `Err`, and the message names the outcome |
| `delete_secret_reports_a_store_that_could_not_delete` | K-2: the fallback directory is made read-only after the store, so the delete cannot be persisted → `Err` naming the store and the orphan consequence |
| `delete_secret_of_a_missing_key_is_not_a_failure` | the `NoEntry` case stays a success (no false alarm on ordinary deletes) |
| `retrieve_secret_reports_a_keyring_entry_that_cannot_be_created` | one deterministic half of the pair that used to be a single accept-either assertion: an unreachable store is an error, not a silent `None` |
| `store_secret_still_succeeds_when_only_the_session_store_holds_the_secret` | **characterisation test** of the behaviour this change deliberately leaves alone (see §5), so a future change to it is deliberate |

Falsification, both probes reverted (the tree carries neither): reverting `store_secret`'s result to
`Ok(())` failed `store_secret_reports_when_no_store_accepts_the_value` only; reverting
`delete_secret`'s result to `Ok(())` failed `delete_secret_reports_a_store_that_could_not_delete`
only — **2 failed / 8 passed** with both reverted, which is exactly the two tests that claim the fix.

## 4. Gates (raw totals, this host)

| Gate | Command | Result | Baseline | Delta |
|---|---|---|---|---|
| Format | `cargo fmt --all -- --check` | exit **0** | — | — |
| Check | `cargo check --workspace` | exit **0** | — | — |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | exit **0**, no warnings | — | — |
| Tests | `cargo test --workspace` | **904 passed / 0 failed / 27 ignored** | 899 / 0 / 27 | **+5** |
| CI-mirror (fixture up) | `DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture cargo test --all -- --include-ignored` | **931 passed / 0 failed / 0 ignored** | 926 / 0 / 0 | **+5** |
| Release build | `cargo build --release --locked -p db-pro-native` | exit **0** | — | — |
| Perf scan | `bash .skills/perf-audit/scripts/perf-scan.sh` | **PASS (partial)**, exit **0** — 4 executed passed, 0 warnings, 0 failed, 4 not executed | same shape | — |

The +5 is exactly the five tests above. The PostgreSQL fixture was started for the CI-mirroring run
and **stopped afterwards**; credentials are redacted. No `qltx-*` container was touched. The tests
that need a keyring use a throwaway service name or none at all, so no real credential is written on
the test host.

## 5. What is **not** fixed here, and why the issue stays open

| Remainder | Why it is not in this commit |
|---|---|
| **K-1, the session-only case**: a store where only the in-memory session store accepted the value still returns success, so the user is not told "saved, but this will not survive a restart" | Delivering that sentence needs a **warning channel** the app does not have: `RuntimeEvent`/`UiEvent` carry `OperationCompleted { operation }` or `Failed { message }`, there is no warning variant, no warning toast variant (`ToastVariant` is `Default`/`Success`/`Danger`), and `OperationCompleted` is constructed at 12 sites. More importantly, the choice between *warning while saving* (what the issue's acceptance asks for) and *hard failure* ("the connection was not saved") is a product decision with user-visible consequences on systems without a keyring — the session fallback exists deliberately (`with_session_fallback`, `docs/architecture/security-boundaries.md:35`), so a hard failure would contradict the shipped design. **This pass does not make that call.** The characterisation test above pins today's behaviour so the change, when it comes, is deliberate |
| **K-3, the agent key lifecycle** | `agent/groq_api_key` is still written and read through a raw `keyring::Entry` in `crates/native-app/src/main.rs:205-234`, bypassing the vault, and there is still no "Forget key" action. Moving it onto the vault means a new `UiCommand`/`UiEvent` pair plus a panel action, i.e. a feature slice of its own, not a bounded patch. It is the **second** half of the issue title, and it stays open |
| **K-5, the rest of the test list** | Covered here: store failure, a delete that could not take effect, the missing-key delete, and one deterministic retrieval case. **Not covered**, and each needing the seam above or a new one: a retrieve `Err` from an available-but-denying keyring, an edit that must *keep* the secret, restart retrieval, a corrupt fallback file. `fallback.rs` and `encryption.rs` still have zero tests |
| Linux persistence semantics | Out of scope by the issue's own text (`Accept RC1` for v0.1, owner decision recorded in the #126 blocker comment). Not re-opened here |

The vault honesty fixes above are complete on their own terms; they do not depend on any of the
remainders, and they are what makes the two "reports success when nothing was stored" cases in the
issue title reportable today.
