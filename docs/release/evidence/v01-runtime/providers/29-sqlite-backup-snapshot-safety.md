# SQLite backup and restore are snapshot-safe (#145)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 5ef1cd8` (clean at the start of the change); fix commit recorded in `LEDGER.md`
  (`fix(data-safety): make the SQLite backup and restore snapshot-safe (#145)`), pushed to `origin/main`
- Issue: **#145** ([P1][RC1][Data Safety] Make SQLite backup and restore snapshot-safe) — source audit
  **#128**, parent goal **#14**
- Baseline finding in the issue body: `15c4e0b1cfb222087800dabeab90fb17cb86bd63`

## 1. Measured behaviour before the change

The issue's premise ("direct `tokio::fs::copy` of the live database file") is **stale**: the engine has
used `VACUUM INTO` for backup and stage → validate → rename for restore for some time
(`crates/infrastructure/src/backup/sqlite_backup.rs:131`, `:184-201`). The parts that were *not*
defensible were measured directly with throwaway probes (raw output kept in §2/§3) before any fix.

| # | Scenario | Measured result | Verdict |
|---|---|---|---|
| T1 | WAL source, a second connection holding `BEGIN IMMEDIATE` with an uncommitted row, backup runs | backup succeeds, contains exactly the committed row `["before"]`; the source later commits to `["before", "uncommitted"]` | **safe** — a real snapshot |
| T1b | rollback-journal source with `BEGIN IMMEDIATE` open | backup succeeds, no torn artifact | safe |
| T2 | WAL source, committed writes still in `-wal` (writer connection open, nothing checkpointed) | backup contains the committed row | **safe** |
| T5 | leftovers after a backup of a WAL source | directory holds only `backup.db` + `source.db`; no `.db-pro-*.tmp`, no temp sidecars | safe |
| T4 | restore whose publish step fails (target path is a non-empty directory) | `Err(Internal("failed to publish SQLite restore: Is a directory (os error 21)"))`, target directory and its file intact, no staged file left | **safe** |
| T7 | restore over a target with a live `-wal`/`-shm` from a WAL-mode session | **`restore` returns `Ok(())` and a reader still sees the OLD database**: `["from-old-db"]`, and the target's header reads WAL (`[2, 2]`) because the old WAL's page 1 was applied to the file that replaced it | **DEFECT** |
| T7-cold | same, but with the `-wal`/`-shm` pair left by a *crashed* session (no open connection at all) | `Ok(())`, sidecars still present, content read back `["from-old-db"]` | **DEFECT** (deterministic, no concurrency needed) |
| T6 | backup published at a path that already has a *garbage* `-wal` | artifact reads back correctly (invalid WALs are ignored) | safe |
| T8 | backup published at a path that already has a *valid* `-wal` of another (deleted) database | **artifact reads back as the other database**: `["foreign-wal-content"]` | **DEFECT** |

Both defects have the same root cause: **SQLite does not bind a WAL to the database file it sits next
to.** Replacing the main file — by `rename` on restore, by `hard_link` on backup — leaves the old
`-wal`/`-shm` in place, and the next reader replays those frames over the new file. `-journal` is
included in the same cleanup because a leftover rollback journal is only self-invalidating when its
recorded change counter no longer matches the database header; removing it removes the question.

## 2. The fix

### Engine — `crates/infrastructure/src/backup/sqlite_backup.rs`

| Change | Location |
|---|---|
| `sidecar_paths` — the three sidecar names SQLite keeps next to a database | `:30` |
| `remove_database_sidecars` — removes them, tolerates `NotFound`, reports any other error instead of publishing | `:38` |
| `flush_file` — `sync_all` on the artifact before it is published by name | `:80`, called at `:143` |
| backup: stale sidecars at the output path are removed before the publish | `:155` (publish at `:160`) |
| restore: stale sidecars at the target are removed after the staged copy validates and immediately before the atomic rename | `:216` (rename at `:221`) |

