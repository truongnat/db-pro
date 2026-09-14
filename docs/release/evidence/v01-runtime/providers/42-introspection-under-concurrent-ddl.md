# Schema refresh under concurrent DDL (#237)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 8c669b0` (clean at the start of the change); fix commit recorded in `LEDGER.md`
- Issue: **#237** ([P2][RC1][Introspection] Schema refresh fails when another session runs DDL
  concurrently) — filed and fixed in this session; parent area #26/#70
- Provider: live fixture container `dbpro-v01-pg-fixture` (`postgres:16`, host port 55432,
  `PostgreSQL 16.15`), `DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture`

## 1. How it surfaced

CI went red on `main @ 46b2bb5` (run **34900407192**, job "Rust checks", step `cargo test`):

```
test pg_introspect_tables ... FAILED
thread 'pg_introspect_tables' panicked at crates/infrastructure/src/postgres/introspect.rs:337:42:
called `Result::unwrap()` on an `Err` value:
  ColumnDecode { index: "\"definition\"", source: UnexpectedNullError }
test result: FAILED. 20 passed; 1 failed; 0 ignored
```

That commit is a documentation-only change on top of a commit that **added a live PG test** — the
failure was not caused by the new test's assertions but by the parallel schedule it changed: CI runs
`cargo test --all -- --include-ignored` (`ci.yml:127`), so the ignored PostgreSQL tests execute in
parallel, and the new test shifted the interleaving enough to hit a race that was always there.

## 2. The defect, measured

`introspect_indexes` decoded `pg_get_indexdef(i.indexrelid) AS definition` as a non-nullable `String`
(`introspect.rs:337`), and `introspect_check_constraints` did the same for `pg_get_constraintdef`
(`:661`). Both are **catalog functions evaluated with a fresh snapshot** while the row set comes from
the statement's snapshot, so an index/constraint dropped by another session in between yields a row
whose `definition` is NULL.

Two measured proofs:

| Proof | Result |
|---|---|
| Mechanism: create an index, capture its OID, drop it, then `SELECT pg_get_indexdef(<oid>) IS NULL` | `t` (true) |
| Reproduction: create 80 indexes, drop them from a second session while `introspect()` runs | **fails in 0.65 s** with `ColumnDecode { index: "definition", UnexpectedNullError }` at `introspect.rs:337` (pre-fix) |

A second form of the same race cannot be absorbed by decoding at all — it is a server-side error raised
while the statement runs:

```
QueryFailed("cache lookup failed for attribute 1 of relation 17678")
```

## 3. The fix

1. **Tolerant decoding** at the two function-derived definition columns, matching how triggers, views
   and functions already decode (`optional_string(…).unwrap_or_default()`); a meanwhile-dropped object
   now degrades one row instead of failing the refresh. The indexes closure became
   `Result<Index, DbError>`-returning so the fallible decode can be propagated.
2. **Bounded retry** in `run_introspection` (`introspect.rs:7`): three attempts with a 10/20 ms
   backoff, and **only** for the narrow transient catalog-error messages —
   `cache lookup failed`, `could not open relation with OID`, `tuple concurrently updated`,
   `cached plan must not change result type`. Every other error is returned unchanged, so a real
   failure is not retried behind the user's back. The previous single-shot body became
   `run_introspection_once`.

## 4. The regression test

`pg_introspection_survives_concurrent_index_churn` (`crates/infrastructure/tests/pg_integration.rs`,
live, `#[ignore]`d behind `DATABASE_URL`), in two parts:

1. **the mechanism** — create an index, read its OID (a column that also pins the OID→`Int64` decode),
   drop it, and assert `pg_get_indexdef(<oid>) IS NULL` is `true`;
2. **the race** — create 40 indexes on a scratch table, drop them from a second connection while the
   first one refreshes the schema repeatedly, and require every refresh to succeed; the scratch table is
   dropped at the end.

**Fails before / passes after** (same command, source file stashed and restored):

```
$ git stash push -- crates/infrastructure/src/postgres/introspect.rs
$ DATABASE_URL=… cargo test -p db-pro-infrastructure --test pg_integration -- --include-ignored \
    pg_introspection_survives_concurrent_index_churn
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 24 filtered out
$ git stash pop
$ … same command
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out
```

## 5. Verification

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo check --workspace` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | **863 passed / 0 failed / 26 ignored** (baseline 863/0/25; the new case is `#[ignore]`d) |
| `cargo build --release --locked -p db-pro-native` | exit 0 |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | `Status: PASS` (4/0/0) |
| **CI-mirroring** `DATABASE_URL=… cargo test --all -- --include-ignored` (§CI's own invocation) | **889 passed / 0 failed / 0 ignored** |
| Parallel re-runs of the integration binary (`--include-ignored`, default threads) | 3 × `25 passed / 0 failed` |

The SSH-gated case (`crates/infrastructure/tests/ssh_backup_runtime_verification.rs`) needs the nine
`DB_PRO_SSH_*` variables CI configures and does not fail locally without them, so the CI-mirroring run
above is green on this host; the CI job itself remains the authority for that leg.

## 6. Not claimed here

- That the race is fully eliminated: concurrent DDL can still produce a *different* consistency outcome
  (the refresh may miss or include an object that appears mid-flight). The claim is narrower and
  testable — **a refresh no longer fails** because of it, and genuine errors are not retried.
- Windows/Linux behaviour: measured on macOS against `postgres:16` only.
