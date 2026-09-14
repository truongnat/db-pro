# Keyring and encrypted-fallback secret lifecycle — audit (#126)

- **Audited baseline:** `main @ 6446d94` (issue-queue pass 3, 2026-09-15)
- **Issue:** #126 ([RC1][Trust] Audit keyring and encrypted-fallback secret lifecycle) — parent **#14**,
  supports #27/#29/#122/#105
- **Predecessors:** #142 (file fallback removed from the release path, `78fb39b8`;
  risk-register `R-KEYRING-FALLBACK` = `FIXED`) and the #122 trust audit
  (`docs/release/audit-security-boundaries.md` §2).
- **Status of this issue: left OPEN.** The lifecycle audit is complete here; the issue's own closure gate
  (agreed in the 2026-08-14 review comments) additionally requires the packaged per-platform proof, which
  needs Windows/Linux hosts this environment does not have (#92/#93, `R-WINLINUX`) and a GUI session for
  the macOS Keychain prompt path (#91, `R-GUI-SMOKE`, `R-KEYRING-STALL`). Findings are filed as a child
  issue rather than patched here.

## 1. Storage wiring (what is on in which build)

`build_secret_store` (`crates/runtime/src/lib.rs:99-110`) constructs `KeyringVault::new(KEYRING_SERVICE,
secrets_dir)` (service `com.dbpro.app`, `:62`; dir `<data_dir>/secrets`, `:118`) with the session
fallback always attached, and the encrypted-file fallback only when
`file_secret_fallback_enabled_for(debug_build, opt_in)` (`:83-91`) allows it.

| Build | OS keyring | in-memory session map | encrypted `secrets.json` |
|---|---|---|---|
| debug | attempted | ON | ON |
| release, no env | attempted | ON | **OFF** |
| release, `DB_PRO_ALLOW_FILE_SECRET_FALLBACK=1/true` | attempted | ON | ON |

Keyring *availability* is never probed at startup — it is attempted per operation. The fallback file path
is `<data_dir>/secrets/secrets.json` (`crates/infrastructure/src/secret/keyring_vault.rs:87`); its key is
derived from the **service name** and documented `DEV-ONLY` (`:106-120`), so the opt-in build is not
described as secure here.

Backends linked per OS (workspace `Cargo.toml:29-33` + `crates/native-app/Cargo.toml:22`):
`apple-native` (macOS Keychain), `windows-native` (Credential Manager), and on Linux the **combo**
`linux-native-sync-persistent` = `linux-native` + `sync-secret-service`, which keyring 3.6.3 resolves to
`keyutils_persistent` (`~/.cargo/registry/…/keyring-3.6.3/src/keyutils_persistent.rs:1-27`): kernel keyutils
as a headless cache **plus** the Secret Service over session D-Bus for anything that must survive a reboot.
With no Secret Service running, Linux writes/reads fail as `NoStorageAccess`/`PlatformFailure`, which the
vault classifies as "keyring unavailable" and — see event 3 — **swallows**.

## 2. Lifecycle matrix

| Event | Metadata state | Keyring state | Fallback state | Expected retrieval | Failure behaviour |
|---|---|---|---|---|---|
| **Create (Postgres)** `connection_service.rs:63-114` | row written **after** the secret (`repo.save` at `:99`) | `connection/{id}/password` written best-effort (`keyring_vault.rs:179-181`) | session always; file when enabled | `retrieve_secret` at connect | empty password → `AuthFailed("database password is required for PostgreSQL")` (`:66-70`); secret-store error **aborts before** the row is written (`:79`); row-write failure deletes both secrets (`:99-111`, logs only) |
| **Create (SQLite)** `:78-84` | `secret_ref = None` | nothing | nothing | no secret read (`SQLiteConnector::connect` ignores the password) | n/a — SQLite has no password concept here (§4) |
| **Edit (blank password — the documented path)** | `secret_ref` unchanged; host/user/database may change | **unchanged** | unchanged | the previously stored password | decided in two places: the UI maps `""` → `None` (`crates/native-app/src/translate.rs:447`) and the service only stores when `Some` (`connection_service.rs:180-187`). The UI hint says "Leave blank to keep saved password" (`connection_view.rs:657`). **Recorded:** a *host/username* change keeps the old password silently — the mismatch surfaces only when a connect/test fails |
| **Edit (password supplied)** | `secret_ref` set | overwritten | overwritten | the new password | `store_secret` error propagates; the update is not persisted |
| **Edit (driver leaves Postgres / SSH removed)** `:191,206` | `secret_ref` cleared | deleted | deleted | none | delete is best-effort (event 3) |
| **Delete connection** `:271-321` | disconnect first, then secrets, **then** the row (`:296-316`) | deleted | deleted | none | row missing → **no secret delete attempted** (`:277-286`) = orphan; `repo.delete` failure restores the secrets (`:309-315`, logs only); a crash between the secret deletes and the row delete leaves a row whose password is gone |
| **Connect / reconnect** `:323-364` | — | read (or `NoEntry`) | file → session → keyring | `AuthFailed("password not found in secret store")` when absent (`:340`) | already-active connection returns the registered handle without touching secrets (`:324-326`) |
| **Restart** | — | read | session map is empty; file only in debug/opt-in | as above | **a keyring write that silently failed becomes this error after a restart** (event 3) — the password must be re-entered |
| **Keyring unavailable** (`NoStorageAccess`/`PlatformFailure`) `keyring_vault.rs:152-167` | — | unavailable | `require_fallback()` → file or session | session value if the process stored one; otherwise `Ok(None)` → the connect error | `require_fallback` errors only when **both** fallbacks are off, which the shipping wiring never is (`:93-104`), so the app degrades to the session store |
| **Corrupt / undecryptable fallback file** | — | — | read error | `DbError::EncryptionFailed` propagates on **read** (`fallback.rs:31-42`) | on **delete** the corrupt file is silently skipped (`keyring_vault.rs:231-235`); on **write** the parse error aborts the store (`fallback.rs:103-116`). Only reachable in debug/opt-in builds |
| **Legacy connection without `secret_ref`** | `secret_ref = None` | default key `connection/{id}/password` | — | the default key (`:335`) | deleting it is covered by `delete_legacy_postgres_removes_default_secret_key` |
| **Agent API key** `crates/native-app/src/main.rs:198-234` | not in `meta.db` | `agent/groq_api_key` via **raw `keyring::Entry`**, bypassing the vault | none | seeded into the env at startup (`:205-220`) | failures only `tracing::warn!`; **there is no delete path in the UI** |

## 3. Findings

| # | Severity | Finding | Disposition |
|---|---|---|---|
| K-1 | **P2** | **A failed keyring write is reported as success.** `store_secret` ends with `let _ = entry.set_password(value);` (`keyring_vault.rs:179-181`) and returns `Ok(())`, and `delete_secret` does the same for all three layers (`:230-244`). In a release build (no file fallback) a denied/absent keyring therefore means: the UI says the connection was saved, the password works for the rest of the session through the session map, and after a restart the connect fails with `password not found in secret store`. Linux without a Secret Service is exactly this case (renders unusable results for a headless user, and the keyutils-only path does not survive a reboot) | **`Fix RC1` — child issue #243**: surface a store/delete failure to the caller, and add the missing lifecycle tests (§5) |
| K-2 | **P2** | **Orphans are possible and invisible.** Deleting a connection whose row is already gone deletes nothing (`connection_service.rs:277-286`); a failed `delete_secret` is indistinguishable from success (K-1); and the `SecretStore` port has no enumeration, so a sweep is not even expressible (`crates/core/src/ports/secret_store.rs:9-15`) | **`Fix RC1` — child issue #243** (minimum: report the failure; optionally extend the port with a listing for a startup sweep) |
| K-3 | **P2** | **The agent API key has no lifecycle.** It is written and read through raw `keyring::Entry` in `crates/native-app/src/main.rs:205-234`, bypassing the vault (no session fallback, no error surface), and the app offers no way to **remove** it. A key that the user once saved keeps being used for automatic prediction egress (#242) | **`Fix RC1` — child issue #243** (route it through the vault and add "Forget key") |
| K-4 | **P2** | **Linux persistence semantics are weaker than the storage name suggests.** The shipped feature set is keyutils + Secret Service; without a running Secret Service nothing survives a reboot, and the keyutils-only path cannot store empty secrets | **`Accept RC1`** for v0.1 (Linux is not release-qualified: #93, `R-WINLINUX`, LIM-017/018) with the wording recorded here; the owner decision on a persistent Linux backend is named in the #126 blocker comment |
| K-5 | **P3** | **Test coverage gaps**: no test exists for a store failure, a retrieve `Err`, a vault-level delete (the session test never re-reads), a delete with an unavailable keyring, an edit that must *keep* the secret, restart retrieval, or a corrupt fallback file. `missing_os_keyring_secret_returns_none_without_fallback` (`keyring_vault.rs:284`) accepts `Ok(None)` or `Err(…)`, so it cannot fail on either branch. `fallback.rs` and `encryption.rs` have **zero** tests | **`Fix RC1` — child issue #243** (the K-1/K-2 fixes need these tests); the accept-either assertion is named there too |
| K-6 | **P3** | **`SecretStore::delete_secret`'s `Ok` is not a proof**: the vault cannot distinguish "deleted" from "never existed" from "failed", which is what makes K-2 invisible | folded into K-2 |
| K-7 | **P3** | **Dead legacy key format**: `crates/core/src/domain/secret.rs:95-100` `connection:{id}:{field}` is referenced only by its own test; the live format is `connection/{id}/password` | **`Accept RC1`**, recorded (post-v0.1 cleanup with the other dead helpers from the #122 audit) |

**Positively verified (no action):** multi-connection isolation is per-UUID and cannot collide
(`connection_service.rs:41-47`); the SSH password is stored under its own key and stripped from metadata
(`#[serde(skip_serializing)]` + nulled before save, `connection.rs:63`, `connection_service.rs:95-97`);
create is ordered so a secret is never written without a row and a row never survives a failed secret
write; the file fallback is genuinely unreachable in a release build that did not opt in
(`disabled_file_fallback_never_touches_a_stale_fallback_file`, `runtime/src/lib.rs:362`); the debug
`Debug` impls print `[redacted]`/`has_password: bool`; SQLite requires no secret at all.

## 4. Explicit negatives

- SQLite has **no** password concept in this app: `requires_database_secret` is
  `matches!(driver, Postgres)` (`connection_service.rs:49-51`) and `SQLiteConnector::connect` ignores the
  password argument.
- No rename path exists (`grep -rn "RenameConnection\|rename_connection" crates` → no matches); a rename is
  an `UpdateConnection` with a new `config.name`, so the secret key (which is id-based) is unaffected.
- No startup orphan sweep exists (`grep -rni "orphan\|cleanup" crates` → only two log strings in
  `connection_service.rs:102,107`); the port cannot even enumerate.
- No key rotation, no expiry, no "secret age" concept anywhere.

## 5. Test inventory (what covers which event)

| Event | Covering test(s) |
|---|---|
| store + retrieve via session | `session_fallback_round_trips_without_writing_a_file` (`keyring_vault.rs:297`) |
| stale file ignored when the file fallback is off | `disabled_file_fallback_never_touches_a_stale_fallback_file` (`:338`) |
| fail-closed without any fallback | `new_vault_fails_closed_without_os_keyring` (`:278`) |
| opt-in gating (debug/opt-in/release) | `file_fallback_needs_a_debug_build_or_an_explicit_opt_in`, `shipping_secret_store_follows_the_build_profile`, `release_secret_store_never_selects_the_file_fallback_by_default` (`runtime/src/lib.rs:323,340,362`) |
| create/update/delete service flows, legacy key, SSH key, rollbacks | 24 tests in `crates/core/src/application/connection_service/tests.rs` (incl. `create_cleans_up_secret_on_repo_failure:573`, `delete_repo_failure_restores_secret:464`, `update_repo_failure_restores_missing_secret:655`) |
| provider-specific secret rules | 6 tests in `tests/connection_service_provider_tests.rs` (SQLite stores nothing, Postgres rejects an empty password, SQLite→Postgres needs a new password) |
| metadata hygiene | `ssh_password_is_not_serialized_or_debugged` (`connection.rs:452`), `sqlite_draft_drops_postgres_only_credentials` (`main.rs:416`) |
| **not covered at all** | store failure, retrieve `Err`, vault-level delete proof, delete with unavailable keyring, edit-keeps-secret, restart retrieval, corrupt fallback file, session-as-restart-degradation |

## 6. Packaged-smoke scenarios added for the human runbook

`docs/release/0.1.0-interactive-verification-runbook.md` §Phase 4 gained a credential step (Step 19) so
the packaged smoke covers what this audit could only read in source: save → relaunch → connect without
re-entering the password; edit with a blank password; delete → relaunch → confirm the connection is gone
and no stale prompt appears; and the keyring-unavailable case (deny the Keychain prompt or run with no
Secret Service on Linux) with what to record. Those runs remain `NOT VERIFIED` here and belong to #91/#92/#93.

## 7. Not claimed

- No per-platform packaged verification: no Windows/Linux host and no interactive macOS session exist here
  (#92/#93, #91).
- No dynamic test of the keyring backends themselves: what `keyring 3.6.3` does is quoted from the vendored
  crate source, not observed.
- No P0/P1: nothing here leaks a secret or loses database data; K-1/K-2/K-3 are silent-failure and
  lifecycle gaps with a user-visible recovery path (re-enter the password).