Ordering on restore is deliberate and documented in the code: the sidecar removal happens only once the
staged copy has been validated, so the only step that can still fail is the `rename` itself, and that
path is reported explicitly with the previous target still in place. The trade-off (a rename that fails
*after* the sidecars were removed leaves the old database without a hot journal) is accepted because the
alternative — removing them after the rename — recreates exactly the window this fix closes, and a crash
is far more likely than a rename failure on a validated path.

### Service — `crates/core/src/application/backup_service.rs`

| Requirement (issue) | Change | Location |
|---|---|---|
| "reject restore while the target has unsafe live handles" | already enforced; now pinned by a test (see §3) | `:70-73` |
| "define deterministic reconnect/cache/introspection behaviour after success" | a successful restore now drops the introspection cache for that connection, so the next connect cannot serve a schema that describes the replaced database | `:89` |

`BackupService::new` gained the `introspection_cache: Box<dyn IntrospectionCache>` parameter — the same
shape `SchemaService::new` already uses (`crates/core/src/application/schema_service.rs:20-25`); the
runtime wiring passes the meta store (`crates/runtime/src/lib.rs:170`), which is the implementation that
persists the cache (`crates/infrastructure/src/meta/introspection_cache_repo.rs`). A failed invalidation
warns and does not fail an already-completed restore, matching `ConnectionService::invalidate_introspection_cache`.

## 3. Tests

Eight regression tests; the two defect tests were run against the unfixed code first.

