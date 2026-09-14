# V01-01 → V01-05 Evidence Consistency Audit

- Audited at: HEAD `3e0b0775c2248d37253d7325eddebd952e26d101`, branch `main`
- Evidence commit under audit: `53e89f5` — *test(release): record live native runtime verification for V01-01 through V01-05*
- Method: read the evidence files the closure plan cites, plus `git show 53e89f5`; re-ran the commands needed to test each factual claim.
- This audit **changes no code and no claims**. Nothing was fixed, softened, or removed.

## Epistemic labels used

- `OBSERVED` — directly seen in a command I ran or in a file I read, with the path/command given.
- `CANDIDATE` — the mechanism is plausible and consistent with source, but not reproduced here.
- `NOT OBSERVABLE / NOT VERIFIED` — cannot be established with the tools available in this environment.
- `EVIDENCE_GAP` — the claim is **not** backed by the evidence the repo actually contains.

## Allowed verdicts

`EVIDENCED` / `PARTIAL` / `EVIDENCE_GAP`

---

## 0. Section 0 preconditions (goal-3 §0)

| Precondition | Verdict | Evidence |
|---|---|---|
| HEAD contains V01-01..05 runtime evidence | `EVIDENCED` (text only) | `docs/notes/V0_1_CLOSURE_PLAN.md:100,116,135,153,171` each read `- **Current State**: \`PASS / VERIFIED\` (2026-09-14).` (via `git show HEAD:...`). The six cited evidence files all exist at HEAD. |
| HEAD contains the full product goals | `EVIDENCED` | `git ls-tree -r --name-only HEAD -- docs/goals/` returns all 9 files (`goal-full-product.md`, `goal-phase-a-object-crud.md` … `goal-phase-h-ai.md`). |
| No uncommitted Phase A production code | `EVIDENCED` | `git status --porcelain -uall` filtered for everything except `goal-3.md` and this run's own `docs/release/evidence/` returns **nothing**. `grep -rn "IMPLEMENTING" docs/goals/` finds no Phase A `IMPLEMENTING` state. |

**Precondition note (not a blocker):** `docs/goals/goal-full-product.md:127` still reads *"Native Visual Redesign itself is still `IMPLEMENTING`"*, which contradicts `V0_1_CLOSURE_PLAN.md:100`. Per goal-3 §34 this file must not be modified in this run; recorded here as a consistency contradiction only.

---

## 1. The structural problem affecting every claim below

`53e89f5` did **not** add any new evidence artifact. Its full diff is:

```
 crates/infrastructure/tests/pg_integration.rs      |  2 +-      (CellValue::Text -> CellValue::Decimal)
 docs/notes/V0_1_CLOSURE_PLAN.md                    | 78 +++---   (checkbox [ ] -> [x], state -> PASS)
 docs/plans/active/er-hardening-verification/VERIFICATION.md      | 25 ++---  (new PASS markdown table)
 docs/plans/active/native-visual-redesign/VERIFICATION.md         | 18 +++++  (new PASS markdown table)
 docs/plans/active/query-editor-intelligence/VERIFICATION.md      | 18 +++++  (new PASS markdown table)
 docs/plans/active/rc1-full-product-qa/VERIFICATION.md            | 33 ++---   (Status column: PASS)
 docs/plans/active/schema-regression/VERIFICATION.md              | 10 +--    (PG table: 9 -> 18 tests, PASS)
 docs/release/0.1.0-manual-smoke.md                               | 17 ++---   (sign-off block filled in)
 fixtures/smoke/postgres/002_smoke_seed.sql                       |  4 +-     (X'..' -> '\x..' bytea syntax)
```

So the commit is **documentation + a 1-line test assertion change + a fixture syntax fix**. Everything asserted about runtime behaviour in this commit is prose. No screenshot, log, capture, timing dump, or reviewer artifact was added.

That is not fatal by itself — the closure plan points at evidence that may live in *earlier* commits. The rest of this document checks whether such evidence exists.

### 1.1 Screenshot evidence is gone (affects V01-01 and V01-03)

