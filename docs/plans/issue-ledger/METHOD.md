# Method — how each disposition was decided

Baseline: `main @ a9c1174cbfdfd0105c939c3bc703f4c6fa07a975`, branch `main`, worktree clean
(`git status --porcelain -uall` empty). Snapshots: `issues-open-2026-09-14.json` (136 issues) and
`issues-open-2026-09-14-late.json` (11 issues, #217–#227). Nothing on GitHub was modified — the late
snapshot and the live count were fetched with **read-only** `gh issue list` calls — and no product code
was touched by this pass.

This file exists so a later reader can **re-derive or falsify** the classification rather than
trust it. Every claim in `INVENTORY.md` is reproducible with the probes below.

## 1. Evidence hierarchy

When two sources disagreed, the lower number won:

1. **Code on `main` at `a9c1174`** — the file:line was opened and read (via `Read` or `sed -n`).
2. **Machine-generated evidence under `docs/release/evidence/`** — raw command output with an
   exit status (`v01-06/`, `v01-runtime/providers/`), plus the CI run id. A recorded PASS/FAIL
   in a prose document is *not* the same thing and is treated as a claim.
3. **Release-truth documents** — `docs/release/0.1.0-readiness.md`, `0.1.0-handoff.md`,
   `0.1.0-final-report.md`, `known-limitations.md`, `risk-register.md`,
   `provider-capability-matrix.md`, `platform-prerequisites.md`, `0.1.0-packaging.md`,
   `0.1.0-human-decisions.md`, `docs/plans/STATUS.md`, `docs/notes/V0_1_CLOSURE_PLAN.md`.
4. **Issue bodies.** Used for the *question*, never for the *answer*. Several bodies describe a
   state that has since changed in the safe direction (#147) or a layer that no longer exists
   (#60, #63, #65, #84, #115, #116).
5. **Historical documents are not evidence.** `docs/release/0.1.0-source-audit.md` and
   `docs/plans/STATUS.md`'s frontend-era rows are explicitly marked historical by their own
   headers (`0.1.0-source-audit.md:4-20`, `STATUS.md:44`).

`DONE_ON_MAIN` required a pointer from (1) or (2). A document alone never produced a
`DONE_ON_MAIN` row — which is why, for example, #70 (CHECK end-to-end exposure) is
`PARTIAL_ON_MAIN` even though the issue's *premise* is obsolete: the code path exists, but the
normalisation/serialization-shape work it also asks for does not.

## 2. The probes

Run from the repository root. These are the commands this triage actually used, grouped by what
they settle.

### 2.1 Snapshot and repository state

```bash
python3 -c "import json;d=json.load(open('docs/plans/issue-ledger/issues-open-2026-09-14.json'));print(len(d))"   # 136
python3 -c "import json;d=json.load(open('docs/plans/issue-ledger/issues-open-2026-09-14-late.json'));print(len(d))"  # 11
# read-only live recount (no GitHub mutation):
gh issue list --state open --limit 300 --json number --jq 'length'                      # 147
gh issue list --state open --limit 300 --json number,title,labels,body,createdAt,updatedAt,author,milestone \
  --jq '[.[] | select(.number >= 217)] | sort_by(.number)' > docs/plans/issue-ledger/issues-open-2026-09-14-late.json
git log --oneline -1                     # a9c1174 docs(release): clarify the HEAD field ...
git status --porcelain -uall             # empty
git tag -l                               # empty  -> no v0.1.0 tag exists (#108)
git branch --show-current                # main
git remote -v                            # origin https://github.com/truongnat/db-pro.git
```

The snapshot is a copy of the harness-provided `/tmp/dbpro-issues/open.json`; the copy was
verified byte-identical (`json.load(a) == json.load(b)`).

### 2.2 Which documents answer which issue

Several documents carry an explicit `Issue: #NN` marker, which is the strongest kind of pointer
for a document-deliverable issue:

```bash
grep -rn "Issue: #" docs/ README.md CHANGELOG.md
# docs/release/known-limitations.md:16     > Issue: #135
# docs/release/provider-capability-matrix.md:10  > Issue: #132
# docs/release/platform-prerequisites.md:22      > Issue: #134
grep -rn "FINAL_RC_SHA" docs/ README.md CHANGELOG.md     # no matches -> #89 not executed as written
ls LICENSE* COPYING* NOTICE* 2>&1 | head                 # no matches -> #119 open, R-LICENSE BLOCKING
grep -rn "updater\|auto-update" docs/release/*.md README.md | head
```

### 2.3 Gate 5 / provider-value chain

```bash
grep -n "and_utc\|to_rfc3339\|TIMESTAMP" crates/infrastructure/src/postgres/query_mapper.rs   # :369-372 invent UTC for TIMESTAMP
sed -n '329,400p' crates/infrastructure/src/postgres/query_mapper.rs                          # decode_cell arms
sed -n '53,84p'   crates/core/src/domain/query.rs                                             # CellValue variants + serde tags
sed -n '649,668p' crates/native-app/src/translate.rs                                          # map_cell: 7 variants -> UiCell::Text
sed -n '1821,1840p' crates/ui/src/result_grid_view.rs                                         # copy-as-JSON parses to f64
sed -n '1357,1389p' crates/ui/src/query_view.rs                                               # UI export writer, no escaping
grep -rn "CREATE DOMAIN" fixtures/                                                           # 0 matches -> domain path untested (#58/#59)
grep -rn "#\[ignore" crates --include="*.rs" | wc -l                                         # 20 hits = 19 tests + 1 prose line
```

### 2.4 The four "do not wave through" P1 concerns

Each was inspected in the code, not inferred from docs.

**PostgreSQL secure-by-default (#144):**

```bash
grep -n "SslMode" -A 10 crates/core/src/domain/connection.rs | head -20   # Disable is #[default]
grep -rn "SslMode::Disable" crates --include="*.rs" | wc -l               # 41 occurrences, mostly fixtures/tests
sed -n '1,20p' crates/infrastructure/src/postgres/connection_string.rs    # mapping only, no policy
sed -n '622,639p' crates/ui/src/connection_view.rs                         # 4-way selector, no warning
```

**Real OS keyring stores / DEV-only fallback (#142):**

```bash
grep -n "with_fallback\|with_session_fallback\|KeyringVault::new" crates/runtime/src/lib.rs
#   73:  KeyringVault::new("com.dbpro.app", secrets_dir)
#   74:      .with_session_fallback()
#   75:      .with_fallback(),
git log --oneline -S'with_fallback' -- crates/runtime/src/lib.rs crates/tauri-app/src/lib.rs
#   81dcad19 fix(security): use native OS keyring without production fallback
#   cce09fd3 feat(ui): refine table editor toolbar ...   <- re-introduced it in the runtime wiring
grep -rn "dev-only\|disabled in production\|no longer the production default" docs/architecture/security-boundaries.md docs/release/provider-capability-matrix.md docs/release/risk-register.md
# security-boundaries.md:38 claims 'dev/CI only, disabled in production' - contradicted by lib.rs:73-75
sed -n '150,250p' crates/infrastructure/src/secret/keyring_vault.rs       # fallback written first, read first; log line :244
grep -n "keyring" Cargo.toml crates/native-app/Cargo.toml                 # platform features ARE enabled (apple/windows/linux-native)
```

**SQLite backup/restore snapshot safety (#145):**

```bash
grep -n "VACUUM INTO\|hard_link\|fs::copy\|rename" crates/infrastructure/src/backup/sqlite_backup.rs
sed -n '58,156p' crates/infrastructure/src/backup/sqlite_backup.rs       # backup :58-113, restore :115-156
sed -n '61,71p'  crates/core/src/application/backup_service.rs           # restore blocked while connection active
grep -rn "wal_checkpoint\|journal_mode" crates --include="*.rs" | wc -l  # 0 -> no WAL handling
grep -n "rusqlite" Cargo.toml                                            # features = bundled, column_decltype -> no online-backup API
```

**Multi-statement partial-write semantics (#147):**

```bash
sed -n '213,239p' crates/core/src/application/query_service.rs           # any mutation -> single transaction
sed -n '262,299p' crates/core/src/application/query_service.rs           # failure -> 'transaction rolled back' / 'outcome unknown'
sed -n '388,405p' crates/infrastructure/src/postgres/connector.rs        # tx.rollback() on statement failure
sed -n '16,41p'   crates/core/src/application/sql_policy.rs              # lexical splitter
sed -n '66,98p'   crates/core/src/domain/safety.rs                       # destructive classification
# 0 hits: the classifier is never called from crates/ui, crates/native-app, crates/tauri-app
grep -rn "classify_statement_safety\|StatementSafety" crates/ui crates/native-app crates/tauri-app --include="*.rs" | wc -l
# Agent gap: 349 classifies the whole string, while the draft path guards multiple statements (agent.rs:678-683)
sed -n '345,361p' crates/core/src/domain/agent_workflow.rs; sed -n '674,690p' crates/runtime/src/agent.rs
```

### 2.5 Retired layers (the `SUPERSEDED` set)

```bash
sed -n '40,50p' docs/plans/STATUS.md            # React frontend archived 2026-09-11, product UI is native egui
grep -n "Retired\|retired\|archived" CHANGELOG.md | head
ls _archive/frontend/src/modules/query/types/    # the TS CellValue type still cited by old issues
sed -n '13,20p' .github/workflows/ci.yml        # single CI job 'Rust checks' -> no frontend gates to run
sed -n '71,113p' crates/ui/src/diagram/tests.rs # native ER fixtures replace the React-era factory
```

### 2.6 Container issues

Container/epic bodies carry a `## Status` line that is stale in every case (e.g. #22 says
"In Progress — Workstream A" while A1 is closed and A2 is partly implemented). Container
dispositions were therefore derived from children plus release documents:

```bash
grep -rn "Gate 5\|QA-W5" docs/plans/active/rc1-full-product-qa/CHECKLIST.md   # :127-143 shows unchecked enum/array rows
sed -n '18,27p' docs/plans/STATUS.md                                          # corrected V01 evidence verdicts
sed -n '25,33p' docs/plans/STATUS.md                                          # measured 815/0/19
sed -n '1,50p'  docs/release/evidence/v01-06/06-rc1-p2-dispositions.md        # #74's baseline + 25-finding inventory
sed -n '1,10p'  docs/release/risk-register.md                                 # run 34860902181, all jobs green
sed -n '25,39p' docs/release/0.1.0-readiness.md                               # INTERNAL_RC_READY=YES / PUBLIC_RELEASE_READY=NO
```

## 3. Decision rules that produced the dispositions

| Situation | Disposition chosen | Example |
|---|---|---|
| The deliverable exists and was read on this tree | `DONE_ON_MAIN` | #132 (`provider-capability-matrix.md:10`), #135 (`known-limitations.md:16`) |
| Part of the deliverable exists; the rest is named | `PARTIAL_ON_MAIN` | #56, #58, #70, #88, #134 |
| Bounded change, no owner decision, no external resource, not blocked on another open issue | `ACTIONABLE_NOW` | #56, #61, #121, #142, #144, #145, #147 |
| The next action is a product/governance choice | `NEEDS_OWNER_DECISION` | #119 (license), #101 (naming), #146 (branch protection), #68 (CHECK scope) |
| The next action needs a host/session/service this environment lacks | `NEEDS_EXTERNAL_RESOURCE` | #92, #93 (Windows/Linux hosts), #95 (needs #92/#93), #100 (live web diligence), #112 (Projects v2) |
| The issue's own body places it after v0.1 | `OUT_OF_SCOPE_V01` | #32–#35, #182–#216 |
| The layer/consumer it targets was retired; the replacement is nameable | `SUPERSEDED` | #60, #63, #65, #84, #115, #116, #113 |
| Body plus tree cannot settle it | `UNCLEAR` | *none — see §5* |

Two refinements worth stating explicitly, because they are where a triage usually cheats:

- **Gate 5 containers are not "done".** The decoder, the live provider run and the lossless
  i64/NUMERIC handling are real and evidenced, but two of the gate's own non-negotiables —
  "timestamp-without-time-zone never gains invented timezone semantics" and "representation
  decisions are locked in Workstream A before decoder/UI implementation" — are unmet
  (`query_mapper.rs:369-372`; no A2/A3/A4 contract document exists). #21/#22/#23/#24/#52 are
  `PARTIAL_ON_MAIN` for that reason, and closing them would be a false claim.
- **A regression against a doc is recorded, not smoothed over.** `docs/architecture/security-boundaries.md:38`
  and `docs/release/risk-register.md:318` state the encrypted credential fallback is not used in
  production; `crates/runtime/src/lib.rs:73-75` unconditionally enables it. That contradiction is
  the basis of #142 being `ACTIONABLE_NOW` and of #122 staying `PARTIAL_ON_MAIN`.

## 4. What was deliberately *not* done

- No issue was closed, commented on, edited, relabelled or re-milestoned.
- No product code, test, workflow or release document was modified. This pass adds files under
  `docs/plans/issue-ledger/` only.
- No claim was accepted from an issue body's own `Status:` line, a milestone, or the age of the
  issue.
- No `DONE_ON_MAIN` was granted on the strength of a document title alone. Where a document is
  partly historical (for example `platform-prerequisites.md`, whose matrix still describes the
  retired Tauri bundler), the row says so and the disposition is `PARTIAL_ON_MAIN`.

## 4b. The two snapshots (why the row count is 147 and not 136)

The frozen input had 136 issues. Before publishing this triage, the live open count was re-checked
read-only:

```bash
gh issue list --state open --limit 300 --json number --jq 'length'   # 147
```

The difference is 11 issues, #217–#227 (created 2026-09-14T16:52–16:54Z, i.e. a few minutes after the
first snapshot was written). They are the tail of the same post-v0.1 decomposition as #182–#216: every
one of them is either `[Phase …]` or a `[Query]`/`[Data]`/`[ER]`/`[Connections]` feature with
`Parent Goal: #182`, and all 11 are `OUT_OF_SCOPE_V01` for the same reason as the rest of that family.

They were kept in a **separate snapshot file and a separate table** rather than merged into the first
one, for two reasons: the first snapshot is the frozen input this triage was computed from and must stay
reproducible, and the late arrivals arrived with no chance to be reviewed against the tree in the same
pass. The inventory therefore states three numbers explicitly — 136 (snapshot 1), 11 (snapshot 2) and
147 (rows = live open count) — and the generator asserts
`len(rows) == len(snapshot1) + len(snapshot2)`.

Two of the late arrivals overlap open v0.1 items and could be mistaken for v0.1 work; their rows say so:
#223 (advanced TLS/SSH profiles) overlaps #144, and #224 (explicit transaction controls) is the natural
home of the contract #147 must record. Both remain post-v0.1.

## 5. On the absence of `UNCLEAR`

The vocabulary allows `UNCLEAR`, and **zero rows use it**. That is a result, not an omission:
every one of the 136 issues resolved to a code path, a document, a release decision or an
explicit scope statement, because this repository is unusually well instrumented (release
readiness, risk register, limitations registry, capability matrices, two evidence trees) and
because the issue bodies carry their own acceptance criteria.

If a row is challenged, the honest failure mode is a wrong `PARTIAL_ON_MAIN`/`DONE_ON_MAIN`
boundary — so the confirmation pass in phase 2 must re-open the cited pointer before closing
anything, and `LEDGER.md` must record the correction rather than overwrite it.

## 6. Reproducing `INVENTORY.md`

`INVENTORY.md` is generated: a classification map (issue number → priority, disposition, note,
evidence pointer, estimate, batch) is joined against the frozen JSON, and the generator aborts if
the sets do not match exactly:

```
open issues in snapshot: 136
rows written          : 136
assert len(ISSUES) == len(issues)      # passes
```

The generator was run from `/tmp` and writes only `docs/plans/issue-ledger/INVENTORY.md`. To
regenerate: edit the classification map, re-run, then verify the row count in the `## Inventory`
section equals the open-issue count in the snapshot:

```bash
python3 - <<'PY'
import re
txt = open('docs/plans/issue-ledger/INVENTORY.md').read()
rows = [l for l in txt.split('## Inventory')[1].splitlines() if re.match(r'^\| \d+ \|', l)]
print(len(rows))    # 136
PY
```
