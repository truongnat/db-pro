# Open-issue workstream — handover

**Status:** phase 2 complete as far as this environment allows; the remaining work is owner decisions and
host-blocked items. **Branch:** `main` only. **Audited at:** `13fde57`.
**Truth lives in:** [`LEDGER.md`](LEDGER.md) (what happened, per issue), [`INVENTORY.md`](INVENTORY.md)
(one disposition per issue), [`PLAN.md`](PLAN.md) (batches and order), [`METHOD.md`](METHOD.md) (how each
classification was derived, with the commands). This file is the entry point to those four; it does not
replace them. Every number below was re-verified against GitHub and git on 2026-09-15, not copied from prose.

## 1. Scope and method (recap)

The workstream triaged **every open issue** in `truongnat/db-pro` against the **live tree** (not against
the issue text), then worked the actionable remainder directly on `main`:

- one commit per issue, subject ends with `(#NN)`, pushed to `origin main`;
- each issue closed with an evidence comment (command + result, or the evidence file path), or left open
  with a blocker comment naming the blocker, what is already done, and what would unblock it;
- `LEDGER.md` updated in the same push (the row, the queue table, and — for closures — the
  "Closed by this workstream" section + a Revisions line).

**No branch, PR, tag or force-push was created by this workstream.** Work already claimed on another
branch (PR #245, issue #238) was verified and reported, never duplicated or merged. `gh` writes were
limited to closing issues and posting the comments above; nothing was relabelled, re-milestoned or edited.

## 2. Final counts (verified 2026-09-15)

```bash
gh issue list --state open   --limit 400 --json number -q length                              # 117
gh issue list --state closed --limit 400 --json number -q length                              # 83
gh issue list --state closed --limit 400 --search "closed:>=2026-09-14T16:45:00Z" --json number -q length  # 47
```

| Measure | Value | Check |
|---|---:|---|
| Open issues | **117** | live |
| Closed issues | **83** | live |
| Closed since `2026-09-14T16:45:00Z` | **47** | all 47 closed between `17:30Z` and `23:53Z` on 2026-09-14; the previous closure is `2026-08-17T11:18Z` (#51), so the window isolates this workstream exactly |
| Closed by this workstream (ledger) | **47** | the 47 issues below; each verified `CLOSED` by `gh issue view <n> --json state` |
| Ledger open rows | **117** | 114 open rows in the triage table (incl. the post-triage #238 row) + the three filed-and-left-open follow-ups #242/#243/#244 recorded in "Closed by this workstream" |
| **Invariant: open ledger rows = live open issues** | **117 = 117** | holds |

Composition of the 164 ledger rows (155 snapshot rows + the nine post-triage arrivals #236–#244): 47 closed,
117 open. Data-hygiene note: the `#72` line in `LEDGER.md` carries its pre-closure `ACTIONABLE_NOW` row and
its `CLOSED` row **concatenated on one physical line** (a missing newline). It is recorded as closed and
counts once; fix it if you touch that row.

**Session start, for contrast** — the workstream began from the frozen triage input, not from 117:

| Point in time | Open issues | Closed by this workstream | Source |
|---|---:|---:|---|
| Frozen snapshot 1 (`issues-open-2026-09-14.json`) | 136 | 0 | `README.md`, `METHOD.md` §4b |
| First live re-count during triage (snapshots 1+2) | 147 | 0 | `METHOD.md` §2.1 |
| Session universe after snapshot 3 added #228–#235 | **155** | 0 | `README.md` — the ledger's 155 snapshot rows |
| Mid-session 2026-09-15 re-count (after the first 11 closures) | 144 | 11 | `README.md` — 155 rows = 144 open + 11 closed; **not** the session start |

Exact reconciliation to today's 117: **164 ledger rows** (155 snapshot rows + the nine post-triage arrivals
#236–#244) **− 47 closed = 117 open**. Split by origin: 155 snapshot rows → 42 closed, 113 open; nine
arrivals → 5 closed (#236, #237, #239, #240, #241), 4 open (#238, #242, #243, #244).

## 3. What was delivered — the 47 closed issues

Pull from `LEDGER.md` "Closed by this workstream" plus the pass-2/3/4 queue tables and the Revisions log
(the section holds 35 rows: 32 of these 47 plus the three follow-ups #242/#243/#244 that were filed and
left open). Grouped by theme; every sha is on `main` and was re-read with `git show -s`.

**A. Security / data-safety / user-visible correctness fixes (9)**

| # | Outcome | Commit |
|---:|---|---|
| #142 | Encrypted-file secret fallback kept out of release builds; store built via `build_secret_store` with the fallback gated | `78fb39b` |
| #236 | Editing or duplicating a connection no longer silently downgrades its stored TLS mode (filed from the #144 verification) | `aba947b` |
| #237 | Schema refresh survives concurrent DDL (tolerant decode of `pg_get_indexdef`/`pg_get_constraintdef` + bounded retry on narrow transient catalog errors) | `0d3aa84` |
| #56 | Temporal decoder no longer invents a timezone (`TIMESTAMP` without tz) | `f7c61c9` |
| #58 | Unsupported value classes decode without corrupting the row (format- and `PgTypeKind`-aware fallback) | `c7f4328` |
| #61 | Copied and exported values serialized losslessly through one shared serializer | `205539c` |
| #62 | Type-aware editability policy enforced before staging, at every edit entry point | `0af2647` |
| #129 | A `Destructive` statement (or script whose worst statement is destructive) is held for confirmation of the exact text | `458ec72` |
| #239 | The SSH card discloses the LIM-006 qualification in-app (filed from the #77 audit) | `ccc0a24` |

**B. Gate 5 provider-value work and v0.1 evidence closures (21)**

| # | Outcome | Commit |
|---:|---|---|
| #55 | Exact numeric/integer decoder paths re-verified on the tree (9 mapper tests green) | — (pre-existing; no new commit) |
| #74 | Post-P1 audit baseline and 25-finding inventory frozen (`v01-06/06-rc1-p2-dispositions.md`) | — (pre-existing) |
| #83 | Candidate SHA and release-evidence input locked (`85a7fa3`, run `34860902181`) | — (pre-existing) |
| #85 | Complete workspace release gates recorded for the candidate (815/0/19 at that time) | — (pre-existing) |
| #86 | PostgreSQL + SQLite integration suites on the candidate (live 18/0, SQLite 32/0) | — (pre-existing) |
| #87 | Release artifact matrix built from the exact candidate SHA | — (pre-existing) |
| #90 | Packaged-runtime smoke fixtures, credentials and evidence worksheet prepared (preparation only) | — (pre-existing) |
| #132 | PostgreSQL vs SQLite capability/support matrix corrected and pointer-verified | — (pre-existing) |
| #133 | Deterministic packaged-smoke fixture datasets (incl. 250-table large-ER fixture) | — (pre-existing) |
| #135 | Canonical known-limitations registry (18 `LIM-*` entries) | — (pre-existing) |
| #88 | Candidate evidence manifest + 9 contract tests tie gates, artifacts and checksums to one SHA | `326c313` |
| #64 | Live provider → domain → DTO end-to-end test over the 26-column fixture row | `483f3da` |
| #66 | Exact-head Gate 5 verification record (six gates + live matrix + CI-mirror on one SHA) | `06e137e` |
| #23 | Workstream-B umbrella closed as `SATISFIED_NOW` (children #55–#59 carried the work) | — (no code change) |
| #22 | Workstream-A umbrella closed after re-verifying A1–A4 on the live tree | `89afeec` (docs) |
| #52 | Dedicated `timestamp`/`timestamptz`/`timetz` variants across domain, decoder, DTO, UI, export, binding | `27c52db` |
| #54 | Whole-result provider-value DTO contract pinned byte-for-byte to a generated fixture | `266e917` |
| #57 | Structured and binary value-class decoding asserted live (UUID/JSON/JSONB/BYTEA/CIDR) | `a153c1e` |
| #59 | 26-column decoder-matrix fixture and its live test | `75865db` |
| #53 | A3 fallback contract recorded in `docs/release/provider-value-contract.md` | `dbfe1e1` |
| #72 | Introspection IPC serialization contract pinned by four tests + a checked-in fixture | `a059506` |

**C. P2 audit dispositions (11)** — one disposition per live row in `docs/release/rc1-p2-release-dispositions.md` §A–§F; `Fix RC1` rows became their own issues.

| # | Outcome | Commit |
|---:|---|---|
| #75 | §A Workspace/Shell: 7 rows accepted, 1 deferred; no fix spawned | `84f21c0` |
| #76 | §B Data Grid: 5 rows accepted/deferred; one `Fix RC1` → **#238** filed | `744d91f` |
| #77 | §C Connection/session/SSH: 6 rows settled; one `Fix RC1` → **#239** filed | `e764f9b` |
| #78 | §D Query/ER: 4 rows settled; no fix spawned | `522f173` |
| #79 | §E Perf tooling: 8 rows; three `Fix RC1` fixed in place under **#240** | `9e8c0f6` |
| #240 | perf-scan no longer certifies a partial run, names skipped sections, exits 2 on `WARN`, prints provenance | `9e8c0f6` |
| #80 | §F Cross-cutting polish: 9 bullets; six defects fixed under **#241** | `ebf085b` |
| #241 | Feedback, false DDL affordance, icon-button names, `Shift+F10` menus, shortcut labels, theme drift | `ebf085b` |
| #122 | Trust-boundary audit (`audit-security-boundaries.md`): no P0/P1; one P2 → **#242** filed | `6446d94` |
| #128 | Data-integrity audit (per-operation classification) + capability-matrix corrections; → **#244** filed | `6f75477` |
| #131 | 25-row capability → tests → CI → smoke → owner matrix (`audit-release-traceability.md`) | `4eae4ad` |

**D. Brand / traceability / process documents (6)**

| # | Outcome | Commit |
|---:|---|---|
| #102 | Brand rename surfaces and persisted identifiers inventoried, classified with file:line | `f5d94ff` |
| #121 | Same inventory closed for the second brand issue | `f5d94ff` |
| #120 | Source-backed differentiator/parity inventory + can-say/cannot-say table for #97 | `391091b` |
| #114 | Agent execution evidence contract (`docs/plans/_template/AGENT_EVIDENCE.md` + an `AGENTS.md` section) | `81f5bc9` |
| #127 | Versioning/updater/persisted-state audit; absent updater recorded as absent | `39fe37e` |
| #134 | Platform prerequisites reconciled with the native packaged runtime | `dd984bd` |

**Filed by this workstream and closed in the same pass:** #236 (`aba947b`), #237 (`0d3aa84`), #239
(`ccc0a24`), #240 (`9e8c0f6`), #241 (`ebf085b`) — all five appear in the theme tables above and were never
open rows at any snapshot.

## 4. New issues filed by this workstream and still open

| # | One-line problem | Why it is open |
|---:|---|---|
| **#242** | The AI features are the app's only egress (automatic prediction with `Eager` default, keyring-seeded API key, sample rows in agent payloads) and nothing in the product discloses it. | Filed from the #122 audit, whose rule is audit-only: proven gaps become focused child issues rather than in-place behaviour patches. Needs in-app + release-note disclosure and an owner decision on the prediction default. |
| **#243** | The secret store reports success when nothing was stored, orphan secrets are invisible, and the agent API key cannot be deleted from the app (findings K-1/K-2/K-3/K-5). | Filed from the #126 audit under the same inspection-only rule; carries its own acceptance and the file:line evidence. |
| **#244** | Export is not atomic, PostgreSQL restore is not transactional, and backup cannot be cancelled (E-1/E-2/E-3). | Filed from the #128 audit (inspection-only); each finding carries the pointer, the failure mode and its own acceptance (`temp-file + rename`, `--single-transaction` or explicit partial-state disclosure, wire or record cancellation). |
| **#238** | The result-grid sort rebuilds an allocating projection on every frame (measured: 200k rows × 4 columns → 6,649.6 ms for a temporal-text sort, 5.29 ms unsorted). | **Claimed by open PR #245** (`a0452a32`, branch `fix/rc1-p2-result-grid-sort-perf-…`), whose head is **not** an ancestor of `main` and which covers only the issue's option (b) — not option (a) or acceptance items 1 and 3. The defect was re-measured on `main` with a temporary probe (removed before the commit) and reported; the fix is another session's to land. |

## 5. Open queue — every class and why it is still open

Counts are of open ledger rows (114 in the triage table + #242/#243/#244 = 117, matching §2).

| Class | Count | Why still open | Issues |
|---|---:|---|---|
| `OUT_OF_SCOPE_V01` | 58 | Explicitly post-v0.1 scope (v0.2–v0.5 backlog tiers, Phase A–H, the post-v0.1 Goal). Not v0.1 gaps; do not start. | #32–#35, #182–#216, #217–#227, #228–#235 |
| `BLOCKED_OWNER` | 23 | Re-verified on the tree, then left open with one blocker comment each naming the blocker, the done part and the unblocking step. Product/governance call. | #14, #21, #24, #25, #26, #27, #28, #30, #31, #67, #69, #70, #73, #81, #82, #89, #96, #105, #106, #107, #110, #136, #141 |
| `NEEDS_OWNER_DECISION` | 14 | Needs a licensing/naming/release-scope/protection decision by the owner; recorded in `0.1.0-human-decisions.md` and `risk-register.md` first. | #68, #71, #97, #98, #99, #101, #103, #104, #108, #109, #111, #119, #144, #146 |
| `SUPERSEDED` | 7 | The target layer was retired; **awaiting the owner's confirmation before closure** (do not close them on your own initiative). | see below |
| `BLOCKED_EXTERNAL` | 5 | Needs a Windows/Linux host, an interactive GUI session, or a publication this environment does not have; audits that are done are named in each blocker comment. | #29, #91, #94, #126, #145 |
| `NEEDS_EXTERNAL_RESOURCE` | 5 | Needs a resource the environment lacks (Windows/Linux hosts, live web diligence, GitHub Projects v2 mutation). | #92, #93, #95, #100, #112 |
| `ACTIONABLE` (remaining) | 2 | #238 is claimed by PR #245 (not on `main`); #147 is reclassified onto a **measured correctness gap** plus a contract choice that is the owner's. | #238, #147 |

**The `SUPERSEDED` seven** (retired-frontend/React-era targets; `INVENTORY.md` names each replacement):

| # | Named replacement |
|---:|---|
| #60 | Native UI value types (`crates/ui/src/runtime.rs:541-549`, `crates/native-app/src/translate.rs:649-668`); alignment tracked by #61/#62 |
| #63 | No native value-policy matrix test set exists — it would be a new task; nearest coverage `crates/ui/src/app_tests.rs:219-263` |
| #65 | Native in-process translate + UI tests (`crates/native-app/src/translate.rs`, `crates/ui/src/app_tests.rs`); no serde JSON path |
| #84 | `.github/workflows/ci.yml` job "Rust checks" + the release workflow "Pre-flight checks" job |
| #113 | `docs/plans/STATUS.md` + `docs/plans/FEATURE_LIFECYCLE.md` + `AGENTS.md` |
| #115 | Native ER test fixtures/invariants in `crates/ui/src/diagram/tests.rs` (Gate 4, #47/#48) |
| #116 | Native diagram tests asserting worker/layout behaviour (`crates/ui/src/diagram/tests.rs:166-260`, `:333-360`) |

## 6. Owner decisions blocking progress

Each row is the **owner's choice**; "recommended" is only stated where the evidence in the tree supports one.

| Issue(s) | Decision needed | What it unblocks | Evidence / recommendation |
|---|---|---|---|
| **#147** | What a multi-statement batch may contain. Measured: `execute_transaction(["BEGIN","INSERT …","COMMIT","SELECT no_such_column …"])` returns `outcome=RolledBack` **with 1 row surviving**, against a control that returns `RolledBack` with **0 rows** — the failure envelope claims a rollback that did not happen. | Closing #147; the #136 register update | `providers/51-multistatement-contract-criterion-audit.md` §2–§3, `docs/release/audit-execution-safety.md` §3. **Recommended: reject** a batch containing transaction control, with a precise message (option 1 — the only one that fails closed and keeps the `docs/09` §5 contract true; consistent with manual transaction controls being `OUT_OF_SCOPE_V01`). Alternatives: honour it, or weaken the atomicity claim. |
| **#144** | The PostgreSQL TLS default, and whether CA/client-cert input ships in v0.1 | #144, indirectly #136 | `providers/27-pg-secure-default.md`; ledger row `#144`. **No recommendation the evidence can support**: no available variant is both non-breaking and non-plaintext (`Require` breaks compatibility and verifies nothing without a root CA; `VerifyCa`/`VerifyFull` are unsatisfiable with no CA fields; `Prefer` needs a new domain/DTO/UI option the issue forbids inventing). Four sub-choices: which default, host-aware or not, CA input in v0.1, and whether the `Require` break is acceptable. |
| **#119** | Licence (`HD-001`) — no `LICENSE` file or manifest key exists anywhere | Public distribution (`R-LICENSE` `BLOCKING`); #108/#109/#111 | `0.1.0-human-decisions.md` HD-001. **Recommendation deliberately withheld** (legal consequences); holding pattern if needed: all rights reserved, no public distribution. |
| **#30**, **#101** | The public name and controlled rename (`LIM-001`) | #30/#101, #103 implementation checklist, release notes | `known-limitations.md:35-47`; the rename surfaces are inventoried (`f5d94ff`, #102/#121) so the decision is the only thing missing. Owner's call. |
| **#108**, **#109**, **#111** | Tag sequence and timing (`HD-008`): `v0.1.0-rc.1` → smoke → `v0.1.0`, or tag `v0.1.0` directly | The tag, the GitHub Release, asset attachment, post-publish verification | `0.1.0-human-decisions.md` HD-008; `git tag -l` is still empty. **Recommended: (a) two-step** (recorded in `0.1.0-readiness.md` + `0.1.0-handoff.md` §8). Gated in practice by HD-001. |
| **#81**, **#82**, **#27** | Whether the P2 disposition table is final (`Fix`/`Accept`/`Defer`), and the verification of it on an exact post-remediation SHA | The P2 programme gate (#27 umbrella, #81 synthesis, #82 verification) | `docs/release/rc1-p2-release-dispositions.md` §A–§F (all six modules dispositioned); `STATUS.md:88` records the gate checkbox as unchecked → "the P2 gate is formally unsatisfied"; `R-RC1-P2` = `DEFERRED + ACCEPTED + FIXED`. All four `Fix RC1` rows were spawned and three are closed (#239, #240, #241) with #238 claimed by PR #245. No recommendation recorded — the acceptance is the owner's. |
| **#126** | Whether a **persistent** Linux keyring backend is required | #126's closure (audit is complete) | `docs/release/audit-keyring-secret-lifecycle.md`; `R-KEYRING-STALL` `OPEN`/`ACCEPTED`, Linux is not release-qualified (#93, `R-WINLINUX`). The packaged per-platform proof is blocked externally (#92/#93, #91); the decision is the owner's. |

## 7. Verification baseline to inherit

| Gate | Command | Recorded result |
|---|---|---|
| 1 | `cargo fmt --all -- --check` | exit 0, no output |
| 2 | `cargo check --workspace` | exit 0 |
| 3 | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no warnings |
| 4 | `cargo test --workspace` | **886 passed / 0 failed / 27 ignored** at `13fde57` (re-run on the current tree while writing this handover: 20 suites, 0 failures) |
| 5 | `cargo build --release --locked -p db-pro-native` | exit 0 (23,777,216 bytes at `ccc0a24`) |
| 6 | `bash .skills/perf-audit/scripts/perf-scan.sh` | exit 0 — reports `PASS (partial)` when sections are skipped, naming them (exit contract `PASS` 0 / `WARN` 2 / `FAIL` 1) |
| CI-mirror | `DATABASE_URL=postgres://dbpro:<pw>@127.0.0.1:55432/dbpro_fixture cargo test --all -- --include-ignored` | **913 passed / 0 failed / 0 ignored** (recorded at the #239 commit; baseline 910) |

- **The 886/0/27 line is still the code baseline at `13fde57`:** the four commits after `ccc0a24`
  (`3889970`, `31ed252`, `8cd1cd4`, `13fde57`) are documentation-only — `git diff --stat ccc0a24 13fde57 -- crates/`
  is empty. Later doc-only commits do not change the gate line. The 27 ignored tests are the live-database
  tests behind `DATABASE_URL`; gate 4 alone does not need the fixture.
- **PostgreSQL fixture** (only for the CI-mirror run and the live suites): container `dbpro-v01-pg-fixture`,
  image `postgres:16` (16.15), host port **55432**, user/db `dbpro`/`dbpro_fixture`.
  Start with `docker start dbpro-v01-pg-fixture` (it is stopped and left in place); if it has to be
  recreated, follow §0.3 of `docs/release/0.1.0-interactive-verification-runbook.md` — **the password is the
  one in that runbook**. Stop it with `docker stop dbpro-v01-pg-fixture` when done.
  **Never touch the `qltx-*` containers**: port 5432 belongs to that unrelated project, which is why the
  fixture lives on 55432. (The `#239` run started and stopped the fixture and left the tree clean.)
- **perf-scan**: the script is part of the shipped repo under `.skills/perf-audit/`; `perf-scan.sh --self-test`
  (14 assertions) is its regression test.

## 8. How to continue — the exact procedure for a new issue

1. **Read the ledger row** in `LEDGER.md` and its disposition in `INVENTORY.md` (the triage pointer is a
   starting hint, never evidence). Check `PLAN.md` for the batch it belongs to.
2. **Re-verify the remainder on the current tree**: open the cited `file:line`/test/document and confirm it
   still says what the row claims. A similar-sounding document is not enough; body text, milestones and age
   are never evidence (`METHOD.md` §1).
3. **Implement with a reproduction first** — a failing test/probe that is shown to fail before the change and
   pass after it. Remove the probe before committing unless it is the regression test.
4. **Run the six gates plus the CI-mirroring run** (§7), with the fixture started for the mirror run and
   stopped afterwards.
5. **Commit referencing the issue** — `type(scope): … (#NN)`, one issue per commit — and **push to
   `origin main`**.
6. **Update `LEDGER.md` in the same commit** (row status/commit/evidence, the queue table, the Revisions
   line; move the row to "Closed by this workstream" if it closed) and `INVENTORY.md` too if the disposition
   changed. If a sha or a comment URL cannot be known before the push, record it in a follow-up ledger-only
   commit — and **re-read the file after pushing** to confirm it actually landed. A doc-only commit once
   shipped a note claiming URLs it had not written (`8cd1cd4`, corrected in `13fde57`); the escape was
   caught by re-reading the file, not by trusting the commit message.
7. **Close with an evidence comment** (command + result, or the evidence path), or **leave open with a
   blocker comment** naming the blocker, what is already done and what would unblock it. Close nothing on a
   partly-met gate.

**Ledger invariant (non-negotiable):** the number of open rows in `LEDGER.md` must equal the live open-issue
count (`gh issue list --state open --limit 400 --json number -q length`). Dispositions are never silently
rewritten: a correction is added to the Revisions table and the old row is kept. Both frozen snapshots stay
in place; a new triage adds a new snapshot file.

**Evidence files** live in `docs/release/evidence/v01-runtime/providers/` (numbered; **the next free number
is 52** — 51 is `51-multistatement-contract-criterion-audit.md`). Release-facing audits live directly under
`docs/release/` (`audit-*.md`, `rc1-p2-release-dispositions.md`). Release-truth registries that must stay
consistent with any change: `docs/release/known-limitations.md`, `risk-register.md`,
`provider-capability-matrix.md`, `0.1.0-human-decisions.md`.

## 9. Residual risks (honest list)

| Risk | State | What it means for the next engineer |
|---|---|---|
| **`R-GUI-SMOKE`** | P1, OPEN, `ACCEPTED` for the internal RC / `BLOCKING` for interactive claims | No window has ever been rendered or interacted with on a packaged build; all **165** manual-smoke rows are `blocked`. The runbook (`0.1.0-interactive-verification-runbook.md`, ~30–60 min) converts them into evidence — `HD-007` is the owner's call on how much of it is the gate. |
| **`R-WINLINUX`** | P1, OPEN; build+package verified in CI run `34860902181` | Both platforms compile and package green; **no process has ever been launched from either archive**. Never make cross-platform *runtime* claims. Linux credentials additionally need a D-Bus Secret Service. |
| **Unsigned artifacts** | `R003` P2, `ACCEPTED` with disclosure (`HD-002` PENDING) | macOS: `Signature=adhoc`, `spctl -a -vvv -t execute` **rejects**; Windows SmartScreen warns; Linux artifacts unsigned. Signing later means a new candidate SHA and a new CI run. |
| **`R-KEYRING-STALL`** | P2, OPEN/`ACCEPTED` (`LIM-018`) | An unbounded keychain read sits on the startup path: an unattended first launch can stall on the credential prompt. Untouched by the #142 fallback fix. |
| **`F1` / `HD-006`** | P2, recorded; disposition PENDING | Under PostgreSQL client 18.4 / server 16.15 skew, `pg_restore` exits 1 (`SET transaction_timeout = 0`, a PG17+ GUC) **after a complete restore**, and the app reports "restore failed" on any nonzero exit (`pg_dump.rs:167-170`). Evidence `providers/23-findings-and-dispositions.md` §F1. Recommended: docs-only for 0.1.0. |
| **Caution — commit messages are not evidence** | — | `8cd1cd4` shipped a ledger note claiming two comment URLs it had not written (the edit script asserted on a stale anchor and exited early); it was corrected in `13fde57`. Trust the file content, and re-read after pushing. |

**Not residual:** the packaging/build pipeline and the three platform archives (`34860902181`, candidate
`85a7fa3`) and the secret-store fallback wiring (`R-KEYRING-FALLBACK`, `FIXED` in `78fb39b`).