The native-visual-redesign evidence trail cites 50 distinct macOS temp-directory captures, e.g.
`/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/<uuid>-screenshot.png`.

```
$ grep -rhoE 'orca-computer-use/[0-9a-f-]{36}' docs/ | sort -u | wc -l
50
$ ls /var/folders/bh/.../T/orca-computer-use/ | wc -l
22
```

I checked every one of the 50 cited UUIDs against the live directory:

```
PRESENT=0  MISSING=50
```

`OBSERVED` — **0 of 50 cited screenshots still exist.** The surviving 22 files have unrelated UUIDs, are dated 2026-09-12/13 (before the 2026-09-14 evidence commit), and are referenced by no document. The directory carries a `.last-cleanup` marker; macOS periodically purges `T/`. This path is **not durable evidence** — it is an ephemeral scratch directory.

The only committed image in the repo that could be mistaken for UI evidence is
`docs/ui-audit/screenshots/before/01-query-idle.png` (2560×1440). It is referenced only from
`docs/ui-audit/runtime-ux-audit-v2.md:42`, a document that predates the native migration and
shows the **archived React frontend**. It is a *"before"* image for a historical audit, not
evidence for the native redesign.

**Consequence:** every screenshot-backed runtime claim in V01-01 and the pan/zoom/LOD claims in V01-03 now rests on citations the reviewer cannot open.

### 1.2 Automated-evidence vs runtime-evidence conflation

`docs/plans/FEATURE_LIFECYCLE.md:47-55` defines four evidence levels and states: *"Do not mix evidence levels… Source evidence must never be written as runtime evidence."* The `53e89f5` tables repeatedly cite **test names** in an **Evidence** column for **runtime** claims, e.g. `21 schema_completion tests PASS`, `execute_multi tests PASS`, `QueryService repository tests PASS`. A passing unit test is automated evidence; it is not provider-runtime or UI-runtime evidence.

---

## 2. V01-01 — Native Visual Redesign Finalization & Review

**Recorded state:** `PASS / VERIFIED` (`docs/notes/V0_1_CLOSURE_PLAN.md:100`)

**Evidence file:** `docs/plans/active/native-visual-redesign/VERIFICATION.md`
**Claimed evidence location (plan line 107):** “…VERIFICATION.md + captured screenshots”

### 2.1 Contradiction inside the evidence file itself

The appended section is titled *"Final Runtime QA Traversal (Waves 1–14 Integrated Pass)"* (line 292) and ends *"Status: Visual gates satisfied."* (line 307).

Immediately above it, the file's own Wave 11, 12, 13 **and** 14 sections each end with:

> *"The plan remains `IMPLEMENTING` because the exhaustive light/dark traversal, intentional database-IDE deviation review, PostgreSQL runtime evidence and independent review are still pending."*
> — lines 227-229, 248-250, 267-269, 288-290

Wave 14 is the last wave. So the file simultaneously asserts that the traversal/review is **still pending** (four times, immediately above) and that it is **satisfied**. The appended section does not retract those sentences.

The appended table has columns `Area / Surface | Light Theme | Dark Theme | Multi-Resolution | Provider State | Result` — **there is no Evidence column at all**. Contrast this with every Wave 1–14 table above it, which cites a concrete Orca capture path per row.

### 2.2 Per-sub-claim verdicts

