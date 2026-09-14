# Findings, classifications and dispositions — `v01-runtime` run

- Session: `v01-runtime`, 2026-09-14, repo HEAD `8e1d63f`
- Bug policy applied: brief §40 (`P0` data loss / security leak / unusable app / destructive op
  executing incorrectly; `P1` core workflow broken / crash / stale state hitting the wrong target
  / unsafe DB op / release-blocking startup; `P2` workflow inconvenience or low-risk incorrect UX;
  `P3` cosmetic).
- **No production code was changed in this run**, so no `P0`/`P1` was found-and-fixed and the six
  gates were not re-minted. Every finding below is `P2`/`P3` and is **recorded, not chased**.

## F1 — `pg_restore` reports failure under client/server major-version skew, after a complete restore — `P2`

**Observed.** With the host's PATH tools (PostgreSQL client **18.4**) against the fixture server
(**16.15**), a custom-format restore **restores everything correctly but exits 1**:

```
pg_restore: error: could not execute query: ERROR:  unrecognized configuration parameter "transaction_timeout"
Command was: SET transaction_timeout = 0;
pg_restore: warning: errors ignored on restore: 1
```

`transaction_timeout` was introduced in **PostgreSQL 17**; the server is 16.15.
Evidence: `10-pg-backup-restore-commands.txt`.

**The restore is nevertheless complete.** Object counts (10 tables / 2 views / 17 indexes /
8 sequences), row counts, per-table MD5 content fingerprints, the enum type, the trigger, both
functions and a working view all match the source exactly, and `fixtures/postgres/003_verify.sql`
passes against the restored database. Evidence: `11-pg-restore-verification.txt`,
`12-pg-restore-comparison-and-skew-control.txt`.

**Root cause isolated by control experiment.** Dumping *and* restoring with the version-matched
16.15 tools inside the container gives **exit 0** and identical data (`14-…control.txt`). The
nonzero exit is therefore PostgreSQL tooling version skew, not a data or fixture defect. A second
control showed the skew is stronger than a warning: a 16.15 `pg_restore` **cannot read at all** an
archive written by 18.4 (`unsupported version (1.16) in file header`), consistent with
PostgreSQL's rule that the restore tool must be same-or-newer than the dumper.

**Why this is a `P2`, not a `P1`.** No data is lost and nothing is written to the wrong target —
the restore is correct; only the reported *status* is wrong. The app surfaces it because
`crates/infrastructure/src/backup/pg_dump.rs:167-170` treats any non-zero `pg_restore` exit as
`restore failed: …`. Note the plain-format path is unaffected: it uses `psql -f`
(`pg_dump.rs:143-147`), which exits 0 on the same skew while logging the `ERROR` line
(`13-…controls.txt`).

**Disposition: recorded, not chased.** Failing on a nonzero exit is the *safe* default; weakening
it is a production change to the credential/backup path, out of scope for this run and not
requested by the brief (which asks only for CLI-level evidence that the documented `pg_dump` /
`pg_restore` dependency is satisfiable — it is). Recommended follow-up for the owner: state the
client/server version expectation in `platform-prerequisites.md`, and optionally detect the
version pair before a custom-format restore.

**Impact on the release claim.** None of the existing release text changes: `pg_dump`/`pg_restore`
remain "must be on `PATH`; not bundled" (`docs/release/0.1.0-readiness.md`, Known limitations).

## F2 — `epaint` font-atlas glyph fallback warning — `P3`

**Observed.** With `RUST_LOG=info`, the packaged binary emits 13 identical lines:

```
2026-09-14T16:06:33.200604Z  WARN epaint::text::font: Failed to find replacement characters '◻' or '?'. Will use empty glyph.
```

A glyph requested by the bundled icon font (`◻`, U+25FB) has neither a glyph in the font nor a
resolvable replacement, so egui draws an empty glyph. Source is `egui`/`epaint`, not DB Pro code.
Evidence: `19-error-log-audit.txt`.

**Disposition: recorded, not chased** (`P3`, cosmetic). Whether it is visible on screen is **not
verifiable here** (no window server), so no visual claim is made either way. It is emitted only
when `RUST_LOG` is set; with `RUST_LOG` unset, launches A, B, C and branches 1/2/4 produced
**zero bytes** of output.

## F3 — keyring startup stall — `P2`

Reproduced, classified and dispositioned in `22-keyring-stall-classification.md`. Summary: the
launch blocks indefinitely inside `SecKeychainFindGenericPassword` → `securityd` decrypt →
`mach_msg`, before the data directory is resolved; it needs a pre-existing `com.dbpro.app` item
and a context where the authorization prompt cannot be shown; a user with no stored item is
unaffected (proven by control Launch B). `P2`, already tracked as `R-KEYRING-STALL` / `LIM-018`,
recorded not chased, with an explicit escalation condition stated for the coordinator.

## No-finding areas (checked, nothing to report)

| Area | Result |
|---|---|
| PostgreSQL cancellation capability gating | **Verified correct** — no bug. `15-cancellation-capability-gating.md` |
| SQLite fixture correctness | Deterministic; two independent builds are byte-identical (`05-…determinism.txt`) |
| Live PostgreSQL integration suite | **18 passed / 0 failed** (`07-pg-integration-live.txt`) |
| Live PostgreSQL fixture integrity after the suite | Object and row counts identical to the pre-test baseline; no leftover `pg_index_lifecycle` objects (`09-…txt`) |
| State-directory resolution | All four branches behave as documented; `/.db-pro-data` never created; nothing written inside the bundle (`17-`, `18-…txt`) |
| Launch stdout/stderr | No `panic`, no unwrap-on-`Err`, no channel/stale-event/db-lock/keyring/worker/provider errors (`19-…txt`) |
