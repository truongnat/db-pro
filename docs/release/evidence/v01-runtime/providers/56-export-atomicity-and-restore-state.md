# Export atomicity, restore partial-state reporting, and the cancellation record (#244)

- Session: issue-queue pass 5, 2026-09-15
- Issue: **#244** ([P2][RC1][Data Safety] Export is not atomic, PostgreSQL restore is not
  transactional, and backup cannot be cancelled), filed from the #128 data-integrity audit
  (`docs/release/audit-data-integrity.md`, findings E-1/E-2/E-3)
- **Base head:** `main @ e4a90c4` (worktree clean at session start)
- **Outcome:** all three findings are dispositioned **within the options the issue itself offers**:
  E-1 is fixed (atomic publish + overwrite confirmation + truthful row count, 6 tests), E-2 takes the
  issue's option (b) (behaviour kept, partial-failure state stated in the failure message, argv shape
  pinned by a test, matrix + registry + release notes updated), E-3 takes the issue's second accepted
  option (no Stop affordance promised; the absence is recorded in the matrix — which already said so —
  and in the release notes). No transactional-restore semantics were changed, because option (a) is a
  compatibility decision the issue leaves open.

## 1. E-1 — export is now atomic, confirmed before overwrite, and truthful about row counts

Before (`crates/ui/src/query_view.rs`): `std::fs::write(path, output)` straight at the destination —
a failure part-way (disk full, I/O error, volume removed) left a **truncated file that looked like a
complete export**, and the failure was only a status-bar string. No overwrite check either.

Now:

| Property | Implementation |
|---|---|
| Atomic publish | `write_file_atomically(&path, contents)` writes a sibling temp file (`<dir>/.<name>.tmp.<pid>.<seq>`, same filesystem), `sync_all`s it, and renames it over the destination — so the destination either keeps its previous bytes or holds the complete new ones, never a prefix |
| No scratch file survives | the temp file is removed on any failure path |
| Overwrite is asked for | a first Export click on an existing path sets `export_overwrite_pending` and the dialog paints "`<path>` already exists. Overwrite it?" with **Overwrite** / **Keep existing file**; nothing is written before the confirmation |
| Row count is truthful | when `result.row_count > rows.len()` (a capped result) the message says `Exported N rows to <path> (M rows matched; the result holds the first N)`, instead of reading as a complete export |
| Failure message | `Export failed: <error> (no file was written)` |

## 2. E-2 — the issue's option (b), implemented

The restore argv was extracted into `PgDumpEngine::restore_command(options, config, password)` so the
**argv shape is testable without a live server** — which is what the issue asks for. The behaviour is
unchanged on purpose: `psql -f <dump>` / `pg_restore <dump>` still run **without**
`--single-transaction` and **without** `--exit-on-error`.

What changed is that the resulting state is no longer implicit. A failed restore now reports:

> `restore failed: <stderr> -- the restore was NOT run in a transaction, so the target database may
> now be partially restored; inspect it before using it (documented in LIM-020). The full
> psql/pg_restore output is above`

Option (a) — `--single-transaction` — is **not** taken: it would make a dump containing
non-transactional statements (`CREATE INDEX CONCURRENTLY`, `VACUUM`, …) fail as a whole, which is a
compatibility change the issue leaves to the owner. It is recorded in the matrix row and in LIM-020
instead of being decided here.

## 3. E-3 — the absence is recorded, not left as a silent third state

`RuntimeCommand::CancelOperation` is declared (`crates/runtime/src/worker.rs:205`) and handled
(`:1563`) and constructed nowhere in the workspace — verified by grep this session, unchanged from the
audit. The issue's acceptance offers two options; the second is taken: **no Stop affordance is
promised**, and the absence is now stated in the release notes and in the registry, on top of the
matrix row that already carried it (`Backup / restore cancellation | NOT SUPPORTED`). Wiring a Stop
control means a UI affordance plus child-process kill plus temp/reserved-file cleanup — real work, and
the issue explicitly allows recording it instead.

## 4. Tests