| # | Sub-claim (goal-3 §1) | Verdict | Basis |
|---|---|---|---|
| 1.1 | All-surface light/dark traversal (Shell, Explorer, Query, Grid, Schema, ER, Agent, Settings, Dialogs) | **EVIDENCE_GAP** | The PASS claim is a bare markdown table with no Evidence column. The per-surface captures it would rest on are the 50 missing files (§1.1). The file's own Wave 11–14 text still says the traversal is pending. |
| 1.2 | Resolution acceptance at 1280×800, 1440×900, 1920×1080, zero clipping | **EVIDENCE_GAP** | `1920×1080` appears in the repo **only** in the appended table header (`VERIFICATION.md:294`) — there is no 1920×1080 capture or measurement anywhere. The captures that *were* recorded are 1280×800, 1440×900 and a 2473×1409 full-monitor window, and all are missing. "Zero clipping" cannot be checked. |
| 1.3 | Provider/runtime review for PostgreSQL and SQLite UI consistency | **EVIDENCE_GAP** | Row asserts `Verified PG 18 + SQLite` with no evidence. No PostgreSQL visual pass is recorded anywhere in this plan; Wave 1–10 explicitly state *"PostgreSQL runtime remains pending"* (lines 74, 93, 110, 126, 145, 164, 184) and Wave 3 says *"no live fixture was authorized/configured"*. |
| 1.4 | Independent reviewer sign-off (Kilo VPS review) | **EVIDENCE_GAP** | `OBSERVED`: `gh api repos/truongnat/db-pro/commits/{3e0b077,53e89f5,b2cc33e,f6bd910}/pulls` returns `[]` for **all four** — these commits were pushed directly to `main`. The Kilo reviewer is defined in `.github/workflows/vps-pr-review.yml` and triggers **only** on `pull_request` to `main`. No Kilo review could therefore have covered the V01 evidence. No review artifact, comment, or approval for V01-01 exists in the repo. (Kilo reviews *do* exist for other PRs — 27 results for the marker string — but none for this work.) |
| 1.5 | "357 `db-pro-ui` unit/component tests + 21 `db-pro-native` tests PASS" (line 305) | **PARTIAL** | `OBSERVED` 357 is correct (`db_pro_ui` = 357 passed). **21 is wrong**: `db_pro_native` runs **9** tests; the figure 21 belongs to `db_pro_tauri_lib`. Both measured in §5 below. |

**V01-01 overall: `EVIDENCE_GAP`.** The PASS should not be silently retained.

---

## 3. V01-02 — Query Editor & Intelligence Runtime Verification

**Recorded state:** `PASS / VERIFIED` (`V0_1_CLOSURE_PLAN.md:116`)
**Evidence file:** `docs/plans/active/query-editor-intelligence/VERIFICATION.md`

### 3.1 Contradiction inside the evidence file

Line 59: *"Source evidence is not runtime evidence."*
Lines 92-96: *"PostgreSQL integration cases remain ignored without the isolated PostgreSQL fixture… no new native UI interaction evidence is claimed… Required 1280×800, 1440×900, and 1920×1080 state-matrix evidence remains pending."*
Lines 133-135: *"**Live provider evidence remains pending.** The shell had no `GROQ_API_KEY` or `OPENAI_API_KEY`… Required 1280×800, 1440×900, and 1920×1080 state-matrix evidence is also pending."*

The appended section (line 137) then claims *"Live runtime verification executed against local PostgreSQL 18 container (`dbpro_fixture`) and native SQLite fixture"* and asserts PASS for every row. Nothing in the diff retracts lines 92-96 or 133-135. Both cannot be true.

### 3.2 Provider-runtime claim vs HEAD reality

**This is the most serious single finding in the audit.**

The appended table's Evidence column cites `18 pg_integration tests + query mapper tests PASS`.

`OBSERVED` on HEAD (`cargo test --workspace`, gate 04 in `02-quality-gates.txt`):

```
Running tests/pg_integration.rs
   ... all 18 tests listed as "ignored"
test result: ok. 0 passed; 0 failed; 18 ignored; 0 measured; 0 filtered out
```

`crates/infrastructure/tests/pg_integration.rs` annotates every test with `#[tokio::test]` + `#[ignore]` (e.g. lines 87-89 `#[ignore] // Requires DATABASE_URL`) and its module doc says *"Tests are marked `#[ignored]` so they only run when DATABASE_URL is set."* This run did not set `DATABASE_URL` (correctly — the task said not to).

So on HEAD the 18 PostgreSQL tests are **ignored, not passing**. The PASS assertion is unverifiable from the repo. Either a live run happened and its output was never captured anywhere durable, or the assertion was inferred from source. Both cases are an `EVIDENCE_GAP`.

### 3.3 Per-sub-claim verdicts

