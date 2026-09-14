# Keyring is primary in every build; the DEV-only file fallback is gated out of release (#142)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 1ff33ae` (clean at the start of the change); fix commit recorded in the ledger
- Related register/limitation entries: `R011` (description corrected), new `R-KEYRING-FALLBACK`
  (`FIXED`); `R-KEYRING-STALL` deliberately **unchanged** (separate, already-classified finding)
- Issue: **#142** ([P1][RC1][Security] Configure real OS keyring stores and remove DEV-only
  fallback from production)

## 1. What the triage found, and what the code did

Before the fix the shipping wiring built the secret store like this
(`crates/runtime/src/lib.rs:73-75` pre-fix):

```rust
let secret_store = Arc::new(
    KeyringVault::new("com.dbpro.app", secrets_dir)
        .with_session_fallback()
        .with_fallback(),        // <- encrypted-file fallback, unconditionally
);
```

`with_fallback()` sets `allow_fallback = true`, which in the vault means:

| Effect | Location | Consequence |
|---|---|---|
| every `store_secret` also writes `secrets/secrets.json` | `crates/infrastructure/src/secret/keyring_vault.rs:172` → `:248` | passwords on disk in *every* build |
| every `retrieve_secret` reads the file **first** | `crates/infrastructure/src/secret/keyring_vault.rs:189` | the weakest store is also the default path |
| the file's AES-256-GCM key is derived from the service name | `crates/infrastructure/src/secret/keyring_vault.rs:110` (`derive_master_key` → `encryption::derive_key(&self.service_name, salt)`), salt = the service name bytes (`:114`) | `com.dbpro.app` is not a secret, so the file is decryptable by anyone who can read it |

That was a local-only exposure (read state directory → derive key → decrypt stored passwords), but
it made the OS keyring advisory rather than authoritative, and the log line at `:256` still said the
secret was stored "in fallback file (keyring unavailable)" even though the keyring had not been
tried.

## 2. Store-selection order — debug vs release (post-fix, by `file:line`)

`DbProRuntime::new` builds the store through `build_secret_store`
(`crates/runtime/src/lib.rs:99`), which always calls
`KeyringVault::new(KEYRING_SERVICE, secrets_dir).with_session_fallback()` (`:100`) and adds
`.with_fallback()` only when `file_secret_fallback_enabled()` allows it (`:101`, `:106`).

**Release build, `DB_PRO_ALLOW_FILE_SECRET_FALLBACK` unset** — `allow_fallback = false`,
`allow_session_fallback = true`. `retrieve_secret`
(`crates/infrastructure/src/secret/keyring_vault.rs:187-228`) runs in exactly this order:

| # | Line | Step | Active in release? |
|---|---|---|---|
| 1 | `:189-193` | encrypted-file fallback (`secrets.json`) | **no** — guard is `if self.allow_fallback`, which is `false`, so the file is never opened |
| 2 | `:194-198` | in-memory session map | yes |
| 3 | `:200-209` | OS keyring entry lookup (`keyring::Entry::new`) | yes — on `NoStorageAccess`/`PlatformFailure` → `:204` → `:152` `retrieve_unavailable_secret` → session |
| 4 | `:211-212` | `entry.get_password()` hit | yes — returns the keyring value |
| 5 | `:213-221` | `NoEntry` → `:216` `allow_fallback` is false → `:219` session lookup | yes |
| 6 | `:222-225` | keyring unavailable during read → `:224` session lookup | yes |
| 7 | `:226` | any other keyring error → `DbError::Internal` | yes |

**Debug build** (or release with the opt-in): `allow_fallback = true`, so step 1 does open and read
`secrets.json` and returns before the keyring is ever consulted, and `store_secret` writes the file
before touching the keyring (`:172-174`, `:180-182`). That ordering is **deliberate and unchanged**:
the comment at `:188` states the reason — the local encrypted vault avoids an OS keychain prompt on
every developer launch. It is confined to builds where the file fallback is allowed at all.

So the answer to the review question is: *in a release build the encrypted-file fallback is not
consulted before the OS keyring — it is not consulted at all*, because the only code path that
touches the file is behind `allow_fallback`, which is `false` in a release build that did not set
the opt-in. In a debug build the file **is** consulted first, by design and always with the warning
log emitted at `crates/runtime/src/lib.rs:102`.

## 3. The gating decision

| Input | Value | Result |
|---|---|---|
| build profile | `debug_assertions` on | enabled |
| build profile | `debug_assertions` off | disabled unless the opt-in is set |
| `DB_PRO_ALLOW_FILE_SECRET_FALLBACK` | `1`, `true`, `TRUE`, `" true "` | enabled |
| `DB_PRO_ALLOW_FILE_SECRET_FALLBACK` | unset, empty, `0`, `yes`, any other value | disabled |