| Test | Pins | Falsified by |
|---|---|---|
| `atomic_write_publishes_the_whole_file_and_leaves_no_scratch_file` | complete content; directory holds exactly the destination afterwards | — |
| `failed_atomic_write_leaves_the_destination_untouched` | read-only directory → error, destination keeps its **previous** content, no scratch file | probe 1 |
| `failed_atomic_write_into_a_missing_directory_creates_nothing` | nothing created anywhere | — |
| `export_refuses_to_replace_an_existing_file_without_confirmation` | first click asks and writes nothing; confirming writes and closes the dialog | — |
| `export_message_names_a_capped_result_as_capped` | the capped-row message | — |
| `export_replaces_the_destination_instead_of_truncating_it` | the **export path** publishes: the destination's inode changes (rename), rather than being rewritten in place | probe 2 |
| `restore_argv_has_no_transaction_boundary_and_is_documented_as_such` | `psql -f` / `pg_restore` argv has no `--single-transaction` and no `--exit-on-error`, and targets the configured database | probe 3 |

Falsification, all three probes reverted (the tree carries none):

1. `std::fs::write` in place of the atomic helper → `failed_atomic_write_leaves_the_destination_untouched` fails.
2. same revert at the **export path** (`export_result_to_disk`) → **`export_replaces_the_destination_instead_of_truncating_it` fails, and only that one.** Recorded because it caught a real weakness: the first version of this suite tested the helper and left no test on the caller, so reverting the caller stayed green — the same "constant kept, label removed" failure mode the #239 and #242 work had to fix. The inode assertion exists because of that.
3. `--single-transaction` added to the restore argv → `restore_argv_has_no_transaction_boundary_and_is_documented_as_such` fails.

## 5. Gates (raw totals, this host)

| Gate | Command | Result | Baseline | Delta |
|---|---|---|---|---|
| Format | `cargo fmt --all -- --check` | exit **0** | — | — |
| Check | `cargo check --workspace` | exit **0** | — | — |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | exit **0**, no warnings | — | — |
| Tests | `cargo test --workspace` | **911 passed / 0 failed / 27 ignored** | 904 / 0 / 27 | **+7** |
| CI-mirror (fixture up) | `DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture cargo test --all -- --include-ignored` | **938 passed / 0 failed / 0 ignored** | 931 / 0 / 0 | **+7** |
| Release build | `cargo build --release --locked -p db-pro-native` | exit **0** | — | — |
| Perf scan | `bash .skills/perf-audit/scripts/perf-scan.sh` | **PASS (partial)**, exit **0** — 4 executed passed, 0 warnings, 0 failed, 4 not executed | same shape | — |

The +7 is exactly the six export tests plus the restore argv test. The PostgreSQL fixture was started
for the CI-mirroring run and **stopped afterwards**; credentials are redacted. No `qltx-*` container
was touched. The new tests use `std::env::temp_dir()` paths that are created and removed per test, so
no user file is touched.

## 6. Documentation surfaces updated

| File | Change |
|---|---|
| `docs/release/provider-capability-matrix.md` | Restore row: option (b) recorded with the new failure wording and the option (a) trade-off; CSV/TSV export row: **atomic as of #244**, still not cancellable; Backup row and Backup/restore-during-restore cancellation row point at LIM-020 |
| `docs/release/known-limitations.md` | **LIM-020** added (restore not transactional, backup/restore not cancellable — actual behaviour, impact, reason incl. why option (a) is not taken, status, safe release-note wording, must-not-contradict list, evidence); summary 12 → 13 `Accepted v0.1`; no entry deleted or rewritten |
| `docs/release/0.1.0-release-notes.md` | §Known limitations: the restore partial-state bullet, the no-cancellation bullet, and the export-atomicity bullet |

## 7. What this does not claim

- **No transactional restore.** E-2's option (a) is recorded, not implemented; a failed PostgreSQL
  restore can still leave the target partially restored. What changed is that the app says so at the
  moment of failure.
- **No cancellation.** E-3 asks for either a Stop affordance or an explicit record; the record is what
  exists now. `CancelOperation` is still constructed nowhere.
- **No live-server restore test.** The argv shape is pinned without a server, as the issue allows; a
  restore against the fixture was not run in this pass, so the new failure wording is verified by code
  and test, not by a live partial restore.
