# `v01-runtime` — non-GUI runtime evidence for DB Pro v0.1

This tree holds **real, reproducible runtime evidence** collected on 2026-09-14 for the v0.1
release-closure evidence gaps that **do not require driving a native GUI**. It exists because the
previous `V01-01…V01-05` claims were audited as `EVIDENCE_GAP` / `PARTIAL` — those gaps are
runtime-evidence gaps, not build gaps — and because a live-provider PostgreSQL run and a live
SQLite fixture run were the largest remaining holes.

- Session log: [`RUNTIME_SESSION.md`](RUNTIME_SESSION.md) — header, environment, and one row per
  verification case with raw evidence.
- Raw outputs: [`providers/`](providers/) — every command and its unedited output, numbered in
  the order it was produced.
- Screenshots: [`screenshots/README.md`](screenshots/README.md) — **none could be captured**, with
  the reason and the live probe output.

## What this tree proves

| Area | Status | Evidence |
|---|---|---|
| SQLite deterministic fixture (pagination/filter/sort scale) | **Verified** — 60 users / 120 orders / 240 order_items; two independent builds byte-identical | `providers/02`–`05` |
| Live PostgreSQL 16.15 fixture on a disposable container | **Verified** | `providers/06`, `20` |
| The 18 `#[ignore]`d `pg_integration` tests | **Verified — 18 passed / 0 failed** against a live server | `providers/07`–`09` |
| `pg_dump` / `pg_restore` documented dependency | **Verified at CLI level** — complete restore, with a version-skew caveat recorded as `F1`/`P2` | `providers/10`–`14`, `23` |
| PostgreSQL vs SQLite cancellation capability gating | **Verified correct** — PostgreSQL is not offered cancellation anywhere | `providers/15` |
| Packaged-artifact state-directory resolution, all four branches | **Verified** | `providers/17`, `18` |
| Keyring startup stall | **Reproduced and classified** `P2` (mechanism + boundary condition) | `providers/16`, `22` |
| Launch error-log audit | **Verified** — no panics or error classes found; one `P3` cosmetic warning | `providers/19` |

## What this tree explicitly does NOT prove — `PENDING_HUMAN`

**No GUI was driven, observed, or captured. There is no screenshot in this tree and there will
not be one from this environment.** No window was rendered, no control was clicked, no menu was
opened, no visual verdict of any kind is asserted. GUI automation is unavailable to this shell,
re-verified live during this session (`providers/21-gui-unavailability-probes.txt`):

1. `orca computer get-app-state --app com.dbpro.app --json` and
   `orca computer list-windows --app com.dbpro.app --json` both return
   `"code": "runtime_unavailable"` —
   `"Could not read Orca runtime metadata at /Users/truongdev/Library/Application Support/orca/orca-runtime.json. Start the Orca app first."`
2. `osascript -e 'tell application "System Events" to get name of first process'` returns
   `Not authorized to send Apple events to System Events. (-1743)`
3. `screencapture -x <path>` returns `could not create image from display` and creates no file.
4. No accessibility tree is obtainable — the three mechanisms above are the ways to read one, and
   all fail before returning a tree.

Consequently these remain **`PENDING_HUMAN`** and are owned by the coordinator (runbook:
`docs/release/evidence/v01-06/14-install-smoke.txt` §8):

- Window rendering at all; light/dark traversal; 1920×1080 acceptance.
- Settings navigation; creating a SQLite connection through the UI; Test/Save.
- Running `SELECT 1;` / `SELECT * FROM items;` and reading the result grid.
- Interactive row-grid pagination/filter/sort against the SQLite fixture added by this run.
- Clicking Run/Stop and observing the capability-gated "Running…" state.
- The app's own Backup action (this run evidences the `pg_dump`/`pg_restore` *dependency* only).
- A clean window close / ⌘Q exit, and the interactive persistence check.
- Any visual verdict on the icon-font glyph warning `F2`.

## Ground rules this tree was collected under

- No fabricated evidence: no invented screenshots, timings, provider runs, or metrics. Every
  number here came from a command whose raw output is filed alongside it.
- Passwords are redacted everywhere (`postgres://dbpro:<redacted>@…`); no secret value appears in
  this tree, and no keychain secret was read.
- The disposable PostgreSQL container is named `dbpro-v01-pg-fixture` on host port **55432** (chosen
  because 5432 was already taken by an unrelated project's container). It was **stopped** at the end
  of the run and **left in place** for re-runs (`docker start dbpro-v01-pg-fixture`). No other
  project's container or volume was touched, and the repo's own compose `pg_data` volume was never
  created or used.
- Fixtures are addressed at scratch paths under `/tmp`; no developer database was read or written.
  The real state directory `~/Library/Application Support/DB Pro` was observed but **not modified**
  (its `meta.db` inode/size/mtime are identical before and after every launch).

## Numbering of the raw outputs

| File | Contents |
|---|---|
| `02` | SQLite fixture build (version, build, file) |
| `03` | SQLite fixture proof — schema list, row counts, FK check, index list, composite PK |
| `04` | SQLite fixture proof — trigger, view, CHECK constraints, pagination/filter/sort, query plans |
| `05` | SQLite fixture determinism + delete-side trigger |
| `06` | PostgreSQL container, credentials shape, version, fixture baseline |
| `07` | Live `pg_integration` suite (18 ignored tests, `--ignored --test-threads=1`) |
| `08` | Workspace test run with `DATABASE_URL` supplied |
| `09` | PostgreSQL post-test fixture state + exact counts + SSH suite status |
| `10` | Backup/restore commands (`pg_dump`, `pg_restore`, restore target creation) |
| `11` | Restored-database verification (`\dt`, `\dv`, `\ds`, counts, view, enum, trigger) |
| `12` | Corrected object/data comparison + the skew control |
| `13` | Version-matched control + the plain-format (`psql -f`) path |
| `14` | Version-matched control, clean single run |
| `15` | Cancellation capability gating (verdict, `file:line`) |
| `16` | Launch A — keyring stall reproduction + `sample` thread dump |
| `17` | Launches B and C — no-item control and the labelled bypass |
| `18` | Resolution-order branches 1, 2, 4 + `RUST_LOG=info` launch |
| `19` | Error-log audit across all launches |
| `20` | PostgreSQL container shutdown record + docker inventory |
| `21` | GUI unavailability probes (the four blockers, live) |
| `22` | Keyring stall classification |
| `23` | Findings and dispositions (`F1`–`F3`) |