Implementation: `file_secret_fallback_enabled_for(debug_build, opt_in)`
(`crates/runtime/src/lib.rs:83`) is a pure function of the two inputs; `file_secret_fallback_enabled()`
(`:75`) supplies them from `cfg!(debug_assertions)` and the environment. The opt-in is a *release*
escape hatch for CI/packaging hosts without an OS keyring — the debug branch ignores it entirely
(`:84-86`), so setting it to `0` in a dev shell cannot turn the fallback off.

When the fallback is enabled the store emits a `tracing::warn!` naming the env var and the
weak-key property (`crates/runtime/src/lib.rs:102-105`), so a release build running with the opt-in
says so in its own logs. The vault's `with_fallback` doc comment states the same
(`crates/infrastructure/src/secret/keyring_vault.rs:47-52`), and the misleading "keyring unavailable"
log line is gone (`:256` now reads "secret also stored in the encrypted-file fallback
(development/CI fallback)").

## 4. Release build with no keyring item — the degradation path

No keyring item, no panic, no file:

| Operation | Path | Outcome |
|---|---|---|
| `store_secret` | `:172` skipped (no file) → `:175` session insert → `:180` `entry.set_password` best-effort, error discarded | `Ok(())`; the password lives in the process memory for this session |
| `retrieve_secret`, same process | `:194` session hit | `Ok(Some(password))` — no keyring round trip |
| `retrieve_secret`, after restart | `:194` empty → `:200-211` `NoEntry` → `:213-221` → `:219` session miss | `Ok(None)` — the caller asks for the password again; no error, no panic |
| keyring unavailable (no Secret Service / locked keychain) | `:202-205` or `:222-225` → `retrieve_unavailable_secret` (`:152`) → `require_fallback()` passes because the session fallback is on (`:94`) → `:157` session lookup | `Ok(None)` / session value; never `DbError::EncryptionFailed` |
| `delete_secret` | `:231` file skipped → `:236` session remove → `:239` keyring delete | `Ok(())` |

The only configuration that *fails closed* is a vault with neither fallback — preserved behaviour
(`require_fallback` at `crates/infrastructure/src/secret/keyring_vault.rs:93-104`, pinned by
`new_vault_fails_closed_without_os_keyring`). The shipping wiring never uses it: the session
fallback is unconditional.

A stale `secrets.json` written by an older build is inert in a release build: it is not read
(step 1 above is skipped) and not rewritten (`store_fallback` is only reachable through the same
`allow_fallback` guard). This is pinned by
`secret::keyring_vault::tests::disabled_file_fallback_never_touches_a_stale_fallback_file`.

**No migration step is needed** for that file: the new code neither reads nor deletes it, so an
upgraded install keeps the file exactly as the previous version left it. Deleting it manually is
harmless; it is no longer consulted.

## 5. Tests

| Test | Location | Asserts |
|---|---|---|
| `file_fallback_needs_a_debug_build_or_an_explicit_opt_in` | `crates/runtime/src/lib.rs:322` | the decision function: debug ⇒ always enabled (even with `0`); release ⇒ enabled only for `1`/`true`/`TRUE`/`" true "`, disabled for unset/empty/`0`/`yes` |
| `shipping_secret_store_follows_the_build_profile` | `crates/runtime/src/lib.rs:339` | `build_secret_store` always enables the session fallback, and its file-fallback flag equals the decision function's value for the profile it was compiled in |
| `release_secret_store_never_selects_the_file_fallback_by_default` | `crates/runtime/src/lib.rs:361` (`#[cfg(not(debug_assertions))]`) | the release half of the same wiring: session fallback on, file fallback off. Compiled only when `debug_assertions` is off, so `cargo test -p db-pro-runtime --release` is what runs it |
| `fallback_flags_follow_the_builder_calls` | `crates/infrastructure/src/secret/keyring_vault.rs:320` | the builder flags the wiring reads: bare vault = both off, `with_session_fallback` = session only, `with_fallback` = file only |
| `disabled_file_fallback_never_touches_a_stale_fallback_file` | `crates/infrastructure/src/secret/keyring_vault.rs:339` | with a stale `secrets.json` present and the file fallback disabled: the file is not rewritten, a stored value round-trips through the session store, and a key absent from the OS keyring resolves to `None` — never to the stale file |
| `session_fallback_round_trips_without_writing_a_file` (pre-existing) | `crates/infrastructure/src/secret/keyring_vault.rs:298` | the release-shaped store keeps working with no file on disk |
| `missing_os_keyring_secret_returns_none_without_fallback` (pre-existing) | `crates/infrastructure/src/secret/keyring_vault.rs:285` | a missing keyring item with no fallback is `Ok(None)` or a named `EncryptionFailed`, never a panic |
| `new_vault_fails_closed_without_os_keyring` (pre-existing) | `crates/infrastructure/src/secret/keyring_vault.rs:279` | a vault with no fallback at all still fails closed |

Release-profile run of the runtime crate:

```
$ cargo test -p db-pro-runtime --release --lib
test tests::release_secret_store_never_selects_the_file_fallback_by_default ... ok
test tests::shipping_secret_store_follows_the_build_profile ... ok
test tests::file_fallback_needs_a_debug_build_or_an_explicit_opt_in ... ok
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(The same suite reports 31 in a debug run; the extra release-only test is the profile guard.)

## 6. Gate results on this tree

Environment: `DATABASE_URL` unset, `DB_PRO_ALLOW_FILE_SECRET_FALLBACK` unset.

```
$ cargo fmt --all -- --check                              # exit 0
$ cargo check --workspace                                 # exit 0
$ cargo clippy --workspace --all-targets -- -D warnings    # exit 0, 0 warnings
$ cargo test --workspace                                   # exit 0 — 827 passed; 0 failed; 19 ignored
$ cargo build --release --locked -p db-pro-native          # exit 0
$ bash .skills/perf-audit/scripts/perf-scan.sh             # PASS — 4 passed, 0 warnings, 0 failed
```

Suite breakdown of the `cargo test --workspace` run:

| Suite | Result |
|---|---|
| `db-pro-core` (lib) | 298 passed |
| `db-pro-infrastructure` (lib) | 64 passed |
| `db-pro-infrastructure` (tests/integration.rs) | 32 passed |
| `db-pro-infrastructure` (tests/pg_integration.rs) | 0 passed, 18 ignored (needs `DATABASE_URL`) |
| `db-pro-infrastructure` schema/ssh runtime verification suites | 8 passed, 1 ignored |
| `db-pro-native` | 14 passed |
| `db-pro-runtime` (lib) | 31 passed |
| `db-pro-tauri` (lib) | 21 passed |
| `db-pro-ui` (lib) | 359 passed |

**Baseline vs delta.** The measured baseline at this tree's start commit `1ff33ae` (changes stashed,
`cargo test --workspace` re-run) is **823 passed / 0 failed / 19 ignored**. This change adds four
tests — 2 in `db-pro-infrastructure` (62 → 64) and 2 in `db-pro-runtime` (29 → 31) — so the tree now
reports **827 / 0 / 19**, delta **+4**, with no test changing status. The previously recorded
baseline of 822 was measured before PR #181 merged; that merge added
`apply_mutations_detailed_maps_statement_index_to_original_input_mutation`
(`crates/core/src/application/table_data_service.rs`), which accounts for the 823 starting figure.

## 7. Documents corrected (they described the behaviour wrongly)

| Document | Was | Now |
|---|---|---|
| `docs/architecture/security-boundaries.md:35-44` | "Fallback: AES-256-GCM encrypted file (dev/CI only, disabled in production)" — a claim the code did not implement | primary keyring in every build; session fallback always on; file fallback gated to debug or `DB_PRO_ALLOW_FILE_SECRET_FALLBACK`, never consulted in a release build (with `file:line` for each claim) |
| `docs/release/provider-capability-matrix.md:41` | "encrypted fallback is dev-only (#142)" — correct intent, unverifiable in code at the time | keyring primary in every build; release degradation is session-memory only; file fallback is debug/opt-in-gated (#142) |
| `docs/release/risk-register.md` (`R011`) | "The encrypted fallback remains available and is no longer the production default", feature list naming `sync-secret-service` | corrected to what the root `Cargo.toml:29-33` actually selects (`apple-native`, `windows-native`, `linux-native-sync-persistent`) and to the gated behaviour; new `R-KEYRING-FALLBACK` entry (`FIXED`) plus a §2 summary row and a §0 revision note |

## 8. Scope, and what was deliberately not done

- **`R-KEYRING-STALL` / `LIM-018` is untouched.** The unbounded keychain read on the startup path
  (`providers/16`, `providers/22`) is a separate, already-classified finding whose recorded owner
  decision is to keep it disclosed for 0.1.0. This change neither fixes nor worsens it: the release
  store still reads the keyring on the same path.
- **The debug read order was not changed.** The file fallback still wins over the keyring when it is
  enabled (`keyring_vault.rs:189`), which is the point of a dev convenience store; it is now
  impossible to enable it in a release build by accident.
- **`keyring::Entry::set_password` failures are still discarded** (`:181`), so a release build whose
  keychain write fails keeps only a session copy and reports success to the caller. That is
  pre-existing behaviour, unchanged here, and it is the reason the session fallback must stay
  unconditional; recording it as a separate item would be a follow-up, not part of #142.
- **No keyring item was read or written by hand during this work**, and no credential appears in
  this document or in the raw logs.
