# Execution plan — working the 136 open issues

Companion to [`INVENTORY.md`](INVENTORY.md) (what each issue is) and [`LEDGER.md`](LEDGER.md) (what
was actually done). This file is the *order* and the *reason* for the order.

Baseline: `main @ a9c1174`. **147 open issues** (136 in the frozen snapshot plus 11 created minutes
after it — see `INVENTORY.md`). Of those, **12 are `ACTIONABLE_NOW`**, 50 are `PARTIAL_ON_MAIN` with a
named remainder, 10 are `DONE_ON_MAIN`, 18 need the owner or a host, 7 are superseded and 50 are
post-v0.1 scope that must not be worked now.

## Ordering principles

1. **Close the safe ones first (batch 0).** Ten issues already have a verified pointer. Confirming and
   closing them takes one pass, shrinks the board by 7%, and cannot be invalidated by later work if it
   happens *before* the code moves.
2. **Truth before features.** A release may state a limitation or a claim; it must not state a false
   one. The issues that fix a false or missing release claim (#142, #144, #147, #145, #61) come before
   anything additive.
3. **Bounded before unbounded.** Within a batch, an S/M change that closes an issue outright precedes an
   L/XL one that only advances it.
4. **Dependencies are respected, not worked around.** An issue that needs a decision record first
   (#81 → #82, #68 → #69/#70, #96/#120 → #97) waits, even if it looks easy. The `Dependency / block`
   column is empty only where there genuinely is none.
5. **Nothing is closed because it "sounds done".** Every closure in the later phases cites the same kind
   of pointer the triage required: a commit, a file:line, a test name, a command output or an evidence
   file. `LEDGER.md` records it.
6. **The post-v0.1 roadmap stays post-v0.1.** 50 issues are listed in batch 5 purely so that no later
   run mistakes them for v0.1 gaps. Working them is a product decision, not a triage outcome.

## How to read a batch

- `ACTIONABLE_NOW` rows carry the **concrete change and the files** in the row itself, so a phase-2 run
  can start without re-analysing. The full text is in `INVENTORY.md`.
- `PARTIAL_ON_MAIN` rows carry **what is missing**; the batch they sit in reflects whether the missing
  part is blocked (batch 3/4) or not (batch 2).
- Estimates are deliberately coarse: **S** ≤ half a day, **M** ≈ 1–2 days, **L** ≈ 3–5 days,
  **XL** ≈ a week or more, all for one focused agent run with the existing gates.

## Definition of done for one issue (applies to every batch)

1. The change is on `main` in one or more commits with the pathspec confined to the issue's scope.
2. The repository's own gates are green on the touched crates — `cargo fmt --all --check`,
   `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace` (per `AGENTS.md:155-166` and the release pre-flight job).
3. If the issue is about behaviour that cannot be proven by tests here, the row says so and the claim is
   *not* made (the pattern this repository already uses: `BUILD_VERIFIED` ≠ `RUNTIME_VERIFIED`).
4. `LEDGER.md` gets the commit SHA, timestamp and the verification command/result **before** the issue is
   closed.
5. If the issue's own acceptance criteria cannot all be met in this environment, the issue stays open with
   the remainder written into `LEDGER.md` and, where it is release-facing, into
   `docs/release/known-limitations.md` or `docs/release/risk-register.md`.

## What this plan refuses to do

- Close an issue whose acceptance list is only partly satisfied (#56's fix does not close #52 or #21).
- Mark a runtime claim from source evidence (the V01-01…V01-05 history in `docs/plans/STATUS.md:7-16`
  exists precisely because that happened once).
- Treat a document that claims something as evidence that it is true — the keyring fallback sentence at
  `docs/architecture/security-boundaries.md:38` is contradicted by `crates/runtime/src/lib.rs:73-75`, and
  the plan treats the code as authoritative.
- Start Phase A–H work, or any issue whose own body defers it past v0.1.
## Batch 0 — confirm and close (`DONE_ON_MAIN`)

These ten issues have a verified pointer already. The phase-2 confirmation pass re-opens each pointer on the then-current `main`, and closes the issue with the pointer pasted into the closing comment. This must happen **before** new work starts, because closing them first shrinks the board and removes the chance that a later change invalidates the pointer.

| Order | # | Title | Estimate | Evidence to re-verify |
|---:|---:|---|---|---|
| 1 | #55 | [Gate 5][B1] Implement exact numeric/integer PostgreSQL decoder paths | S | crates/infrastructure/src/postgres/query_mapper.rs:336-360,514-584; tests :672,:689; crates/core/src/domain/query.rs:305; docs/release/evidence/v01-runtime/providers/07-pg-integration-live.txt:24 |
| 2 | #74 | [RC1][P2-0] Freeze post-P1 audit baseline and consolidate latest findings inventory | S | docs/release/evidence/v01-06/06-rc1-p2-dispositions.md:3 (audited at HEAD 3e0b077), :13-50 (25 confirmed, disposition table) |
| 3 | #83 | [RC1][Freeze A1] Select candidate main SHA and lock release-evidence input | S | docs/release/0.1.0-readiness.md:4-6; docs/release/evidence/v01-06/09-toolchain-pinning.txt; rust-toolchain.toml |
| 4 | #85 | [RC1][Freeze B2] Run complete Rust workspace release gates on candidate SHA | S | docs/release/evidence/v01-06/08-post-fix-quality-gates.txt; docs/release/evidence/v01-06/12-state-dir-blocker-fix.txt (815 passed / 0 failed / 19 ignored) |
| 5 | #86 | [RC1][Freeze B3] Run PostgreSQL and SQLite integration suites on candidate SHA | S | docs/release/evidence/v01-runtime/providers/07-pg-integration-live.txt:1-33; providers/06-postgres-container-and-baseline.txt |
| 6 | #87 | [RC1][Freeze C1] Build release artifact matrix from exact candidate SHA | S | docs/release/risk-register.md:3-10 (run 34860902181, all nine jobs green); docs/release/0.1.0-packaging.md:19-30 |
| 7 | #90 | [RC1][Smoke A1] Prepare packaged-runtime smoke fixtures, credentials, and evidence worksheet | S | fixtures/smoke/README.md:1-40; docs/release/0.1.0-manual-smoke.md; docs/release/0.1.0-interactive-verification-runbook.md |
| 8 | #132 | [RC1][Capability] Build PostgreSQL vs SQLite v0.1 capability/support matrix from source | S | docs/release/provider-capability-matrix.md:10 ('Issue: #132'), :14-33 (corrections + legend) |
| 9 | #133 | [RC1][Smoke Prep] Create deterministic PostgreSQL/SQLite packaged-smoke fixture datasets | S | fixtures/smoke/README.md:1-40; fixtures/smoke/large-er/large_er_fixture.sql; fixtures/smoke/verify-smoke.sh |
| 10 | #135 | [RC1][Truth] Build canonical known-limitations and non-goals registry for v0.1 | S | docs/release/known-limitations.md:16 ('Issue: #135'), :35-287 (registry), :289-300 (summary by status) |

Independent of each other; all ten can go in one pass. Total: **10 confirmations, ~10 × 10 minutes.**

## Batch 1 - P1 security / data-safety blockers that are bounded and verifiable here

The four P1 issues the RC1 audits raised against the shipping code. They come first because each one is either a claim the release documents already make, or a safety property the release explicitly leaves open - and because they are the only issues a *code* change can close in this environment. Three of the four also have a cross-platform or packaged-runtime leg that stays open; the rows say which.

| Order | # | Title | Disp. | Est. | Why here | Dependency / block |
|---:|---:|---|---|---|---|---|
| 1 | #142 | [P1][RC1][Security] Configure real OS keyring stores and remove DEV-only fallback from production | `ACTIONABLE_NOW` | M | the DEV-only encrypted-file fallback is wired into production - crates/runtime/src/lib.rs:73-75 unconditionally calls .with_fallback(), and keyring_vault.rs reads the fallback file *before* the OS keyring and writes it first, with the encryption key derived from the non-secret service name. Remov… | — |
| 2 | #144 | [P1][RC1][Security] Make PostgreSQL remote connections secure by default | `ACTIONABLE_NOW` | M | make non-local PostgreSQL connections secure by default. Today SslMode::Disable is the #[default] (connection.rs:48) and is what the UI draft and every fixture use, so remote credentials travel in plaintext unless the user changes a dropdown that carries no warning. Pick a safer default (e.g. Req… | — |
| 3 | #147 | [P1][RC1][Query] Decide and enforce multi-statement partial-write semantics | `ACTIONABLE_NOW` | M | the issue's premise is stale in the safe direction - since the core-safety-hardening work, a multi-statement batch that contains ANY mutation runs in one transaction on one connection and rolls back on failure, reporting 'transaction rolled back' or 'final outcome is unknown' (query_service.rs:21… | — |
| 4 | #145 | [P1][RC1][Data Safety] Make SQLite backup and restore snapshot-safe | `ACTIONABLE_NOW` | L | SQLite backup uses VACUUM INTO (a real snapshot mechanism, so the issue's 'raw fs::copy' premise is stale) and restore already stages + quick_checks + renames and refuses while the connection is active - but the claim is not yet defensible: no WAL checkpoint or idle check on the source, no fsync/… | — |

_4 issues · estimates: M=3, L=1._

## Batch 2 - small, real, self-contained fixes and contract documents

Bounded changes and contract documents with no owner decision, no external resource and no unfinished dependency. Ordered so that the two provider-value UI fixes (#56, #61) land before the policy row that depends on them (#62).

| Order | # | Title | Disp. | Est. | Why here | Dependency / block |
|---:|---:|---|---|---|---|---|
| 1 | #56 | [Gate 5][B2] Implement PostgreSQL temporal decoder paths without timezone invention | `ACTIONABLE_NOW` | S | TIMESTAMP WITHOUT TIME ZONE is decoded with `v.and_utc().to_rfc3339()`, which invents a UTC offset that the issue explicitly forbids. Emit the naive canonical text (YYYY-MM-DDTHH:MM:SS.ffffff) for TIMESTAMP and keep the Z form for TIMESTAMPTZ; add a decoder test. Files: crates/infrastructure/src/… | — |
| 2 | #25 | [Gate 5][Workstream D] End-to-end verification and integration | `PARTIAL_ON_MAIN` | M | Provider->domain half proven live (18/18). The Tauri-DTO->frontend half is void: the React frontend was retired 2026-09-11 and the shipping IPC is an in-process channel (no serde DTO), so #65 is superseded and no native-UI E2E matrix exists. | — |
| 3 | #52 | [Gate 5][A2] Define DATE/TIME/TIMETZ/TIMESTAMP/TIMESTAMPTZ semantics | `PARTIAL_ON_MAIN` | M | The five-variant temporal contract in the issue is only partly implemented: DATE/TIME/TIMETZ/TIMESTAMPTZ have decode branches but TIMETZ shares CellValue::Time with TIME and TIMESTAMP shares CellValue::DateTime with TIMESTAMPTZ, so the class distinction is not recoverable from the value; TIMESTAM… | — |
| 4 | #61 | [Gate 5][C2] Render/copy/export canonical provider values without precision or timezone loss | `ACTIONABLE_NOW` | M | two lossy UI paths. (1) Copy-as-JSON parses Decimal/Int64 text into f64 when it happens to parse, so 12345678901234567890.12345 leaves the clipboard as an IEEE-754 number - emit a JSON string instead, as the core ExportService already does (crates/core/src/application/export_service.rs:216). (2) … | — |
| 5 | #21 | [EPIC][P1][Gate 5] PostgreSQL lossless provider-value contract and end-to-end policy | `PARTIAL_ON_MAIN` | L | Decoder implementation + live provider proof landed, but the gate's own non-negotiables are unmet: TIMESTAMP gains an invented UTC offset (crates/infrastructure/src/postgres/query_mapper.rs:369-372) and the enum/domain/array fallback contract was never locked (#53). Close only after #52/#53/#56 a… | — |
| 6 | #22 | [Gate 5][Workstream A] Canonical provider-value contract | `PARTIAL_ON_MAIN` | L | A1 (#51) closed at 35570b2f; A2 partial (#52), A3 not started (#53), A4 partially satisfied by the tagged serde enums plus the legacy tauri DTO. No single contract document in the tree; the A1/A2 conventions live only in the issue bodies. | — |
| 7 | #23 | [Gate 5][Workstream B] PostgreSQL result decoder implementation | `PARTIAL_ON_MAIN` | L | B1-B3 implemented for numeric/temporal/json/uuid/bytea/network/interval; B4 enum/domain/array/custom are a text fallback (decode_textual_value) with no element parsing; arrays over the binary result format are unverified. | — |
| 8 | #24 | [Gate 5][Workstream C] Frontend provider-value policy | `PARTIAL_ON_MAIN` | L | UI consumes canonical text exactly, but collapses every type class into UiCell::Text (type identity lost), copy-as-JSON is lossy, and there is no per-value-class editability policy. Tracked concretely by #61/#62. | — |
| 9 | #134 | [RC1][Platform] Inventory packaged-runtime native prerequisites and OS-specific dependency risks | `PARTIAL_ON_MAIN` | S | The document exists and carries a native amendment, but its platform matrix still describes the retired Tauri bundler (dmg/MSI/NSIS/deb/rpm/AppImage, WebView2, WebKitGTK, libsecret) while v0.1 ships portable archives. The current contract lives in 0.1.0-packaging.md. Concrete change: reconcile on… | — |
| 10 | #62 | [Gate 5][C3] Enforce type-aware editability and safe mutation policy | `ACTIONABLE_NOW` | M | write the value-class -> editable/read-only table and enforce it before staging. Today editing is gated only by connection readonly + table PK (crates/ui/src/table_editor_view.rs:1449-1460) while typing is validated per declared column type (:785-877, :926-956); binary columns are rejected, but a… | — |
| 11 | #72 | [RC1][Introspection B1] Add explicit introspection IPC serialization contract tests | `ACTIONABLE_NOW` | M | add serialized-shape contract tests for the introspection payload on the shipping path - composite FK column order, exact camelCase/snake_case field names, null/optional fields - so the old singular fromColumn/toColumn drift cannot return. Files: crates/native-app/src/translate.rs:804 (+ tests in… | — |
| 12 | #102 | [RC1][Brand C1] Inventory every public-facing rename surface before code changes | `ACTIONABLE_NOW` | M | build the rename-surface inventory now (it does not need the new name): search patterns for 'DB Pro', 'db-pro', 'db_pro', com.dbpro.app, bundle/identifier fields, artifact names, docs/screenshots, workflow artifact names, and the persisted-identifier classes. Files: new doc under docs/ (or a sect… | — |
| 13 | #121 | [RC1][Brand Prep] Inventory persisted storage/config identifiers for rename compatibility | `ACTIONABLE_NOW` | M | inventory every persisted identifier and classify it (public-display / persisted-compatibility / migrate-with-fallback / internal-tech-id). Sources to search: the OS keyring service name (com.dbpro.app, crates/runtime/src/lib.rs:73), the app state directory resolver, the meta store schema_version… | — |
| 14 | #127 | [RC1][Release Prep] Audit app versioning, updater configuration, and persisted-state compatibility across upgrades | `ACTIONABLE_NOW` | M | write the version/updater/persistence decision record - update delivery model (none/manual), version source of truth (crates/native-app/Cargo.toml, read by release.yml), persisted-state compatibility policy (meta store schema_version = 2; no workspace persistence at all per LIM-016), and the bund… | — |
| 15 | #114 | [META][Agents] Standardize evidence, progress, and review log contract | `ACTIONABLE_NOW` | S | add the claim / progress / review-request / review-outcome templates to AGENTS.md (they currently exist only inside the issue body), including the mandatory exact-SHA rule and the 'skipped gate is never a pass' rule. Files: AGENTS.md (extend the existing 'Working model' / 'Runtime verification' /… | — |

_15 issues · estimates: S=3, M=8, L=4._

## Batch 3 - evidence completion, disposition records and decisions that other work waits on

Work that needs another batch's output, an owner answer or a decision record before it can start — plus the evidence records that make a release claim defensible. Nothing here is blocked by a host, so it is the correct queue once batches 0–2 are done and report outcomes.

| Order | # | Title | Disp. | Est. | Why here | Dependency / block |
|---:|---:|---|---|---|---|---|
| 1 | #14 | [GOAL][v0.1.0] Ship a trustworthy, verified desktop Database IDE | `PARTIAL_ON_MAIN` | XL | Goal DoD 1-4,6,7 have evidence; DoD 5 (public identity) and DoD 8 (publish + reconcile) do not. Blocked by the owner decisions HD-001 (license) and HD-002/HD-007; no v0.1.0 tag exists. | — |
| 2 | #68 | [RC1][Introspection A1] Decide CHECK constraint v0.1 disposition from current source and consumer value | `NEEDS_OWNER_DECISION` | S | Owner decision: CHECK shipped end-to-end on both providers, so the practical answer is KEEP - but the release-truth documents still say the disposition is pending, and closing it changes release scope wording. Needs an explicit KEEP/DEFER record before #69/#70 or #71. | — |
| 3 | #82 | [RC1][P2-H] Verify completed P2 dispositions on exact post-remediation main SHA | `PARTIAL_ON_MAIN` | S | Not started: the post-remediation verification (P0=0, P1=0, every P2 dispositioned, no open Fix ticket) was never recorded on a final SHA. Blocked by #81; STATUS.md records the P2 gate checkbox as formally unchecked. | — |
| 4 | #88 | [RC1][Freeze C2] Generate candidate evidence manifest and artifact checksums | `PARTIAL_ON_MAIN` | S | Checksums and provenance exist as CI artifacts and the values are recorded in the release docs, but there is no single in-repo evidence manifest file tying every gate/artifact to the candidate SHA. | — |
| 5 | #89 | [RC1][Freeze D1] Final latest-head review and declare FINAL_RC_SHA | `PARTIAL_ON_MAIN` | S | No `FINAL_RC_SHA` is declared anywhere in the tree (grep returns nothing); the docs call 85a7fa3 the candidate. The final latest-head review step was therefore never executed as written. | — |
| 6 | #101 | [RC1][Brand B4] Select and lock replacement public identity | `NEEDS_OWNER_DECISION` | S | The public identity is explicitly reserved to the owner; it is also a precondition for #28/#103/#104/#105/#106/#108. | — |
| 7 | #26 | [RC1][Workstream] Introspection consistency and IPC contract hardening | `PARTIAL_ON_MAIN` | M | CHECK is wired end-to-end on both providers and parser tests exist, but the #68 KEEP/DEFER decision was never recorded (LIM-011 still says 'pending') and the #72 IPC contract tests were never written. Gated on #68. | — |
| 8 | #81 | [RC1][P2-G] Synthesize Fix/Accept/Defer table and spawn only required RC1 fix tickets | `PARTIAL_ON_MAIN` | M | A disposition table exists (24 OBSOLETE / 1 DEFERRED / 20 CARRIED_OVER_UNVERIFIED, none promoted) but its rows were never copied into #27, and no Fix RC1 child tickets were created. Needs the table to be declared final and the registry updated before #82. | — |
| 9 | #97 | [RC1][Brand A2] Define product wedge and v0.1 vs long-term positioning statement | `NEEDS_OWNER_DECISION` | M | Positioning is a product decision owned by the repository owner; the dependency #96/#141 is also unfinished. | — |
| 10 | #104 | [RC1][Brand C3] Verify rename completeness, compatibility, and release-facing metadata | `NEEDS_OWNER_DECISION` | M | Verification of a rename that has not happened; blocked by #103. | — |
| 11 | #126 | [RC1][Trust] Audit keyring and encrypted-fallback secret lifecycle | `PARTIAL_ON_MAIN` | M | Evidence exists for the stall (reproduced and classified P2, unbounded keyring read with no timeout) and for the fallback storage design, but the required event -> state lifecycle matrix does not exist, and the production fallback defect it feeds (#142) is still open. | — |
| 12 | #128 | [RC1][Data Safety] Audit export/import/backup integrity, cancellation, and partial-failure semantics | `PARTIAL_ON_MAIN` | M | Backup/restore was verified at CLI level with finding F1 and the export/import/backup classification exists in the capability matrix, but the cancellation/partial-file semantics are still undocumented (the native cancel trigger RuntimeCommand::CancelOperation is never constructed, so backup canno… | — |
| 13 | #129 | [RC1][Data Safety] Audit destructive SQL, multi-statement execution, cancellation, and confirmation boundaries | `PARTIAL_ON_MAIN` | M | The classifier and the batch atomicity work landed with tests (34 safety + 9 splitter tests) and #147 was spawned, but the required execution-safety matrix was never recorded and one source-level gap survives: the Agent tool path classifies the whole string by leading keyword (crates/core/src/dom… | — |
| 14 | #136 | [META][RC1] Central release risk, decision, and blocker register | `PARTIAL_ON_MAIN` | M | The register exists with RISK/DECISION record formats, a severity policy and a candidate-invalidation log, and is explicitly designed to stay open through v0.1. It cannot close until P0=0, P1=0 and every P2 has a final disposition - currently false. | — |
| 15 | #27 | [RC1][Workstream] P2 audit, disposition, targeted remediation, and closure | `PARTIAL_ON_MAIN` | L | P2 dispositions for the 25 RC1 findings exist and no P2 was promoted to P0/P1, but the area audits #75-#80 have no separate record, #81 spawned no Fix RC1 ticket, and #82 never verified the table on a final SHA. | — |
| 16 | #28 | [RC1][Workstream] Exact-SHA candidate freeze, automated gates, artifacts, and evidence | `PARTIAL_ON_MAIN` | L | Candidate 85a7fa3 is selected with an all-green pipeline and verified artifacts, but the freeze chain still lacks the frontend gates (#84 superseded - frontend retired), a single in-repo evidence manifest (#88) and the formal FINAL_RC_SHA declaration (#89). | — |
| 17 | #103 | [RC1][Brand C2] Apply selected public identity in one controlled rename PR | `NEEDS_OWNER_DECISION` | L | Blocked by the owner's name selection (#101) and must land before candidate freeze; a rename touches every public surface plus persisted identifiers. | — |
| 18 | #131 | [RC1][Traceability] Map every release-critical capability to tests, CI jobs, runtime smoke, and evidence owner | `PARTIAL_ON_MAIN` | L | docs/release/0.1.0-goal-3-traceability.md maps goal-3 requirements to implementation/evidence/status, which is a different axis from the capability -> test -> CI job -> smoke row -> evidence owner matrix this issue asks for. No capability-level traceability matrix exists. | — |
| 19 | #30 | [RC1][Workstream] Resolve public identity and complete controlled product rename | `PARTIAL_ON_MAIN` | XL | Research shards #137-#140 are closed and the repo records the identity as an open blocker, but no positioning, shortlist, selection or rename landed. Entire downstream chain #97-#104 is blocked on the owner's naming choice. | — |
| 20 | #67 | [Gate 5][D4] Merge verified Gate 5 PR and prove main contains the exact accepted type matrix | `PARTIAL_ON_MAIN` | S | The decoder substance is on main, but the issue's own acceptance (record the Gate 5 merge SHA, prove main carries the accepted matrix, then state P1=0) was never executed as a recorded step. | — |
| 21 | #69 | [RC1][Introspection A2-KEEP] Harden SQLite CHECK parser semantics and focused tests | `PARTIAL_ON_MAIN` | S | Conditional on #68 = KEEP. The SQLite parser is already lexical rather than regex: case-insensitive keyword match, quote/comment skipping, nested-paren depth, doubled-quote escapes, safe stop on malformed SQL, two focused tests. Missing: preserving a `CONSTRAINT <name>` instead of synthesising `<… | — |
| 22 | #73 | [RC1][Introspection C1] Reconcile audit/docs and run exact-head introspection regression verification | `PARTIAL_ON_MAIN` | S | The audit document, the audit/checklist corrections in STATUS.md and the live provider suites exist; what is missing is the single exact-SHA introspection regression record plus the recorded CHECK disposition. Gated on #68/#72. | — |
| 23 | #98 | [RC1][Brand B1] Define naming criteria and rejection rules from product positioning | `NEEDS_OWNER_DECISION` | S | Naming criteria derive from the positioning the owner has not chosen; writing the rubric without it would bake in an unapproved wedge. | — |
| 24 | #54 | [Gate 5][A4] Lock Rust/Tauri/frontend provider-value DTO serialization contract | `PARTIAL_ON_MAIN` | M | Core CellValue and the legacy tauri CellValueDto are both tagged enums with matching lowercase renames and int64-as-string tests, but there is no whole-result fixture test and no Rust<->UI field-name assertion for the shipping (non-serde) channel. | — |
| 25 | #57 | [Gate 5][B3] Implement JSON/JSONB/UUID/BYTEA/network decoder paths | `PARTIAL_ON_MAIN` | M | All five classes have explicit decode branches, but decode_cell has no unit test (no PgRow is constructible without a live server) and the live suite asserts only temporal/network/numeric; JSON numbers beyond i64 route through serde_json's f64 model. | — |
| 26 | #59 | [Gate 5][B5] Add PostgreSQL integration fixture covering full decoder matrix | `PARTIAL_ON_MAIN` | M | fixtures/postgres + the 18 pg_integration cases cover bool/int/float/numeric/temporal/interval/uuid/json/bytea/inet/enum/nulls, but not domain and not array element decoding. | — |
| 27 | #64 | [Gate 5][D1] Verify PostgreSQL fixture -> domain decoder -> Tauri DTO end-to-end | `PARTIAL_ON_MAIN` | M | provider -> domain is proven end to end on live PostgreSQL 16.15; the 'domain -> Tauri DTO -> emitted JSON' half targets the retired Tauri host, and no serialized whole-result fixture is recorded. | — |
| 28 | #66 | [Gate 5][D3] Run exact-head Rust/frontend/provider verification for type matrix | `PARTIAL_ON_MAIN` | M | Rust and provider gates are recorded for the candidate (six gates green on rustc 1.95.0; live PG 18/18), but no Gate-5-specific type-matrix verification record exists and the frontend half is void. | — |
| 29 | #70 | [RC1][Introspection A3-KEEP] Complete CHECK provider -> domain -> Tauri -> frontend exposure | `PARTIAL_ON_MAIN` | M | Conditional on #68 = KEEP. The exposure already runs provider -> domain -> native -> UI (Constraints tab renders Check constraints), so the issue's 'backend-only partial metadata' premise no longer matches main. Missing: PG expression normalisation parity, serialization-shape tests, and a determi… | — |
| 30 | #75 | [RC1][P2-A] Audit Workspace/Shell/navigation findings on post-P1 baseline | `PARTIAL_ON_MAIN` | M | The 25 findings got a per-finding disposition and a native carry-over verdict, but no Workspace/Shell-specific audit record exists and the native rows are CARRIED_OVER_UNVERIFIED. | — |
| 31 | #76 | [RC1][P2-B] Audit Data Grid/read-update-delete findings on post-P1 baseline | `PARTIAL_ON_MAIN` | M | Same as #75 for the Data Grid area: dispositions exist at finding level, no area audit record, native rows unverified. | — |
| 32 | #77 | [RC1][P2-C] Audit connection/session/SSH-related findings on post-P1 baseline | `PARTIAL_ON_MAIN` | M | Same for connection/session/SSH; the SSH rows stay blocked on the DB_PRO_SSH_* fixture (nine variables unset). | — |
| 33 | #78 | [RC1][P2-D] Audit Query/ER residual findings after Gate 4/5 | `PARTIAL_ON_MAIN` | M | Same for Query/ER residuals; Gate 4 evidence is recorded separately (docs/release/evidence/v01-06/13-flaky-er-worker-test.txt) but no residual Query/ER P2 audit exists. | — |
| 34 | #79 | [RC1][P2-E] Audit performance tooling, bundle evidence, and release-measurement semantics | `PARTIAL_ON_MAIN` | M | Perf/tooling semantics were partly corrected (perf scan re-run in the release gates) but no P2-E audit record exists; the stale .skills/perf-audit paths are not audited here. | — |
| 35 | #80 | [RC1][P2-F] Audit cross-cutting i18n/accessibility/error-state release polish | `PARTIAL_ON_MAIN` | M | No i18n/accessibility audit record exists. Known-live echo: the workspace/session persistence gap (LIM-016) and the untranslated-string surface are not dispositioned as a set. | — |
| 36 | #96 | [RC1][Brand A1] Competitive landscape and parity evidence workstream | `PARTIAL_ON_MAIN` | M | Research shards #137-#140 are closed and #141 is the pending synthesis; #120's capability inventory exists in a different shape. The workstream cannot close before #141 and #120 do. | — |
| 37 | #99 | [RC1][Brand B2] Generate and score replacement-name shortlist | `NEEDS_OWNER_DECISION` | M | A shortlist is only meaningful against the owner's positioning (#97) and exists to feed the owner's selection (#101). | — |
| 38 | #100 | [RC1][Brand B3] Run live collision/domain/product diligence on naming shortlist | `NEEDS_EXTERNAL_RESOURCE` | M | Requires live, current-source diligence (web/product/domain/package/trademark search per candidate) that cannot be derived from the repository. | — |
| 39 | #120 | [RC1][Brand Prep] Inventory source-backed product differentiators and parity evidence | `PARTIAL_ON_MAIN` | M | docs/notes/PRODUCT_CAPABILITY_MATRIX.md is a source-backed capability inventory with per-row evidence and release vocabulary, but it does not cover MCP state or the Agent/Action-Platform boundary explicitly, and its status vocabulary differs from the issue's. | — |
| 40 | #141 | [RC1][Brand A1.5] Synthesize competitor research into parity, gap, and positioning evidence map | `PARTIAL_ON_MAIN` | M | The synthesis is blocked on #120 (capability inventory in the required shape) while its other four inputs (#137-#140) are closed; no synthesis document exists in the tree. | — |
| 41 | #53 | [Gate 5][A3] Define INTERVAL/network/enum/domain/array/custom-type fallback contract | `PARTIAL_ON_MAIN` | L | INTERVAL/INET/CIDR are implemented; enum/domain/array/custom fall back to raw text with no recorded contract and no editability metadata. No per-class contract matrix exists in the tree. | — |
| 42 | #58 | [Gate 5][B4] Implement enum/domain/array/INTERVAL/custom-type safe decoding | `PARTIAL_ON_MAIN` | L | INTERVAL implemented to microsecond precision. enum/domain/array/custom take the textual fallback with no domain arm and no fixture; `CREATE DOMAIN` appears in no fixture, so the domain path is untested. Arrays have no element parsing. | — |
| 43 | #122 | [RC1][Trust] Audit local-first security boundaries, credential handling, and unsafe execution paths | `PARTIAL_ON_MAIN` | L | The downstream findings were spawned (#142/#144/#145/#146/#147) and risk-register entries exist, but there is no trust-boundary audit record with the required severity table, and docs/architecture/security-boundaries.md:38 still claims the encrypted fallback is 'dev/CI only, disabled in productio… | — |
| 44 | #71 | [RC1][Introspection A2-DEFER] Remove/deactivate partial CHECK exposure cleanly for v0.1 | `NEEDS_OWNER_DECISION` | S | Owner decision, and only reachable if #68 resolves to DEFER. There is no longer any 'partial backend-only' exposure to remove - deferring would mean removing a shipped, tested feature, which is a scope reduction the owner must authorise. | — |

_44 issues · estimates: S=10, M=25, L=7, XL=2._

## Batch 4 - needs the owner or an external resource (hosts, GUI, GitHub settings, publication)

Genuinely blocked: needs the owner, a host, a GUI session or a publication that does not exist yet. Do not start these; collect the answers instead (see `docs/release/0.1.0-human-decisions.md` for the questions already written down).

| Order | # | Title | Disp. | Est. | Why here | Dependency / block |
|---:|---:|---|---|---|---|---|
| 1 | #119 | [RC1][Prep] Decide license, redistribution, and public release policy | `NEEDS_OWNER_DECISION` | S | Owner decision and the binding blocker for public distribution: no LICENSE file exists and no manifest carries license metadata, so the archives must not be published. Recorded as HD-001 with the options and no recommendation. | — |
| 2 | #106 | [v0.1.0][Release A2] Verify version, identity, package metadata, and artifact naming before tag | `PARTIAL_ON_MAIN` | S | Version 0.1.0 is consistent across the manifest the workflow reads and the release docs, but the identity half of the acceptance cannot be met before #101/#104. | — |
| 3 | #107 | [v0.1.0][Release B1] Final pre-tag consistency check against FINAL_RC_SHA | `PARTIAL_ON_MAIN` | S | The GO/NO-GO is recorded as NO for public distribution with named blockers, but the pre-tag check cannot pass until #95/#105/#106 close and HD-001 is decided. | — |
| 4 | #108 | [v0.1.0][Release B2] Create v0.1.0 tag and GitHub Release on accepted SHA | `NEEDS_OWNER_DECISION` | S | Tagging is explicitly the owner's decision (HD-008), and public artifacts are blocked by HD-001 (no license). No tag exists. | — |
| 5 | #109 | [v0.1.0][Release B3] Attach and validate release assets against evidence manifest | `NEEDS_OWNER_DECISION` | S | Cannot attach assets to a release that does not exist; blocked by #108. | — |
| 6 | #110 | [v0.1.0][Release C1] Update README/release docs to match published v0.1.0 reality | `PARTIAL_ON_MAIN` | S | README and the release docs are already reconciled to the RC reality ('Available now (0.1.0)' vs 'Roadmap (not in 0.1.0)', limitations, governance). What remains is the post-publication pass, blocked by #108. | — |
| 7 | #146 | [P1][RC1][Governance] Protect main and require release CI checks | `NEEDS_OWNER_DECISION` | S | Owner/governance action, not a code task: configure main branch protection + rulesets and the required checks. Note the check names in the issue ('Frontend checks') no longer exist - the only CI job is 'Rust checks', so the required-check list must be rewritten before it can be applied. Also bloc… | — |
| 8 | #91 | [RC1][Smoke B1] Run packaged macOS runtime smoke on FINAL_RC_SHA artifact | `PARTIAL_ON_MAIN` | M | Non-interactive macOS qualification is done on the CI artifact (extract, launch, state-directory branches, no window). The interactive smoke rows (Explorer, query, grid edit, ER, export, quit/relaunch) were never executed: 0 of 165 rows passed. Needs a GUI session (HD-007). | — |
| 9 | #95 | [RC1][Smoke D1] Reconcile platform results and sign off packaged runtime | `NEEDS_EXTERNAL_RESOURCE` | M | Cannot sign off a platform matrix that has no Windows/Linux runtime rows and no interactive macOS rows; the reconciliation document is 0.1.0-final-report.md plus readiness, but the sign-off condition itself needs external hosts. | — |
| 10 | #105 | [v0.1.0][Release A1] Draft release notes from verified shipped scope and limitations | `PARTIAL_ON_MAIN` | M | Release notes exist and are honest about unqualified areas, but two required inputs are missing: the selected public identity (the notes still say DB Pro) and the final platform matrix from #95. Also note LIM-012 advertises XLSX export the UI cannot reach. | — |
| 11 | #111 | [v0.1.0][Release C2] Post-publish verification and close release Goal | `NEEDS_OWNER_DECISION` | M | Post-publish verification and Goal closure cannot start before a publication exists; blocked by #108-#110 and HD-001/HD-007. | — |
| 12 | #29 | [RC1][Workstream] Packaged desktop runtime qualification | `PARTIAL_ON_MAIN` | L | macOS packaged artifact was launched and its state directory re-verified; Windows/Linux runtime and the interactive GUI smoke are unavailable here (#92/#93/#95). Platform-reduction decision is HD-003. | — |
| 13 | #31 | [v0.1.0][Workstream] Publish verified release and close Goal | `PARTIAL_ON_MAIN` | L | Release notes, README and handoff are ready for an RC, but nothing is published: no tag, no GitHub Release, no assets. Blocked by #108 which is blocked by HD-001 (license). | — |
| 14 | #92 | [RC1][Smoke B2] Run packaged Windows runtime smoke on FINAL_RC_SHA artifact | `NEEDS_EXTERNAL_RESOURCE` | L | No Windows host exists in this project; the artifact is BUILD_VERIFIED / PACKAGE_VERIFIED only and no process has been launched from it. | — |
| 15 | #93 | [RC1][Smoke B3] Run packaged Linux runtime smoke on FINAL_RC_SHA artifact | `NEEDS_EXTERNAL_RESOURCE` | L | No Linux host exists; additionally the packaged Linux runtime needs a D-Bus Secret Service provider for credentials (B-7). | — |
| 16 | #94 | [RC1][Smoke C1] Triage packaged-runtime failures and enforce re-freeze loop | `PARTIAL_ON_MAIN` | M | The re-freeze loop was genuinely exercised (state-directory blocker fixed in 543b526 and re-qualified; F1 restore skew triaged as P2; keyring stall classified P2), but no failed smoke row exists to triage because no smoke rows ran. | — |
| 17 | #112 | [META][Project] Configure GitHub Projects control plane for v0.1 execution | `NEEDS_EXTERNAL_RESOURCE` | M | Explicitly blocked by tooling: GitHub Projects v2 mutation is not available through the connector used, and the Project must be created through the GitHub UI/API by a human. | — |

_17 issues · estimates: S=7, M=6, L=4._

## Batch 5 — not work (`OUT_OF_SCOPE_V01`, 50 issues) — listed so nobody re-triages them

| # | Title | Why it is here |
|---:|---|---|
| #32 | [Backlog][v0.5] MCP and external agent ecosystem | Explicit v0.5 backlog tier; its own body says 'Do not start before the internal Agent/Action Platform execution model is production-qualified'. |
| #33 | [Backlog][v0.3] Database administration and distribution | Explicit v0.3 backlog tier ('Do not start before v0.1 and v0.2 release goals are complete'). |
| #34 | [Backlog][v0.4] Agent-native database operations | Explicit v0.4 backlog tier (Agent graduation). |
| #35 | [Backlog][v0.2] Complete row insertion | Explicit v0.2 backlog tier; the v0.1 subset (staged insert) already ships and is documented as narrower than a full workflow. |
| #182 | [GOAL][Post-v0.1] Expand DB Pro into a DBeaver-class, AI-native database IDE | Explicitly the post-v0.1 product goal; its own body says it starts after v0.1 and all nine authority documents exist under docs/goals/. |
| #183 | [Phase A][A01] Typed Object Mutation Framework | Phase A post-v0.1 scope (docs/notes/V0_1_CLOSURE_PLAN.md §1: 'Strict zero-feature-expansion policy ... no Phase A-H feature work is admitted into the v0.1.0 release queue'). |
| #184 | [Phase A][A02] Views and Materialized Views CRUD Workbench | Phase A post-v0.1 scope; depends on #183. |
| #185 | [Phase A][A03] Index CRUD Wizard | Phase A post-v0.1 scope; depends on #183. |
| #186 | [Phase A][A04] Constraint CRUD — PK, FK, Unique, Check | Phase A post-v0.1 scope; depends on #183. Note it overlaps the v0.1 CHECK disposition (#68) for constraint CRUD. |
| #187 | [Phase A][A05] Trigger Editor and Lifecycle Management | Phase A post-v0.1 scope; depends on #183. |
| #188 | [Phase A][A06] PostgreSQL Sequence Management | Phase A post-v0.1 scope; depends on #183. |
| #189 | [Phase A][A07] PostgreSQL Types, Enums, and Domains Workbench | Phase A post-v0.1 scope; depends on #183. |
| #190 | [Phase A][A08] Schema and Database Management | Phase A post-v0.1 scope; depends on #183. |
| #191 | [Phase B][B01] Routine Domain Model and Explorer | Phase B post-v0.1 scope. |
| #192 | [Phase B][B02] Routine Source Editor, CRUD, and Execute/Call Workbench | Phase B post-v0.1 scope; depends on #191 and #183. |
| #193 | [Phase C][C01] Streaming Data Transfer Foundation | Phase C post-v0.1 scope. |
| #194 | [Phase C][C02] CSV Import and Export Workbench | Phase C post-v0.1 scope; depends on #193. |
| #195 | [Phase C][C03] JSON and Excel Transfer Formats | Phase C post-v0.1 scope; depends on #193/#194. |
| #196 | [Phase D][D01] Monitoring Foundation — Sessions, Active Queries, Cancel/Terminate | Phase D post-v0.1 scope. The v0.1 UI still shows a Monitoring placeholder, which is the intended v0.1 state. |
| #197 | [Phase D][D02] Locks, Transactions, Size/Stats, and Maintenance | Phase D post-v0.1 scope; depends on #196. |
| #198 | [Phase E][E01] Native Users, Roles, Memberships, and Privileges Workbench | Phase E post-v0.1 scope; users/roles administration is deferred in v0.1 (LIM-005). |
| #199 | [Phase F][F01] Schema Snapshot and Compare Engine | Phase F post-v0.1 scope. |
| #200 | [Phase F][F02] Migration Preview, SQL Generation, and Safe Apply | Phase F post-v0.1 scope; depends on #199 and #183. |
| #201 | [Phase G][G01] Global Search Activity and Unified Command Palette | Phase G post-v0.1 scope. |
| #202 | [Phase G][G02] Saved Queries, Snippets, Favorites, Recent, and Pinned Objects | Phase G post-v0.1 scope. |
| #203 | [Phase H][H01] Advanced AI Tools on Canonical Application Actions | Phase H post-v0.1 scope; the v0.1 Agent stays Preview (LIM-007) and reuses canonical actions only. |
| #204 | [Productivity][I01] Connection Folders, Tags, Favorites, Import/Export Profiles | Post-v0.1 productivity scope (connection folders/tags). The v0.1 gap is already registered: the UI drops folders/tags that the domain supports. |
| #205 | [Settings][K01] Complete Native Settings and Keybindings Surface | Post-v0.1 settings surface scope. |
| #206 | [Tasks][J01] Saved Database Tasks and Run History | Post-v0.1 task system scope. |
| #207 | [Phase A][A09] Table and Column CRUD Workbench | Phase A post-v0.1 scope; depends on #183. The v0.1 equivalent is the existing staged Table Data Editor and the documented schema-mutation gap (LIM-004). |
| #208 | [Phase C][C04] Backup and Restore Workbench | Phase C post-v0.1 scope. Its SQLite half is exactly the v0.1 defect tracked by #145, so #145 lands first. |
| #209 | [Phase C][C05] Database-to-Database Data Transfer | Phase C post-v0.1 scope; depends on #193/#194. |
| #210 | [Phase F][F03] Data Compare and Row-Diff Workbench | Phase F post-v0.1 scope; depends on #199. |
| #211 | [Phase G][G03] Queries Activity — Open, Saved, History, Snippets, Scratch | Phase G post-v0.1 scope. |
| #212 | [Phase G][G04] Data Activity — Recent, Pinned, and Reusable Dataset Navigation | Phase G post-v0.1 scope. |
| #213 | [Tasks][J02] Scheduled Database Tasks and Execution Policies | Post-v0.1 tasks scope; depends on #206. Any real scheduler conflicts with the 'no fake background scheduler' rule and is explicitly post-v0.1. |
| #214 | [Diagnostics][L01] Native Diagnostics, Logs, and Support Bundle | Post-v0.1 diagnostics scope. Related v0.1-era material exists (error-log audit, redaction rules in AGENTS.md) but the support-bundle surface is not v0.1 scope. |
| #215 | [Phase H][H02] Query Plan Visualizer and Slow Query Advisor | Phase H post-v0.1 scope; depends on #196 for live context. |
| #216 | [Phase A][A10] Object Dependencies and References Navigator | Phase A post-v0.1 scope; depends on the Phase A object models. |
| #217 | [Phase A][A11] PostgreSQL Partitioned Tables and Partition Management | Phase A post-v0.1 scope ('Parent Goal: #182'); partitioned-table management needs the #183 mutation framework. No v0.1 impact. |
| #218 | [Phase A][A12] PostgreSQL Extensions Workbench | Phase A post-v0.1 scope; extension management is schema mutation, deferred in v0.1 (LIM-004). |
| #219 | [Phase E][E02] PostgreSQL Row-Level Security Policies Workbench | Phase E post-v0.1 scope; the existing user/role backend is deliberately not surfaced in v0.1 (LIM-005). |
| #220 | [Phase C][C06] SQL INSERT, PostgreSQL COPY, and Script Export Formats | Phase C post-v0.1 scope. Note the overlap with v0.1 truth: the UI exports CSV/TSV only, while LIM-012 advertises XLSX (#61 tracks the UI gap). |
| #221 | [Phase D][D03] Transaction and Session State Inspector | Phase D post-v0.1 scope; depends on the #196 monitoring foundation. Related v0.1 truth: SQLite cancellation works, PostgreSQL cancellation is capability-gated off (LIM-014). |
| #222 | [Phase G][G05] Workspace Sessions, Layout Presets, and Window Restore | Phase G post-v0.1 scope. Related v0.1 truth: workspace/session persistence is not implemented at all (LIM-016) and is already registered as a limitation. |
| #223 | [Connections][I02] Advanced SSL/TLS, SSH Profiles, and Connection Diagnostics | Post-v0.1 scope, but it overlaps two open v0.1 items: the secure-by-default policy (#144) and the unqualified SSH status (LIM-006). Whatever #144 decides fixes the v0.1 baseline; this issue then extends it (CA/client certs, profiles, staged diagnostics). |
| #224 | [Query][Q01] Explicit Auto-Commit, Manual Transaction, Commit, and Rollback Controls | Post-v0.1 scope, but it is the natural home of the contract that #147 must record for v0.1: today a multi-statement batch containing any mutation already runs in one transaction and rolls back (query_service.rs:213-239), and there are no user-facing commit/rollback controls. |
| #225 | [Query][Q02] SQL Parameters, Variables, and Reusable Execution Bindings | Post-v0.1 scope. Related v0.1 truth: typed parameter binding exists at the provider layer (bind_params fails explicitly rather than coercing) but there is no user-facing parameter/variable system. |
| #226 | [ER][ER01] Schema Design Mode with Draft Table/Relation Editing | Post-v0.1 scope; depends on #183 plus the Phase A CRUD issues (#207, #186, #185). ER in v0.1 is inspection/navigation only. |
| #227 | [Data][DA01] Column Profiling, Distribution, Null, Distinct, and Quality Insights | Post-v0.1 scope (data profiling); depends on the Phase C/D services. No v0.1 impact. |

## Superseded set (7 issues) — close with a pointer, do not work

| # | Title | Replacement |
|---:|---|---|
| #60 | [Gate 5][C1] Align frontend provider-value DTO types with canonical Tauri contract | the native UI value types (crates/ui/src/runtime.rs:541-549 UiCell, crates/native-app/src/translate.rs:649-668) - alignment there is tracked by #61/#62. |
| #63 | [Gate 5][C4] Add frontend regression matrix for provider-value rendering/edit policy | Target layer retired (React frontend archived 2026-09-11). The surviving analogue is a native-UI value-policy matrix test set in crates/ui, which would be a new task rather than this issue; the nearest existing coverage is crates/ui/src/app_tests.rs:219-263. |
| #65 | [Gate 5][D2] Verify Tauri DTO -> frontend render/copy/editability end-to-end | native translate + UI tests (crates/native-app/src/translate.rs, crates/ui/src/app_tests.rs). |
| #84 | [RC1][Freeze B1] Run complete frontend release gates on candidate SHA | `.github/workflows/ci.yml` job 'Rust checks' plus the release workflow 'Pre-flight checks' job (fmt/clippy/test on the pinned toolchain). |
| #113 | [META][Agents] Agent dispatcher, claim queue, and concurrency rules for v0.1 | docs/plans/STATUS.md + docs/plans/FEATURE_LIFECYCLE.md + AGENTS.md. |
| #115 | [Gate 4][Support] Build reusable 201/500/1000/dense-hub ER fixture factory | the native ER test fixtures and invariants in crates/ui/src/diagram/tests.rs, delivered with the closed Gate 4 work (#47/#48). |
| #116 | [Gate 4][Support] Build renderer/layout instrumentation harness for exact invariant tests | the native diagram tests that assert worker/layout behaviour directly (coalescing, latest-result-wins, stale request/version rejection, spatial-index consistency). |
## Cross-batch dependencies (what unblocks what)

```text
batch 0  #55 #74 #83 #85 #86 #87 #90 #132 #133 #135   (confirm + close)
             |
batch 1  #142  keyring / DEV fallback removed  ──────────► #126 lifecycle audit can close
         #144  PG secure-by-default            ──────────► #132 row 41/357 must be re-stated, #122
         #147  multi-statement semantics       ──────────► #129 execution-safety matrix, #128
         #145  SQLite backup/restore           ──────────► #128, and the SQLite half of #208 (post-v0.1)
             |
batch 2  #56  TIMESTAMP decode  ──► closes part of #52, feeds #66/#67 evidence
         #61  lossless copy/export ──► #62 policy row, #128 export semantics
         #62  editability policy
         #72  introspection shape contract ──► #26, #73
         #102 + #121  rename/persistence inventory ──► #103, #104, #106
         #114 agent evidence contract ──► makes batch 3/4 handoffs auditable
         #127 version/updater/persistence decision record
         #134 platform prerequisite reconciliation (unblocks clean #91–#93 rows later)
             |
batch 3  #81 ──► #82 ──► #27 ──► #28 ──► #89 (frozen FINAL_RC_SHA)
         #68 ──► #69/#70  (KEEP)  or  #71  (DEFER) ──► #26 ──► #73
         #120 ──► #141 ──► #96 ──► #97/#98/#99 ──► owner decision #101 ──► #103 ──► #104
         #75..#80  area audits ──► #81
         #88  single evidence manifest ──► #89
         #122/#126/#128/#129/#131  audit records ──► #136 register closure
         #135 (closed set) gets corrected only if a batch-1/2 change makes a LIM entry false
             |
batch 4  owner:    #119 license (BLOCKS public distribution) · #101 naming · #146 main protection
                   #108/#109 tag + assets · #111 post-publish · #68/#71 CHECK scope
         external: #92/#93 Windows/Linux hosts · #95 sign-off · #100 live naming diligence
                   #112 GitHub Projects v2
         blocked:  #14 #21..31 #91 #94 #105 #106 #107 #110  (cannot finish before publication)
             |
batch 5  50 post-v0.1 issues — not work
```

Three cycles exist and should be recognised when planning capacity:

- **#81 → #82 → #27 → #28 → #89** is a chain of records, not code. It is cheap per issue but strictly
  ordered, and it is what turns a green pipeline into a defensible release candidate.
- **#68 → #69/#70 or #71** is a fork with an owner decision at the head. Both branches are small; the
  decision is not.
- **#142/#144/#145/#147 → the release documents**: any of these four landing invalidates statements in
  `docs/release/provider-capability-matrix.md`, `known-limitations.md`, `risk-register.md` and
  `docs/architecture/security-boundaries.md`. Whoever lands them must update those files in the same
  change, or the repo gains a new contradiction of exactly the kind this triage had to find.

## Stop conditions

- **Do not start batch 4 work that needs the owner** before the owner has answered: those answers are
  already written as eight concrete questions in `docs/release/0.1.0-human-decisions.md` (HD-001…HD-008).
  Hand that document over rather than re-deriving the questions.
- **Do not close the four P1 security/data-safety issues** (#142, #144, #145, #147) on code alone — each
  one has a packaged-runtime or cross-platform leg that this environment cannot run; they close with a
  documented remainder or not at all.
- **Do not treat a green `cargo test --workspace` as release readiness.** 19 tests are `#[ignore]`d
  (18 `pg_integration` needing `DATABASE_URL`, 1 SSH needing nine `DB_PRO_SSH_*` variables) and the
  workspace total is 815 passed / 0 failed / 19 ignored (`docs/release/evidence/v01-06/12-state-dir-blocker-fix.txt` §4).
- **Do not let batch 5 leak into v0.1.** `docs/notes/V0_1_CLOSURE_PLAN.md:7` states the zero-feature
  expansion rule; the phase documents under `docs/goals/` are for after the release.

## Effort shape (rough, for capacity planning only)

| Batch | Issues | Estimates | Note |
|---|---:|---|---|
| 0 – confirm + close | 10 | 10 × S | one pass, evidence re-read only |
| 1 – P1 blockers | 4 | 3 × M, 1 × L | code changes + docs reconciliation in the same commit |
| 2 – bounded fixes & contracts | 15 | 3 × S, 8 × M, 4 × L | the 8 `ACTIONABLE_NOW` plus the 7 `PARTIAL` rows whose remainder is not blocked |
| 3 – evidence & disposition records | 44 | 10 × S, 25 × M, 7 × L, 2 × XL | ordered by the dependency graph above; mostly documents, tests and disposition records, not code |
| 4 – owner / external / publication | 17 | 7 × S, 6 × M, 4 × L | cannot be finished here; collect answers and external hosts |
| 5 – post-v0.1 | 50 | — | explicitly out of scope (39 from snapshot 1, 11 late arrivals) |
| Superseded | 7 | S | close with a pointer to the replacement |