| Test | Location | Pre-fix result |
|---|---|---|
| `backup_captures_committed_writes_that_are_still_in_the_wal` | `crates/infrastructure/src/backup/sqlite_backup.rs:372` | passed (this is the *proof* the backup is already snapshot-safe) |
| `backup_during_an_open_write_transaction_is_a_consistent_snapshot` | `:402` | passed (same) |
| `backup_does_not_publish_next_to_a_stale_wal` | `:438` | **FAILED** — `the stale sidecar must not survive` |
| `restore_replaces_the_database_and_invalidates_stale_sidecars` | `:476` | **FAILED** — `the old database's WAL must not survive` |
| `failed_restore_keeps_the_previous_target_and_removes_the_staged_file` | `:523` | passed (this is the *proof* the failure path was already safe) |
| `failed_backup_leaves_no_artifact_and_no_staged_file` | `:561` | passed (pins the issue's "failed backup leaves no accepted partial artifact") |
| `sqlite_restore_is_refused_while_the_connection_is_active` | `crates/core/src/application/backup_service.rs:440` | passed (pins existing behaviour: the engine must not run at all) |
| `successful_restore_drops_the_cached_schema_for_the_connection` | `:491` | passed (pins the new invalidation) |

Pre-fix runs (`EXIT_STATUS=101`):

```
# restore-side fix reverted only
running 7 tests
test backup::sqlite_backup::tests::restore_replaces_the_database_and_invalidates_stale_sidecars ... FAILED
  panicked: the old database's WAL must not survive
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 62 filtered out

# backup-side fix reverted only
test backup::sqlite_backup::tests::backup_does_not_publish_next_to_a_stale_wal ... FAILED
  panicked: the stale sidecar must not survive
test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 62 filtered out

# after both fixes
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out
```

## 4. Full gate set (all six green, exit status 0)

| Gate | Command | Result |
|---|---|---|
| 1 | `cargo fmt --all -- --check` | `EXIT=0` |
| 2 | `cargo check --workspace` | `EXIT=0` |
| 3 | `cargo clippy --workspace --all-targets -- -D warnings` | `EXIT=0` |
| 4 | `cargo test --workspace` | **837 passed / 0 failed / 19 ignored** (19 suites) |
| 5 | `cargo build --release --locked -p db-pro-native` | `EXIT=0` |
| 6 | `bash .skills/perf-audit/scripts/perf-scan.sh` | `Passed 4 / Warnings 0 / Failed 0` — `Status: PASS`, `EXIT=0` |

Delta versus the session baseline **827 / 0 / 19**: **+10 passed** (2 from the #236 fix, 8 from this one),
failed and ignored unchanged. The 19 ignored are the standing 18 `pg_integration` (need `DATABASE_URL`)
plus 1 SSH test (nine `DB_PRO_SSH_*` variables) — untouched here.

## 5. Requirement-by-requirement disposition (issue #145)

| Issue requirement | Status |
|---|---|
| Backup: SQLite-supported snapshot mechanism, not a raw copy | **already satisfied** — `VACUUM INTO` (`sqlite_backup.rs:131`), proved by T1/T2 and two tests |
| Backup: handle WAL/journal state correctly | **satisfied** — WAL/rollback writers measured; the artifact now also cannot inherit a stale sidecar from its output path (T8) |
| Backup: temporary destination, publish only after success | **already satisfied** — `VACUUM INTO` into `.db-pro-<uuid>.tmp`, published by `hard_link` with a no-overwrite check (`:57-76`) |
| Backup: clean up partial output on failure/**cancel** | **failure cleanup satisfied and now tested**: a backup of a non-database fails, publishes nothing and leaves no `.db-pro-*` staged file (`failed_backup_leaves_no_artifact_and_no_staged_file`); the same is pinned for restore (`failed_restore_keeps_the_previous_target_and_removes_the_staged_file`). **Cancel is not implemented**: `RuntimeCommand::CancelOperation` exists (`crates/runtime/src/worker.rs:205`, handled at `:1563`) but is never constructed anywhere in the workspace (`grep -rn CancelOperation crates/` returns only those two lines), so a running backup cannot be cancelled — that is #128's cancellation/partial-failure semantics, unchanged here |
| Backup: report failures explicitly | satisfied — every branch returns a distinct `DbError` (`NotFound` / `Validation` / `Internal`) |
| Restore: reject while unsafe live handles / explicit close-reopen | **satisfied** — `BackupService` refuses while the connection is registered active (`backup_service.rs:70-73`), now pinned by a test. Sidecar files left by an *external* process are still not detectable from here; the restored file no longer inherits them, which is what the measurement showed to matter |
| Restore: validate source readability before replacing | already satisfied (`:199-209`, staging + `quick_check` on both the input and the staged copy) |
| Restore: avoid partially replacing the target on failure | **already satisfied** — single atomic `rename` over a validated staged copy (T4) |
| Restore: deterministic reconnect/cache/introspection after success | **now satisfied** — the persisted introspection cache is invalidated for that connection (`backup_service.rs:89`) |
| Restore: preserve or clearly fail on permission/path errors | satisfied — path/permission failures surface as explicit errors with the previous target intact (T4) |
| Tests/evidence: exact-head Rust tests **and packaged runtime smoke** before completion | Rust/integration side **done** (837/0/19, this file). **Packaged-runtime smoke is not done** — it needs a GUI session and a packaged artifact (`#91`, `HD-007`), so no runtime claim is made here |
| Merge gate: "accepted on one exact SHA and reflected in #128/#136" | **not done** — #136 is the owner's risk register and #128 the umbrella audit; both stay open. The fix commit SHA is recorded in `LEDGER.md` for that pass |

## 6. What this does not claim

- **No packaged-runtime or GUI verification.** The proof is source-level plus deterministic file-level
  tests; `BUILD_VERIFIED` is not `RUNTIME_VERIFIED`.
- **No PostgreSQL change.** `PgDumpEngine` is untouched; the restore cache invalidation does apply to it
  as well, which is the correct behaviour for a replaced database but is not separately tested here.
- **The rename is not fsynced.** The artifact is flushed before publish (`sync_all` on the temp file), but
  the directory entry created by `rename`/`hard_link` is not synced, so a machine crash immediately after
  a successful restore can lose the new file — it cannot leave a *torn* one, which is what the issue
  requires. Directory fsync is not portable through `std`/`tokio` without a platform branch and was left
  out deliberately.