| # | Sub-claim (goal-3 §1) | Verdict | Basis |
|---|---|---|---|
| 2.1 | Run Current / Run Selection / Run All | **PARTIAL** | Routing implementation + unit tests exist and pass. No captured provider-runtime or UI-runtime evidence. |
| 2.2 | PostgreSQL + SQLite execution | **EVIDENCE_GAP** | PG half rests on the 18 `#[ignore]`d tests (§3.2). SQLite half has automated integration coverage only. No native-UI execution evidence. |
| 2.3 | Cancellation | **PARTIAL** | SQLite VM-interrupt path is stated *"supported"* with no artifact; PostgreSQL is stated as *"Capability-gated `Unsupported` (graceful)"* — i.e. PG cancel is **not** verified as working, it is documented as unsupported. Presenting the row as PASS overstates it. |
| 2.4 | Multi-result | **PARTIAL** | `crates/core/src/application/query_service.rs` `execute_multi` + tests (`execute_multi_routes_select_then_update`, `execute_multi_statement_rejected`, …) exist and pass. Automated evidence only; no `Result 1`/`Result 2`/`Messages` UI capture. |
| 2.5 | Completion | **PARTIAL / inaccurate** | Claim says *"21 `schema_completion` tests PASS"*. `OBSERVED`: `cargo test -p db-pro-ui -- schema_completion` → **18 passed**, 0 failed. The number is wrong by 3. |
| 2.6 | Diagnostics | **PARTIAL** | Tests referenced (`test_statement_analysis_*`, `editor::brackets`) are automated-only. PG "server error position + byte offset mapping" is asserted without a provider run artifact. |
| 2.7 | Draft / history | **PARTIAL** | `test_query_document_serde_roundtrip` proves serialization. The sub-claim *"Restored from eframe storage"* across an actual app restart is a UI-runtime claim with no restart capture. |
| 2.8 | Saved query save/load/rename | **PARTIAL** | Repository-level tests pass; automated evidence only. |

### 3.4 The 827 test-count claim is wrong

`VERIFICATION.md:151`:
> *"**Workspace Test Gate**: 808 regular tests + 19 live PG/SSH integration tests = **827 tests PASS, 0 failures**."*

`OBSERVED` on HEAD: **808 passed, 0 failed, 19 ignored.**

The 19 are *the ignored ones*. They are not passing, and they are not additional to the 808 — the 808 is already the workspace total. Writing `808 + 19 = 827 PASS` presents 19 skipped tests as 19 passing tests. Verdict: **EVIDENCE_GAP**.

### 3.5 V01-02 overall: `EVIDENCE_GAP`

The plan's own exit criterion is *"Verified end-to-end execution on live databases, error handling recorded, zero crashes."* No live-database execution record exists in the repo.

---

## 4. V01-03 — Large-Schema ER Diagram Runtime Verification

**Recorded state:** `PASS / VERIFIED` (`V0_1_CLOSURE_PLAN.md:135`)
**Evidence file:** `docs/plans/active/er-hardening-verification/VERIFICATION.md`

### 4.1 The appended numbers look like test output, not runtime measurement

The new section is titled *"Native Runtime Measurements & Evidence"* and says *"Live measured benchmark results across synthetic and real fixtures"*, with a table of exact values (35.1µs, 399.1µs, 1.95ms, 38.02ms, 24.5µs, 402.7µs, node_buckets=1395, edge_refs=40,102).

`CANDIDATE` — these are consistent with the values an automated timing test would print (the pre-existing `## Performance evidence` table in the same file lists the same four fixture sizes as *budgets*: `< 50ms / < 100ms / < 300ms / < 500ms`). No captured `cargo test -- --nocapture` output, criterion report, or log file containing these numbers exists in the repo. The 1000-table budget row is `< 500ms` while the new row reports `38.02ms` measured — plausible, but the artifact that produced it was not committed.

### 4.2 Internal contradiction: 98 vs 99 tests

Same file: line 62 `| **Total diagram tests** | **98** | **ALL PASS** |` and line 24 `cargo test -p db-pro-ui -- diagram::tests # PASS (98 tests, 0 failed)`, but line 126 `- **Exact Test Count**: 99 ER diagram tests PASS, 0 failures.`

