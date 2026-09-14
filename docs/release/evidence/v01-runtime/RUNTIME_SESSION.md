# `v01-runtime` — session log

Runtime-evidence session for DB Pro v0.1 that collects real, reproducible evidence for the gaps
that do **not** require driving a native GUI. Raw outputs for every case below live in
`providers/`; the file number is cited in each row.

## Session header

| Field | Value |
|---|---|
| Date | 2026-09-14 (UTC `2026-09-14T16:08:27Z`; local `2026-09-14 23:08:27 +07`) |
| Repository | `/Users/truongdev/Documents/projects/db-pro/main` |
| Branch / HEAD | `main` @ `8e1d63fde79be6e495cdaa49338790e2009896cd` (`8e1d63f`), worktree **clean** |
| Host | `Mac-mini-cua-Truong.local` — Apple M4 (`Mac16,10`), **arm64** |
| OS | macOS 26.5.2 (build `25F84`), Darwin kernel 25.5.0 |
| Display situation | **No window-server access — GUI is NOT verifiable from this shell.** No window can be rendered, observed, or captured; no UI element can be queried or clicked. Re-verified live; the four blockers are recorded verbatim in `screenshots/README.md` and `providers/21`. |
| rustc | `rustc 1.95.0 (59807616e 2026-04-14)` — matches the pin in `rust-toolchain.toml` (`channel = "1.95.0"`) |
| cargo | `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| Release artifact | `/tmp/dbpro-ci-smoke/db-pro-v0.1.0-macos-arm64.tar.gz`, **10,045,965 B**, SHA-256 `8141d6b7b7ecd98399dd9c2ca23c345da96df496abe0cd0169f9ab967bf2a83a` — **byte-identical to the value recorded for CI run `34860902181`**, so this is the CI-produced artifact; no local re-packaging was needed. Bundled binary SHA-256 `cf40855baa980cdaba8d738f40d0bd88cde939225b8fb8e1834c7b80dfacb55c` also matches the record. |
| PostgreSQL actually used | **PostgreSQL 16.15** (Debian 16.15-1.pgdg13+2), container `dbpro-v01-pg-fixture`, image `postgres:16`, host port **55432** |
| Host PG client tools | `pg_dump`/`pg_restore`/`psql` **18.4** on `PATH` |
| SQLite version | **3.51.0** (2025-06-12), `sqlite3` CLI |
| SQLite fixture path | repo `fixtures/smoke/sqlite/runtime_fixture.sql` → built to `/tmp/dbpro-v01-runtime/sqlite-runtime.db` |
| Docker | 29.7.2 |
| Agent provider | **NOT VERIFIED** — no live provider call was made; `GROQ_API_KEY` was UNSET in this shell |
| `RUST_LOG` | UNSET for every launch except the dedicated audit launch, which used `RUST_LOG=info` |

Header captured by command: `providers/01-session-header.txt`.

## Verification cases

| ID | Action | Expected | Observed | Status | Evidence |
|---|---|---|---|---|---|
| RT-01 | Build the SQLite fixture into a scratch DB: `sqlite3 /tmp/dbpro-v01-runtime/sqlite-runtime.db < fixtures/smoke/sqlite/runtime_fixture.sql` | Script applies cleanly, exit 0 | exit 0, 81,920-byte DB | **PASS** | `providers/02` |
| RT-02 | List schema objects and row counts | 4 tables, 1 view, 2 triggers, 6 explicit indexes (2 unique); 60 users / 120 orders / 240 order_items / 120 user_roles | Exactly that: `users` 60, `orders` 120, `order_items` 240, `user_roles` 120, `v_order_summary` 120 | **PASS** | `providers/03` |
| RT-03 | FK integrity: `PRAGMA foreign_key_check` | No rows | 0 rows | **PASS** | `providers/03` |
| RT-04 | Composite PK is real and enforced | `user_roles` has `pk=1,2` on `(user_id, role)`; a duplicate is rejected | `PRAGMA table_info` shows pk ordinals 1 and 2; duplicate insert fails `UNIQUE constraint failed: user_roles.user_id, user_roles.role` | **PASS** | `providers/03` |
| RT-05 | CHECK constraints enforced | Invalid rows rejected | `CHECK constraint failed: status IN (…)`; `CHECK constraint failed: quantity > 0` | **PASS** | `providers/04` |
| RT-06 | FK enforced at runtime (`PRAGMA foreign_keys=ON`) | Bad FK rejected | `FOREIGN KEY constraint failed` | **PASS** | `providers/04` |
| RT-07 | Insert trigger maintains `orders.total_cents` | 0 orders with a wrong total | 0 of 120 wrong; sample totals 11028 / 16764 / 14320 | **PASS** | `providers/04` |
| RT-08 | Delete-side trigger | Total decreases by the removed line | 11028 → 9954, = remaining `2 × 4977` (run on a throwaway copy only) | **PASS** | `providers/05` |
| RT-09 | View rollup matches a direct aggregate | 0 rows with a wrong rollup | 0 of 120 | **PASS** | `providers/04` |
| RT-10 | Pagination / filter / sort actually exercise the fixture | Multiple pages; filters return subsets; sorts order correctly | Page 3 of 4 (25/page) returns users 51–60; 30 `paid` orders, top-5 by `total_cents DESC` ordered; FK filter joins correctly | **PASS** | `providers/04` |
| RT-11 | Query plans use the declared indexes | Index scans, not full scans | `COVERING INDEX idx_orders_status_total_cents`, `idx_users_email`, `idx_order_items_order_sku` | **PASS** | `providers/04` |
| RT-12 | Determinism: two independent builds are identical | Identical | `.dump` files identical (sha256 `d83f41b4…`); **`.db` files byte-identical** (sha256 `c26fe941…`) | **PASS** | `providers/05` |
| RT-13 | Start the disposable PostgreSQL 16 container on a free port | Server ready, fixtures loaded, no other container touched | `postgres:16` = **16.15**, up on **55432**, initdb ran `001_schema.sql` + `002_seed.sql`, `003_verify.sql` PASS; the four qltx containers untouched | **PASS** | `providers/06`, `20` |
| RT-14 | DATABASE_URL shape redacted | Password never printed | Recorded as `postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture` | **PASS** | `providers/06` |
| RT-15 | Run the 18 `#[ignore]`d tests against the live server | All pass | **18 passed / 0 failed / 0 ignored**, exit 0, 7.57 s | **PASS** | `providers/07` |
| RT-16 | Workspace test run with `DATABASE_URL` supplied | No regressions | **815 passed / 0 failed / 19 ignored**, exit 0 — the 18 stay `ignored` because the attribute needs an explicit `--ignored`; combined with RT-15 the effective total is **833 passed / 0 failed / 1 ignored** | **PASS** | `providers/08`, `09` |
| RT-17 | Fixture integrity after the test suite | Unchanged, no leftovers | `003_verify.sql` PASS; row and object counts identical to baseline (8 S / 17 i / 10 r / 2 v); no `pg_index_lifecycle` leftovers | **PASS** | `providers/09` |
| RT-18 | SSH/backup runtime suite | Runs only if the nine `DB_PRO_SSH_*` variables exist | All nine UNSET → not run | **BLOCKED** — environment variables unavailable (`DB_PRO_SSH_HOST`, `_PORT`, `_USER`, `_KEY`, `_TARGET_HOST`, `_TARGET_PORT`, `_DATABASE`, `_USERNAME`, `_PASSWORD`) | `providers/09` |
| RT-19 | `pg_dump -Fc` the fixture | Archive written | exit 0, 25,485 B, TOC 80 entries, Dump Version 1.16-0, dumped by 18.4 | **PASS** | `providers/10` |
| RT-20 | `pg_restore` into a new disposable database | Restores | exit **1** — 1 ignored error: `unrecognized configuration parameter "transaction_timeout"` (a PG 17+ GUC; server is 16.15) | **FAIL (reported status) / data PASS** → finding `F1`, `P2` | `providers/10`, `23` |
| RT-21 | Verify the restore target actually received everything | Schema + data present | 10 tables / 2 views / 17 indexes / 8 sequences — identical to source; `orders` 4, `order_items` 6, `categories` 4, `products` 4; per-table **MD5 fingerprints identical** (`orders_fp=3146f5fe…`, `order_items_fp=6fc3e4d1…`, `users_fp=71268aab…`); `\dt` lists all 10 tables; enum, trigger, both functions and a working `active_users` view present; `003_verify.sql` PASS against the restored DB | **PASS** | `providers/11`, `12` |
| RT-22 | Control: version-matched tools (16.15 → 16.15) inside the container | Clean round trip | `pg_dump` exit 0, `pg_restore` exit 0, fingerprints and counts identical, `003_verify.sql` PASS | **PASS** | `providers/13`, `14` |
| RT-23 | Control: 16.15 client reading an 18.4-written archive | — | Fails to read at all: `unsupported version (1.16) in file header` | **OBSERVED** (isolates the skew to tooling) | `providers/12` |
| RT-24 | Control: the plain-format path the app uses (`psql -f`) under the same skew | — | exit **0**, data complete (4 orders / 6 items); the `transaction_timeout` `ERROR` is logged but non-fatal | **OBSERVED** | `providers/13` |
| RT-25 | Stop the container, leave it in place | Stopped, still existing, nothing else touched | `docker stop` exit 0; now `Exited (0)` and still present for `docker start`; qltx containers still up; compose `pg_data` volume never created | **PASS** | `providers/20` |
| RT-26 | PostgreSQL `cancel` capability value | `false` | `crates/core/src/domain/capabilities.rs:132` (`cancel: false`, rationale `:129-131`); pinned by test `:244` | **PASS** | `providers/15` |
| RT-27 | SQLite `cancel` capability value | `true` | `crates/core/src/domain/capabilities.rs:187`; pinned by test `:257` | **PASS** | `providers/15` |
| RT-28 | Is the UI Stop/cancel action capability-gated? | PostgreSQL must not be offered cancellation | **Gated on every shipping path**: button rendering + activation `crates/ui/src/query_view.rs:117-136`, Esc path `crates/ui/src/events.rs:1145-1151`, fail-closed lookup `crates/ui/src/app.rs:927-937`, connector-level `Unsupported` `crates/infrastructure/src/postgres/connector.rs:199-202` with test `:714-722`. The only other cancel-shaped control (`components/sql_editor.rs:55-65`) is gallery-only and constructed with `is_running=false`. | **PASS — correctly gated, no bug** | `providers/15` |
| RT-29 | Launch the packaged artifact with no `DB_PRO_DATA_DIR`, `cwd=/` — branch 3 | Uses `~/Library/Application Support/DB Pro`; no `/.db-pro-data`; nothing written in the bundle | Launch B (fresh fake HOME): created `<HOME>/Library/Application Support/DB Pro/{meta.db 90,112 B, secrets/}`; Launch C (real HOME): used the existing store, `meta.db` inode/size/mtime **unchanged**; `/.db-pro-data` **ABSENT**; bundle tree unchanged (names, sizes, mtimes) | **PASS** | `providers/17` |
| RT-30 | Branch 1 — `DB_PRO_DATA_DIR` override | Override wins | `meta.db` created in the override dir; platform dir **not** created | **PASS** | `providers/18` |
| RT-31 | Branch 2 — existing `<cwd>/.db-pro-data` | Legacy dir wins over platform dir | `meta.db` created in the legacy dir; platform dir **not** created | **PASS** | `providers/18` |
| RT-32 | Branch 4 — platform dir unresolvable (`HOME` empty) | Falls back to `<cwd>/.db-pro-data` | Created in the test `cwd` (under `/tmp`); `/.db-pro-data` still absent | **PASS** | `providers/18` |
| RT-33 | Keyring stall, no `GROQ_API_KEY`, real HOME | Reproduce or refute | **Reproduced.** Alive at 12 s, **4762/4762 samples** in one stack: `main` → `keyring::Entry::get_password` → `SecKeychainFindGenericPassword` → `CSSM_DecryptDataFinal` → `SecurityServer::ClientSession::decrypt` → `mach_msg`; `meta.db` unchanged; 0 bytes of output | **PASS (reproduced)** — classified `P2` | `providers/16`, `22` |
| RT-34 | Control: no keychain item reachable (fake HOME) | Should not stall | Passed the keyring immediately; reached `-[NSApplication run]`; created its state dir | **PASS** | `providers/17` |
| RT-35 | Bypass launch with a placeholder key (explicitly labelled a bypass) | Reaches the event loop | Reached `-[NSApplication run]`; state store reused, `meta.db` unchanged | **PASS (bypassed — not a clean launch)** | `providers/17` |
| RT-36 | Error-log audit over all launches | No panic/error classes | No `panic`, no unwrap-on-`Err`, no channel-disconnected, stale-event, database-lock, keyring, worker-failure or unhandled-provider errors. One `P3` cosmetic `epaint` glyph warning ×13, only with `RUST_LOG=info` | **PASS** (1 `P3` recorded) | `providers/19` |
| RT-37 | GUI availability | Available? | **Unavailable** — `runtime_unavailable` (Orca), `-1743` TCC denial (AppleScript), `could not create image from display` (`screencapture`), no accessibility tree | **BLOCKED** — causes every GUI item to be `PENDING_HUMAN` | `providers/21`, `screenshots/README.md` |
| RT-38 | Agent live provider run | — | Not attempted: no `GROQ_API_KEY` and no provider credentials were available; making one would require a real key | **BLOCKED / NOT VERIFIED** | `providers/01` |

