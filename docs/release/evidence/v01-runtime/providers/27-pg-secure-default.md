# PostgreSQL secure-by-default TLS — triage and blocker (#144)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 1a0973c` (worktree clean)
- Verdict: **NOT IMPLEMENTED — BLOCKED, owner decision required.** The defect is confirmed; no
  variant of the fix can land without breaking connection compatibility or a documented capability
  contract. No product code was changed by this triage.
- Issue: **#144** ([P1][RC1][Security] Make PostgreSQL remote connections secure by default);
  source audit #125 (closed); merge gate also names #136 (still open)
- Status comment: https://github.com/truongnat/db-pro/issues/144#issuecomment-5668118059

## 1. The defect, confirmed

| Location | Fact |
|---|---|
| `crates/core/src/domain/connection.rs:47-53` | `SslMode` is `{ Disable, Require, VerifyCa, VerifyFull }` with `#[default] Disable` |
| `crates/core/src/domain/connection.rs:87-88` | `ConnectionConfig.ssl_mode` is `#[serde(default)]`, so its parse fallback **is** `SslMode::default()` |
| `crates/infrastructure/src/postgres/connection_string.rs:6-10` | 1:1 exhaustive mapping onto `PgSslMode`; no mode is lost or invented |
| `crates/ui/src/runtime.rs:81` | new-connection draft default is `UiSslMode::Disable` |
| `crates/ui/src/runtime.rs:304-313` | `UiConnectionSummary` has **no** `ssl_mode` field |
| `crates/ui/src/connection_view.rs:147`, `:176` | edit and duplicate build the draft with a **hardcoded** `UiSslMode::Disable` |
| `crates/ui/src/connection_view.rs:622-638` | 4-option segmented control `["Disable", "Require", "Verify CA", "Verify Full"]`, index-mapped |

A non-local PostgreSQL connection therefore sends credentials and query traffic in plaintext unless
the user changes the segmented control.

## 2. Mode inventory — why each candidate fails

The domain exposes four modes. `Prefer` and `Allow` **do not exist** in it, although the locked
SQLx (`sqlx-postgres 0.8.6`, `Cargo.lock:6158`) defines both and uses `Prefer` as its own default
(`sqlx-postgres-0.8.6/src/options/ssl_mode.rs:8-31`).

| Candidate | Verification | Compatibility | Verdict |
|---|---|---|---|
| `Disable` | none | — | the defect |
| `Require` | **none unless a root CA file is present** (`sqlx-postgres-0.8.6/src/options/ssl_mode.rs:21-23`) | new connection to a TLS-less server fails where it previously worked | encrypts, but the brief rules out the compatibility break |
| `VerifyCa` / `VerifyFull` | CA + (full) hostname | unsatisfiable for private-CA/self-signed remotes | **no CA/client-cert fields exist in the app** |
| `Prefer` | none (no certificate verification at all) | non-breaking (falls back to plaintext) | needs a **new** domain variant + DTO + UI option, and still silently falls back to plaintext |

Evidence that the app cannot supply a CA:

```
$ grep -rniE 'root_cert|sslrootcert|client_cert|client_key|ca_cert' crates/ --include='*.rs'
(no matches)
```

corroborated by `docs/notes/PRODUCT_CAPABILITY_MATRIX.md:357`
("SSL/TLS (mode-only, rustls) … **No CA/cert fields**; all tests use Disable").

`Prefer` also fails this issue's first acceptance requirement — *"a newly configured non-local
PostgreSQL connection must not silently default to plaintext transport"* — because plaintext
fallback is its defining behaviour, and it is strippable by an active attacker.

Adding a `Prefer` variant is additionally forbidden by the issue itself: *"Do not silently invent
`sslmode` semantics unsupported by the current domain/SQLx mapping."*

## 3. Documented capability contract that a secure default would break

| Document | Line | Statement |
|---|---|---|
| `docs/release/provider-capability-matrix.md` | 42 | `TLS/SSL \| SUPPORTED + **NOT YET QUALIFIED** \| N/A \| PG via runtime-tokio-rustls` |
| `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` | 357 | `SSL/TLS (mode-only, rustls) … No CA/cert fields; all tests use Disable` |
| `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` | 350 | duplicate resets password/SSL/SSH **by design** |

Promoting TLS into the default path makes an explicitly unqualified capability the default, and a
verifying default contradicts the documented absence of CA/certificate input.

## 4. Saved-state hazard

`ConnectionConfig.ssl_mode` is `#[serde(default)]`, so the fallback used when parsing persisted
`Connection` JSON that omits the field is exactly `SslMode::default()`. Changing the enum default
therefore changes how **already-persisted** bytes are interpreted: the same record would no longer
round-trip to the same mode, and a pre-change connection could change transport without the user
touching it.

Keeping legacy records stable requires pinning an explicit serde fallback (still `Disable`) that is
**separate** from the new-connection default. The two meanings are one constant today, and the issue
requires *"persisted/parsed connection settings round-trip deterministically"* — so the split has to
be designed deliberately, not assumed.

## 5. The UI cannot round-trip a stored mode, so "new connections only" is not implementable as-is

`UiConnectionSummary` (`crates/ui/src/runtime.rs:304-313`) carries `host`, `port`, `database`,
`username`, `driver`, `readonly` — but **not** `ssl_mode`, and both the edit and duplicate paths
hardcode `UiSslMode::Disable`.

Consequences:

- Changing only the new-connection default creates a **new** defect: a freshly created,
  secure-by-default connection is silently downgraded to `Disable` the first time the user opens and
  saves it.
- The test `default_draft_neutral_fields_are_profile_independent`
  (`crates/ui/src/runtime.rs:856`) pins `draft.ssl_mode == UiSslMode::Disable` and would have to
  change.
- A correct implementation needs `ssl_mode` on `UiConnectionSummary`, plumbing through the runtime
  mapping, and a decided edit semantic — a UI/runtime change, not a domain default flip.
- Separately observed (pre-existing, not caused by this triage): because the edit path hardcodes
  `Disable`, editing a connection that stores `Require`/`VerifyFull` silently **rewrites it to
  `Disable`**. That is a security downgrade in its own right and is not covered by any current issue.

## 6. Merge gate

> *"P1 remains open until the secure-default policy is **accepted** on one exact SHA and #125/#136
> are updated."*

#125 is closed (audit complete). **#136 — the central release risk and decision register — is still
open**, so no policy has been accepted and recorded. The issue offers materially different
alternatives ("a safer default plus explicit opt-out, or a host-aware UI policy"), and explicitly
disallows a documentation-only disposition: *"RC1 severity rules do not allow a P1
connection-security gap to be accepted only through documentation."*

## 7. Decision required from the owner

1. **New-connection default** for non-local PostgreSQL: `Require` (TLS-less servers fail by default),
   or a new `Prefer` mode (new variant + DTO + UI option + docs, plaintext fallback still possible),
   or `VerifyFull` (CA/client-cert input must ship first)?
2. **Host-aware policy?** localhost/loopback permissive, non-local secure — needs a definition of
   "local" and a rule for host edits after the mode was chosen.
3. **Does CA/client-certificate input ship in v0.1?** Without it no verifying mode is usable against
   private-CA servers; if it does not ship, the accurate disposition is a documented v0.1 limitation
   rather than a code default change.
4. **Is the `Require` compatibility break acceptable** for newly created connections?

## 8. Verification note

No product code, test, or configuration was changed by this triage, so no workspace gate was re-run;
the counts recorded elsewhere in this directory are unaffected. The only repository change is this
record plus the ledger row update.