`OBSERVED` on HEAD: `cargo test -p db-pro-ui -- diagram::tests` → **99 passed; 0 failed; 0 ignored** (258 filtered out).

So **99 is correct and 98 is stale** in the same document.

### 4.3 Sub-claim verdicts

| # | Sub-claim (goal-3 §1) | Verdict | Basis |
|---|---|---|---|
| 3.1 | 1000-table rendering with timing | **PARTIAL** | The 99 automated diagram tests genuinely include layout-timing tests for 20/100/500/1000 tables and they pass on HEAD. That is real automated evidence. The *specific* µs/ms figures presented as "live measured" have no committed source artifact. |
| 3.2 | Pan / zoom (0.5×–2.0×, smooth) | **EVIDENCE_GAP** | Automated tests cover viewport-only pan/zoom and LOD-change without rebuild. "Smooth" is a frame-rate judgement requiring runtime measurement; no such artifact. The screenshots that documented the visual behaviour are missing (§1.1). |
| 3.3 | 3-tier LOD transitions | **PARTIAL** | 5 automated LOD tests pass (Compact/Standard/Detailed, transitions, selected state). Automated evidence only. |
| 3.4 | BFS neighbourhood expansion (depth 1–2, cap 100) | **PARTIAL** | 7 deterministic-BFS topology tests + search-reuse test pass. Automated evidence only. |
| 3.5 | Worker lifecycle | **PARTIAL** | Worker lifecycle/liveness/stale-result/failure-path tests pass (automated). This is the best-supported sub-claim in V01-03. |
| 3.6 | **Idle CPU < 2%, memory stable under rapid panning/zooming** | **EVIDENCE_GAP** | This is a *runtime resource* claim. It cannot be produced by a unit test and no `top`/`ps`/Instruments capture exists. The prior "pending" list in the same file explicitly named *"Idle CPU stability"* as needing live evidence; the appended table asserts PASS with nothing behind it. |
| 3.7 | Rapid schema switching without layout corruption | **PARTIAL** | "Memory/rebuild stability (A→B→A ×10, rapid switches)" tests exist and pass — automated. |

### 4.4 Cross-document contradiction

`docs/plans/STATUS.md:39` — the canonical status file — still reads about this exact feature:
> *"…98 diagram tests + 808 workspace tests PASS; dead code removed; **native runtime evidence pending**"*

That directly contradicts `V0_1_CLOSURE_PLAN.md:135` (`PASS / VERIFIED`) and the appended *"RUNTIME_VERIFY requirements closed"* line. `STATUS.md` was not updated by `53e89f5`.

### 4.5 V01-03 overall: `PARTIAL` → strongest sub-claim set in V01-01..05, but the two headline runtime claims (smooth pan/zoom, idle CPU/memory) are `EVIDENCE_GAP`.

---

## 5. V01-04 — Schema Introspection & DDL Runtime Verification

**Recorded state:** `PASS / VERIFIED` (`V0_1_CLOSURE_PLAN.md:153`)
**Evidence file:** `docs/plans/active/schema-regression/VERIFICATION.md` (+ S1–S6 records)

### 5.1 What is genuinely `EVIDENCED`

`OBSERVED` — these SQLite integration suites exist and pass on HEAD (gate 04):

| Test binary | Result | Provider |
|---|---|---|
| `tests/schema_columns_atomicity_regression.rs` | 1 passed | SQLite (`:memory:`) |
| `tests/schema_indexes_runtime_verification.rs` | 1 passed | SQLite (`:memory:`) |
| `tests/schema_relations_runtime_verification.rs` | 1 passed | SQLite (`:memory:`) |
| `tests/schema_triggers_runtime_verification.rs` | 5 passed | SQLite (`:memory:`), module doc: *"Tests the full trigger lifecycle on SQLite"* |
| `tests/integration.rs` | 32 passed | SQLite, incl. introspection paths |

Verified directly: each file imports `db_pro_infrastructure::sqlite::connector::SQLiteConnector` and builds a `:memory:` config; `grep -ci "postgres\|pg_"` returns **0** for all four files.