## Case that was deliberately NOT run

- **PostgreSQL query cancellation was not implemented, and no code was added for it.** The brief
  states the current model (`postgres.cancel = false`, `sqlite.cancel = true`) is correct as-is;
  RT-26…RT-28 verify that model rather than change it.

## Code changes in this session

**None.** No production code, workflow, release script, manifest or toolchain pin was modified.
Only two additive, non-release-infrastructure files were added to the repo plus this evidence tree:

- `fixtures/smoke/sqlite/runtime_fixture.sql` (new fixture; the pre-existing `smoke_fixture.sql`
  and `verify-smoke.sh` are untouched).
- `docs/release/evidence/v01-runtime/**` (this tree).

Because no production code changed, the six gates were **not** re-run and no gate number is
re-minted here. The authoritative gate baseline remains the recorded post-`543b526`
**815 passed / 0 failed / 19 ignored**; this run's contribution is the live-fixture count from
RT-15/RT-16 (**18 passed / 0 failed** on `pg_integration`, giving an effective 833/0/1 with the
live server supplied, the single remaining ignored test being the SSH one).

## Findings summary

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| `F1` | `P2` | `pg_restore` exits 1 under PG 18.4-client / 16.15-server skew after a **complete** restore (`SET transaction_timeout = 0`); the app reports `restore failed` on any nonzero exit (`pg_dump.rs:167-170`) | Recorded, not chased — full analysis and controls in `providers/23` |
| `F2` | `P3` | `epaint` font-atlas glyph fallback warning (`◻`, U+25FB) — cosmetic, only with `RUST_LOG` set | Recorded, not chased |
| `F3` | `P2` | Keyring stall: unbounded Keychain read before the data directory is resolved; needs a pre-existing item + a context where the prompt cannot be shown | Recorded, not chased; already `R-KEYRING-STALL` / `LIM-018`; escalation condition stated in `providers/22` |

**No `P0` or `P1` was found**, so nothing was fixed and no regression test was added. Full
classification: `providers/23-findings-and-dispositions.md`.

## What could not be verified here, and why

1. **Any GUI behaviour** — no window-server access (`providers/21`). Listed item by item in
   `README.md` under `PENDING_HUMAN`.
2. **Screenshots** — none taken, none possible; `screenshots/README.md` records why.
3. **The SSH backup/tunnel runtime suite** — the nine `DB_PRO_SSH_*` variables are unset.
4. **Agent live-provider behaviour** — no provider key; `GROQ_API_KEY` unset and no live call made.
5. **The app's own Backup/Restore action** — only the `pg_dump`/`pg_restore` *dependency* was
   verified, at CLI level, on this host. That is a CLI-level verification, **not** a GUI backup test.
6. **SQLite cancellation interactively** — the gating is verified in code and tests, but no query
   was cancelled by clicking or keying anything.
7. **Windows/Linux runtime** — no such host exists; unchanged from the recorded state.
8. **Whether the `F2` glyph warning is visible on screen** — needs a window server.