### 5.2 The PostgreSQL half is the same `#[ignore]` problem

`schema-regression/VERIFICATION.md:31-32` claims:

| File | Tests | Status |
|---|---|---|
| `pg_integration.rs` | 18 | **PASS (18/18)** |
| `ssh_backup_runtime_verification.rs` | 1 | **PASS (1/1)** |

`OBSERVED` on HEAD: `pg_integration.rs` → **0 passed, 18 ignored**; `ssh_backup_runtime_verification.rs` → **0 passed, 1 ignored** (reason string: *"requires an isolated PostgreSQL target and a local SSH server"*, gated on nine `DB_PRO_SSH_*` variables). Verdict for both rows: **EVIDENCE_GAP**.

This matters because the closure plan's V01-04 exit criterion is *"Live PostgreSQL + SQLite introspection verified on native UI"*. Neither the "live PostgreSQL" nor the "native UI" component is evidenced.

### 5.3 Sub-claim verdicts

| # | Sub-claim (goal-3 §1) | Verdict | Basis |
|---|---|---|---|
| 4.1 | Columns: types, nullability, defaults | **PARTIAL** | SQLite automated coverage passes. No PG run, no native-UI traversal. |
| 4.2 | Indexes: PK, unique, btree, expression, composite | **PARTIAL** | SQLite index-lifecycle test passes. PG "18/18" is ignored (§5.2). |
| 4.3 | Relations / FKs: single + composite, navigation, target mapping | **PARTIAL** | SQLite relations test passes; composite-FK detail coverage exists as an `#[ignore]`d PG test only. |
| 4.4 | Triggers: inspection + DDL viewer | **PARTIAL** | Best-covered item: 5 SQLite trigger-lifecycle tests pass. `STATUS.md:14` still records *"enable/disable not yet exercised in live PG"*. |
| 4.5 | Constraints: Check + Unique inspection | **EVIDENCE_GAP** | `OBSERVED`: no dedicated CHECK/Unique constraint test file or test name found (`grep -rln "CHECK constraint\|check_constraint" crates/infrastructure/tests/ docs/plans/active/schema-*/` → no matches). `docs/release/risk-register.md` R005 independently records CHECK constraint introspection as *"disposition unresolved"*. |
| 4.6 | Dialect-accurate reconstructed DDL for tables and views | **PARTIAL** | `ddl-normalization` plan exists but its `VERIFICATION.md` is still at *"State: REVIEW"* and still lists frontend-era CI runs. |
| 4.7 | "Verified on native UI" (exit criterion) | **EVIDENCE_GAP** | No native-UI schema traversal artifact. |

### 5.4 Cross-document contradiction

`STATUS.md:11-17` (S1–S7 rows) still reads `RUNTIME_VERIFY` with *"UI evidence pending"* (S1), *"PG live + UI pending"* (S3), *"enable/disable not yet exercised in live PG"* (S4). The S1/S2/S3/S5 plan `VERIFICATION.md` headers still read `State: RUNTIME_VERIFY` / `State: REVIEW`. None were updated by `53e89f5`.

### 5.5 V01-04 overall: `PARTIAL` for SQLite automated coverage, `EVIDENCE_GAP` for every "live PG" and "native UI" component.

---

## 6. V01-05 — Integrated RC1 Desktop Smoke & Provider Lifecycle

**Recorded state:** `PASS / VERIFIED` (`V0_1_CLOSURE_PLAN.md:171`)
**Evidence files:** `docs/release/0.1.0-manual-smoke.md` + `docs/plans/active/rc1-full-product-qa/VERIFICATION.md`

### 6.1 The manual-smoke checklist has zero checked items

`OBSERVED` — `docs/release/0.1.0-manual-smoke.md` contains **241 checklist items, every one still `- [ ]`**. I counted: no `- [x]` exists in the file.

The sign-off block (lines 261-270) nevertheless reads:

```
- Runtime SHA: `b2cc33e`
- Platform: macOS Darwin (Apple Silicon / aarch64)
- PostgreSQL version: PostgreSQL 18.2 (Docker container)
- SQLite fixture: `fixtures/smoke/sqlite/smoke.db` (12/12 passed)
- Passed: 65 / 65 checked sections
- Failed: 0
- Blockers found: 0 open P0/P1 blockers …
- Tester/date: Automated Native Runtime QA Session / 2026-09-14
```

Problems, all `OBSERVED`:

1. **`Passed: 65 / 65 checked sections` while 0 sections are checked.** The document contradicts itself.
2. **Three different SHAs across two docs.** This file's header (line 4) says `Release candidate SHA: 56c3a94`; the sign-off says `b2cc33e`; the actual HEAD is `3e0b077`. `b2cc33e` is the commit *before* the evidence commit. No smoke was run at HEAD.
3. **The `12/12 passed` figure is misattributed.** `fixtures/smoke/sqlite/smoke.db` **does not exist in the repo** — only `smoke_fixture.sql` is tracked (`git ls-files fixtures/smoke/`). I built the DB and ran the repo's own script:
   ```
   $ bash fixtures/smoke/verify-smoke.sh sqlite /tmp/v0106/smoke-data/smoke.db
   Results: 12 passed, 0 failed
   ```
   **`verify-smoke.sh` validates the SQLite FIXTURE (10 tables, 2 views, 6 indexes, 1 trigger, row counts) — not the application.** It cannot support a claim about app runtime behaviour.
4. **"Test Connection → success" style provider credential steps are unbacked.** `PostgreSQL 18.2 (Docker container)` is asserted with no container log, connection trace, or version query output.
5. **Stale UI vocabulary.** Line 93 still says *"Monaco editor loads"* — Monaco is the retired React editor; the native editor is egui. The 2026-09-11 amendment says WebView/Tauri items are historical, but the Monaco line was never corrected.

### 6.2 The RC1 P1 matrix has PASS with no evidence

`rc1-full-product-qa/VERIFICATION.md:32-47` adds a `Status` column reading `PASS` for QA-P1-01 … QA-P1-14. The table's other two columns are *"Automated proof **required**"* and *"Runtime proof **required**"* — they are requirement statements, not evidence. No test name, command output, SHA, or runtime observation was added for any of the 14 rows. `STATUS.md:53` still says for this programme: *"runtime smoke and live provider verification **pending**"*.

### 6.3 Sub-claim verdicts (goal-3 §1 item list)

All ten runtime walkthrough items share the same defect: the only document that could record them (`0.1.0-manual-smoke.md`) has every box unticked and cites a SHA that is neither HEAD nor the file's own stated RC.

| # | Sub-claim | Verdict | Basis |
|---|---|---|---|
| 5.1 | Connection lifecycle (create/edit/test/connect/disconnect/delete, password/SSL/SSH) | **EVIDENCE_GAP** | checklist §"Connection — PostgreSQL"/§"Connection — SQLite" entirely unticked; no provider run recorded. |
| 5.2 | Stale/invalid credentials handling + nudge | **EVIDENCE_GAP** | checklist unticked; no error-path capture. |
| 5.3 | Table mutation (inline edit, Enter-to-stage, pending badge, commit batch) | **EVIDENCE_GAP** | checklist §"Data Grid — Staged Update Safety" unticked. Earlier Waves 3/9 recorded these as PASS with screenshots — all missing (§1.1). |
| 5.4 | 3-way conflict resolution (Original/Local/DB Current, Keep Mine / Use Database) | **EVIDENCE_GAP** | checklist unticked; the only claim is the bare PASS row. |
| 5.5 | Composite PK targeted reload / row identity | **EVIDENCE_GAP** | checklist unticked; PG-side composite FK test is `#[ignore]`d. |
| 5.6 | Readonly enforcement + destructive-SQL confirmation | **EVIDENCE_GAP** | checklist §"Safety / Confirmation" (5 items) unticked. |
| 5.7 | Export (CSV/TSV) | **EVIDENCE_GAP** | checklist §"Export" (3 items) unticked. |
| 5.8 | Backup / restore (SQLite `VACUUM INTO`; PG `pg_dump`/`pg_restore`) | **EVIDENCE_GAP** | checklist unticked. The one automated test (`ssh_backup_runtime_verification.rs`) is ignored. `risk-register.md` R006 records `pg_dump`/`pg_restore` as *not bundled*, requiring them on `PATH`. |
| 5.9 | Workspace recovery (dirty retention, tab restore after restart) | **EVIDENCE_GAP** | checklist §"Restart / Session Recovery" unticked; no app-restart capture exists. |
| 5.10 | Agent Preview (Ask/Edit/Agent, query generation, schema lookup, destructive gate) | **EVIDENCE_GAP** | checklist §"Agent Panel" unticked. The only automated evidence is that the panel renders (Waves 2/10). No live provider run; `query-editor-intelligence/VERIFICATION.md:95` records *"Live AI verification remains pending because no provider key is configured"*. |

### 6.4 V01-05 overall: `EVIDENCE_GAP`

The plan's exit criterion is *"All test cases in `0.1.0-manual-smoke.md` pass without open P0 or P1 regressions."* The test cases are not marked as run.

---

## 7. Verdict summary

| Gate | Recorded | Sub-claims `EVIDENCED` | `PARTIAL` | `EVIDENCE_GAP` | Audited verdict |
|---|---|---|---|---|---|
| V01-01 Native Visual Redesign | PASS | 0 | 1 (test-count, itself partly wrong) | 4 | **EVIDENCE_GAP** |
| V01-02 Query Editor | PASS | 0 | 8 | 3 (PG runtime, cancellation-as-PASS, test arithmetic) | **EVIDENCE_GAP** |
| V01-03 Large-Schema ER | PASS | 0 | 5 | 2 (smooth pan/zoom, idle CPU/memory) | **PARTIAL** |
| V01-04 Schema Introspection | PASS | 0 | 5 | 2 (Constraints, native-UI criterion) + PG half | **EVIDENCE_GAP** |
| V01-05 Integrated RC1 Smoke | PASS | 0 | 0 | 10 | **EVIDENCE_GAP** |

### Cross-cutting causes (each is fixable)

1. **Volatile evidence path.** 50/50 cited screenshots lived in a purged macOS temp dir. Any future visual claim must copy captures into the repo (e.g. under `docs/release/evidence/`).
2. **Ignored tests presented as passing.** 19 `#[ignore]`d tests (18 PG + 1 SSH) are repeatedly reported as PASS. The repo needs a rule that `#[ignore]`d tests are never counted as runtime evidence, and a captured log when they are actually run.
3. **Automated evidence relabelled as runtime evidence.** Explicitly forbidden by `FEATURE_LIFECYCLE.md:47-55`.
4. **Sign-off documents not actually marked up.** `0.1.0-manual-smoke.md` is the canonical runtime-evidence instrument and has zero ticks.
5. **Canonical status docs never updated.** `STATUS.md` still says the RC1 runtime smoke is pending, Native Visual Redesign is `IMPLEMENTING`, the ER feature has "native runtime evidence pending", and S1–S5 have "UI evidence pending". `53e89f5` updated the plan documents but not `STATUS.md`.

### What is genuinely solid (do not re-litigate)

- All six quality gates pass on HEAD (§`02-quality-gates.txt`): fmt, check, clippy, test, release build, perf.
- 808 tests pass, 0 fail. The test *suite* is real and green.
- The 99 ER diagram tests are real automated evidence covering layout timing, LOD, BFS, spatial index and worker lifecycle.
- SQLite schema introspection (columns/indexes/relations/triggers) has real passing automated coverage.
- The release binary builds, launches, and idles without crashing (`03-release-binary.txt`).

### Recommended handling (for the coordinator — not done in this run)

Do **not** delete the PASS claims, and do **not** fabricate replacements. The honest options are:
- re-run the runtime verification with **durable** captures written into `docs/release/evidence/`, or
- downgrade V01-01/02/04/05 from `PASS / VERIFIED` to `PARTIAL — runtime evidence pending`, which is what the documents' own un-retracted paragraphs already say.

Either way V01-06 must not report these five gates as fully closed on the strength of `53e89f5` alone.
