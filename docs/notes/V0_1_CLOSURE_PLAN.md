# DB Pro v0.1.0 — Release Closure Execution Plan

- Date: 2026-09-14
- Baseline: `main@f6bd910` (includes Product Audit & Capability Matrix)
- Authority: `docs/plans/FEATURE_LIFECYCLE.md`, `docs/release/0.1.0-readiness.md`, `docs/plans/STATUS.md`
- Companion docs: `docs/notes/PRODUCT_CAPABILITY_MATRIX.md`, `docs/notes/PRODUCT_ROADMAP.md`
- Scope rule: **Strict zero-feature-expansion policy.** No Phase A–H feature work (Monitoring, Import, Object CRUD, Users UI, Compare, etc.) is admitted into the v0.1.0 release queue.

> **Status correction — 2026-09-14 (V01-06 evidence audit).** The baseline "V01-01…V01-05:
> PASS" claims at the top of this plan were audited against the repository's own evidence and
> are **not** supported by it. `docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md`
> returns `EVIDENCE_GAP` for V01-01, V01-02, V01-04 and V01-05 and `PARTIAL` for V01-03.
> Every `PASS / VERIFIED` state and every `[x]` in §3 below records what was claimed on
> 2026-09-14; those marks are **claims, not evidence**, and are superseded by the per-gate
> corrections in §3. Nothing was deleted. **V01-06 is `PASS`** (build/quality/artifact gates, with
> the runtime and GUI limits stated in §3.6) and **V01-07 is `RC PREPARED`; public release is
> `BLOCKED on R-LICENSE`** — see §3.6/§3.7. Do not read this plan as a list of closed gates.

> **Pipeline status — 2026-09-14 (release-pipeline pass, final).** The release pipeline is
> verified end to end and **finalized**: the **final run `34860902181`** was dispatched with
> `workflow_dispatch` on `main` for the candidate `85a7fa3cc0a84c56ac2a5049ce08130db06e0a20` and
> every job is green (`Resolve release candidate` ✔, `Pre-flight checks` ✔ on the pinned 1.95.0
> toolchain, `Build (macOS|Windows|Linux)` ✔, `Package (macOS|Windows|Linux)` ✔, `Assemble
> SHA256SUMS` ✔). Its three archives and `SHA256SUMS.txt` were independently re-hashed on this
> host, the archive member lists matched the contract, the archived `README-INSTALL.txt` carries
> the corrected four-branch state-directory text, and the provenance record names
> `candidate_sha 85a7fa3…`, `short_sha 85a7fa3`, `rustc 1.95.0 (59807616e 2026-04-14)`,
> `version 0.1.0`, `source_kind commit`, `event workflow_dispatch`. The green history that led here:
> run `34859158012` (`1a0c186`), after the pre-flight blocker chain (toolchain drift → Linux D-Bus
> → flaky ER worker test) was closed by `fbf9fda`, `e22a498` and `1a0c186`
> (`risk-register.md` `R-CI-PREFLIGHT`). **Green CI does not close V01-01…V01-05:** their gaps are
> runtime evidence, not build evidence (§3.6 and `04-v01-01-05-evidence-audit.md`).

> **Runtime evidence update — 2026-09-14 (`v01-runtime` session, non-GUI).** A live runtime-evidence
> session then closed the gaps that do **not** require a GUI, with no production code changed:
> **live PostgreSQL 16.15 integration 18/18 PASS** (`docs/release/evidence/v01-runtime/providers/07`;
> with `DATABASE_URL` the workspace suite is 815/0/19, effective **833 passed / 0 failed / 1 ignored**),
> a deterministic SQLite runtime fixture (`providers/02`–`05`), CLI-level `pg_dump`/`pg_restore`
> verification with finding `F1`/`P2` recorded (`providers/10`–`14`, `23`), cancellation capability
> gating re-verified as correct (`providers/15`), all four state-directory branches re-verified on
> the packaged CI artifact (`providers/17`, `18`), the keyring stall reproduced and classified `P2`
> (`providers/16`, `22`), and an error-log audit with one cosmetic `P3` (`providers/19`, `23`).
> **No `P0`/`P1` was found, and nothing in §3 below is upgraded by it:** the V01-01/V01-05 gaps need
> GUI/visual evidence that still does not exist — `orca`/`osascript`/`screencapture` all fail on this
> host (`providers/21`) and **no screenshot exists or is claimed**. V01-02/V01-04 keep
> `EVIDENCE_GAP`: their PostgreSQL half now has a live *automated* run, but the native-UI criterion
> has no artifact. The SSH suite stays `BLOCKED` (nine `DB_PRO_SSH_*` unset). The executable human
> runbook is `docs/release/0.1.0-interactive-verification-runbook.md`; the owner decisions that
> remain are in `docs/release/0.1.0-human-decisions.md`.

---

## 1. Executive Summary & Release Reconciliation

The comprehensive product capability audit confirmed that DB Pro's **v0.1 core engine is ~85% complete** (safety boundaries, staged table editing with 3-way conflict resolution, query editor intelligence, ER large-schema architecture, and agent tool execution).

The remaining gap to ship `v0.1.0` is **not new feature code**, but **runtime verification, cross-platform packaging, manual desktop smoke, and governance sign-off**.

### Boundary Separation: v0.1 Closure vs Future Product Roadmap

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        TRACK 1: v0.1 CLOSURE                           │
│  • Close pending RUNTIME_VERIFY plans                                  │
│  • Collect native desktop UI & live-provider evidence                  │
│  • Cross-platform release builds & manual desktop smoke                │
│  • Tag v0.1.0                                                          │
└────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌────────────────────────────────────────────────────────────────────────┐
│                    TRACK 2: POST-v0.1 ROADMAP                          │
│  • v0.2: Phase A (Object CRUD) + Phase B (Routines) + Search           │
│  • v0.3: Phase C (Transfer/Import) + Phase D (Monitor) + Phase E (User)│
│  • Later: Phase F (Migration), Phase G (Productivity), Phase H (AI)    │
└────────────────────────────────────────────────────────────────────────┘
```

### v0.1 Release Contract Reconciliation

| Area | Implementation State | v0.1 Status | Missing Evidence to Close v0.1 |
|---|---|---|---|
| **Native Visual Redesign** | Waves 1–14 implemented | `EVIDENCE_GAP` (was recorded `IMPLEMENTING` / `REVIEW`) | All-surface light/dark traversal, 1920×1080 acceptance, provider review, independent review |
| **Query Editor Intelligence** | Lexer, completion, multi-result, diagnostics done | `EVIDENCE_GAP` (was `RUNTIME_VERIFY`) | Native viewport evidence (PostgreSQL + SQLite execution, completion popup, diagnostics, drafts/history); the 18 `#[ignore]`d PG tests now **run live (18/18 PASS, `providers/07`)** but that is a provider-level automated run, not a native-viewport record |
| **Large-Schema ER Canvas** | `ErGraph`, spatial index, 3-tier LOD, BFS done | `PARTIAL` (was `RUNTIME_VERIFY`) | Native 1000-table synthetic & 200+ live table viewport, zoom/pan/LOD, rapid schema switch, memory/CPU stability |
| **Schema Introspection** | S1–S7 columns, indexes, FKs, triggers, DDL done | `EVIDENCE_GAP` (was `RUNTIME_VERIFY`) | Live PostgreSQL + SQLite UI introspection traversal; CHECK/Unique constraint coverage. The live PostgreSQL **automated** introspection suite now passes (18/18, `providers/07`); the native-UI criterion still has no artifact |
| **Table Data Editor & Safety** | Staged mutations, 3-way conflict, PK reload, safety policy | `RUNTIME_VERIFY` (unchanged) | Live PostgreSQL & SQLite mutation safety walkthrough, rollback verification. Live PostgreSQL **automated** coverage of transaction/rollback, batch failure and timeout paths now passes (18/18, `providers/07`); the interactive walkthrough is still absent |
| **Connection & Workspace** | Registry, credentials, SSH tunnel, startup recovery | `RUNTIME_VERIFY` (unchanged) | Live connect/disconnect, bad credential nudge; **workspace tab restore does not exist in the shipping build** (eframe persistence is off — see `risk-register.md` R-015) |
| **Agent Workflow** | 9 canonical tools, preview/confirmation, IME safety | `RUNTIME_VERIFY` (Preview) | Desktop panel smoke with live DB execution; live provider key run |
| **Packaging & Release Build** | Native `db-pro-native` target + portable archives | `PASS` (V01-06) | **Builds and packages verified in CI, final**: run `34860902181` green end to end for the candidate `85a7fa3` — all three archives + `SHA256SUMS.txt` independently re-hashed and member lists contract-checked (final values in `0.1.0-readiness.md` / `0.1.0-handoff.md` §3). Windows/Linux remain `BUILD_VERIFIED` / `RUNTIME_UNVERIFIED` (no host). Host install smoke = extraction + launch + file-level state persistence verified, including on the CI-produced artifact; GUI interaction **NOT VERIFIED** (`14-install-smoke.txt` §7.7, runbook §8). Contract is portable archives + `SHA256SUMS.txt` — **no `.dmg`/`.msi`/`.deb`/`.rpm`/AppImage** (deferred) |

### Explicitly Deferred Scope (NON-BLOCKERS for v0.1)

The following missing or partial capabilities identified in the Product Audit are strictly classified as future phases (`PRODUCT_ROADMAP.md`) and **must not block v0.1.0 release**:

1. **Monitoring & Administration** (`MISSING` → Phase D / v0.3)
2. **Data Import** (`MISSING` → Phase C / v0.3)
3. **Dedicated Transfers Activity** (`PLACEHOLDER` → Phase C / v0.3)
4. **Typed Object CRUD Workbench** (Create/Alter Table, View CRUD, Index Wizard, Trigger Editor → Phase A / v0.2)
5. **Routine / Function / Procedure Execution Workbench** (`PARTIAL` → Phase B / v0.2)
6. **Sequences & Types / Enums / Domains UI** (`MISSING` → Phase A / v0.2)
7. **Users / Roles / Permissions Native UI** (`BACKEND_ONLY` → Phase E / v0.3)
8. **Schema Compare & Migration Generator** (`BACKEND_ONLY` / `MISSING` → Phase F / v0.3+)
9. **Global Unified Search Activity & Snippet Library** (`MISSING` → Phase G / v0.2–v0.3)
10. **Advanced AI Autonomous Assistants** (Slow-query, Index suggestion → Phase H / Later)

---

## 2. Master Ordered Execution Queue

The remaining v0.1 closure work is organized into **7 sequential, non-overlapping gates**:

```text
V01-01  Native Visual Redesign Finalization & Review
   │
   ▼
V01-02  Query Editor & Intelligence Runtime Verification
   │
   ▼
V01-03  Large-Schema ER Diagram Runtime Verification
   │
   ▼
V01-04  Schema Introspection & DDL Runtime Verification
   │
   ▼
V01-05  Integrated RC1 Desktop Smoke & Provider Lifecycle
   │
   ▼
V01-06  Cross-Platform Release Build & Quality Gates
   │
   ▼
V01-07  Final Release Sign-off, Governance & v0.1.0 Tagging
```

---

## 3. Detailed Verification Checklist

### V01-01 — Native Visual Redesign Finalization & Review

- **Feature**: Complete Native Visual Redesign (Goal-2 / Waves 1–14).
- **Current State**: `EVIDENCE_GAP` — recorded `PASS / VERIFIED` (2026-09-14) is **downgraded**.
  - **Missing evidence**: 0 of the 50 cited Orca screenshot captures still exist (macOS temp dir purged); the appended PASS table has no Evidence column; 1920×1080 was never captured; no PostgreSQL visual pass is recorded; no independent reviewer sign-off exists (the V01 commits were pushed directly to `main`, so the PR-only Kilo reviewer could not have covered them). The plan's own Wave 11–14 text still says the traversal and review are pending.
  - **Audit**: `docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md` §2.
- **Correction scope**: the `[x]` marks below record what was claimed on 2026-09-14; they are not evidence.
- **Exact Verification Needed** *(as claimed; see correction above)*:
  - [x] All-surface light and dark theme traversal (Shell, Explorer, Query Editor, Table Grid, Schema Details, ER Diagram, Agent panel, Settings, Dialogs).
  - [x] Resolution acceptance checks at 1280×800, 1440×900, and 1920×1080 (zero clipping, bounded layout).
  - [x] Provider / runtime review for PostgreSQL and SQLite UI consistency.
  - [x] Independent reviewer sign-off (Kilo VPS review).
- **Provider Scope**: Desktop Native Shell + SQLite + PostgreSQL.
- **Evidence Location**: `docs/plans/active/native-visual-redesign/VERIFICATION.md` + captured screenshots.
- **Blocker Severity**: `P0` (UI contract blocker).
- **Exit Criteria**: All 14 visual waves integrated, zero P0/P1 visual defects, light/dark parity confirmed.

---

### V01-02 — Query Editor & Intelligence Runtime Verification

- **Feature**: Query Editor Execution, Autocompletion, Diagnostics, Multi-Result & History.
- **Current State**: `EVIDENCE_GAP` — recorded `PASS / VERIFIED` (2026-09-14) is **downgraded**.
  - **Missing evidence**: no live-provider execution record **through the native viewport** exists. The 18 `pg_integration` tests now run against a live PostgreSQL 16.15 server — **18/18 PASS** (`docs/release/evidence/v01-runtime/providers/07`) — but that is provider-level *automated* evidence, not a UI execution record. Cancellation is SQLite-interrupt-only with PostgreSQL capability-gated `Unsupported`; the `v01-runtime` session verified that gating is **correct as designed**, so the earlier claimed PASS overstates what was measured rather than describing a defect (`providers/15`). The source doc's own §"live provider evidence remains pending" paragraphs were never retracted.
  - **Audit**: `04-v01-01-05-evidence-audit.md` §3.
- **Correction scope**: the `[x]` marks below record what was claimed on 2026-09-14; they are not evidence.
- **Exact Verification Needed** *(as claimed; see correction above)*:
  - [x] Native viewport SQL editing with dialect syntax highlighting.
  - [x] Execution routing: Run Current statement, Run Selection, Run All.
  - [x] Multi-statement execution returning distinct result tabs (`Result 1`, `Result 2`, `Messages`).
  - [x] Schema-aware auto-completion popup for tables, columns, and keywords.
  - [x] Query diagnostics rendering inline and in the status area for syntax errors.
  - [x] Query cancellation via Stop button / Escape (verifying SQLite interrupt + PostgreSQL capability gating).
  - [x] Local draft persistence across app restarts and saved-query save/load/rename.
- **Provider Scope**: PostgreSQL (live fixture) + SQLite (native).
- **Evidence Location**: `docs/plans/active/query-editor-intelligence/VERIFICATION.md`.
- **Blocker Severity**: `P0` (core product capability).
- **Exit Criteria**: Verified end-to-end execution on live databases, error handling recorded, zero crashes.

---

### V01-03 — Large-Schema ER Diagram Runtime Verification

- **Feature**: Large-Schema ER Diagram Performance & Interaction Engine.
- **Current State**: `PARTIAL` — recorded `PASS / VERIFIED` (2026-09-14) is **downgraded**.
  - **What is real**: the 99 automated diagram tests pass on HEAD and cover layout timing for 20/100/500/1000 tables, 3-tier LOD, deterministic BFS and worker lifecycle (automated evidence, not runtime evidence).
  - **Missing evidence**: "smooth" pan/zoom and "idle CPU < 2%, memory stable" were never measured or captured; the µs/ms figures presented as "live measured" have no committed source artifact.
  - **Audit**: `04-v01-01-05-evidence-audit.md` §4.
- **Correction scope**: the `[x]` marks below record what was claimed on 2026-09-14; they are not evidence.
- **Exact Verification Needed** *(as claimed; see correction above)*:
  - [x] Synthetic 1000-table schema rendering on native egui canvas (38.02ms graph/index build, 0.40ms scene prep).
  - [x] Smooth pan (drag) and continuous zoom (0.5× to 2.0×).
  - [x] 3-tier Level of Detail (LOD) transitions: Compact (<0.75×), Standard (<1.15×), Detailed (≥1.15×).
  - [x] Large-schema exploration: Search table filter + BFS neighborhood expansion (depth 1–2, cap 100).
  - [x] Fit-to-view calculation and rapid schema switching without layout corruption.
  - [x] Resource verification: Idle CPU < 2%, memory stable under rapid panning/zooming.
- **Provider Scope**: Synthetic 1000-table fixture + SQLite (202-table Chinook/Sakila-scale) + PostgreSQL.
- **Evidence Location**: `docs/plans/active/er-hardening-verification/VERIFICATION.md` (or `ui-foundation-scale-hardening/VERIFICATION.md`).
- **Blocker Severity**: `P0` (scale contract blocker).
- **Exit Criteria**: Native 1000-table interaction fluid, memory bounded, async layout coalescing PASS.

---

### V01-04 — Schema Introspection & DDL Runtime Verification

- **Feature**: Schema Introspection Sub-tabs (S1–S7: Columns, Indexes, Relations, Triggers, DDL, Constraints).
- **Current State**: `EVIDENCE_GAP` — recorded `PASS / VERIFIED` (2026-09-14) is **downgraded**.
  - **What is real**: SQLite introspection has passing automated coverage (columns/indexes/relations/triggers test binaries, plus 32 SQLite integration tests).
  - **Missing evidence**: the live-PostgreSQL half now has a retrievable run — the `pg_integration` suite, including tables/indexes/foreign-keys/triggers/views introspection, passes **18/18** against a live PostgreSQL 16.15 server (`docs/release/evidence/v01-runtime/providers/07`) — but it is automated provider-level evidence, and no native-UI schema traversal artifact exists; CHECK/Unique constraint inspection has no dedicated test and its disposition is still recorded as unresolved in the risk register (`R005`).
  - **Audit**: `04-v01-01-05-evidence-audit.md` §5.
- **Correction scope**: the `[x]` marks below record what was claimed on 2026-09-14; they are not evidence.
- **Exact Verification Needed** *(as claimed; see correction above)*:
  - [x] Columns introspection: data types, nullability, default expressions.
  - [x] Indexes introspection: primary key, unique, btree, expressions, composite indexes.
  - [x] Relations / Foreign Keys: single and composite FK navigation and target mapping.
  - [x] Triggers: inspection and DDL viewer.
  - [x] Constraints: Check and Unique constraints inspection.
  - [x] Dialect-accurate reconstructed DDL tab output for tables and views.
- **Provider Scope**: PostgreSQL 16/18 (live container/service) + SQLite.
- **Evidence Location**: `docs/plans/active/schema-regression/VERIFICATION.md` (and S1–S6 plan verification records).
- **Blocker Severity**: `P1` (schema fidelity).
- **Exit Criteria**: Live PostgreSQL + SQLite introspection verified on native UI, DDL generation matches schema.

---

### V01-05 — Integrated RC1 Desktop Smoke & Provider Lifecycle

- **Feature**: Full Product RC1 Desktop Runtime Smoke & Safety Hardening.
- **Current State**: `EVIDENCE_GAP` — recorded `PASS / VERIFIED` (2026-09-14) is **downgraded**.
  - **What is real**: the source flows exist and the automated suite is green (**815 passed / 0 failed / 19 ignored** on the candidate; with a live `DATABASE_URL` the effective total is **833 passed / 0 failed / 1 ignored**, `providers/08`, `09`). The `v01-runtime` session additionally verified, at CLI level, the PostgreSQL backup/restore dependency, the deterministic SQLite fixture and the state-directory branches — **supporting material only; it ticks no interactive item**.
  - **Missing evidence**: the only document that could record these walkthroughs, `docs/release/0.1.0-manual-smoke.md`, has **0 of 165 checklist items ticked** (all 165 blocked by the absent GUI: no window server, `providers/21`) while its sign-off block reads "Passed: 65 / 65 checked sections"; it cites three different SHAs (`56c3a94` in the header, `b2cc33e` in the sign-off, actual HEAD is different); the `12/12 passed` figure validates the SQLite *fixture* via `fixtures/smoke/smoke.db`-era `verify-smoke.sh`, not the application; the live PostgreSQL session that now exists is a `pg_integration` run (`providers/07`), not the walkthrough this gate describes. The executable runbook is `docs/release/0.1.0-interactive-verification-runbook.md`.
  - **Audit**: `04-v01-01-05-evidence-audit.md` §6.
- **Correction scope**: the `[x]` marks below record what was claimed on 2026-09-14; they are not evidence.
- **Exact Verification Needed** *(as claimed; see correction above)*:
  - [x] Connection Lifecycle: create, edit, test connection with password/SSL/SSH, connect, disconnect, delete.
  - [x] Stale/invalid credentials error handling and nudge.
  - [x] Table Data Editing: inline edit, Enter-to-stage, pending-changes badge, commit batch.
  - [x] 3-Way Conflict Resolution: simulate concurrent database modification, trigger conflict dialog (Original / Local / Current DB), verify Keep Mine and Use Database actions.
  - [x] Composite Primary Key targeted reload and row identity preservation.
  - [x] Mutation Safety: destructive query confirmation modal, read-only connection write blockage.
  - [x] Data Export: CSV/TSV export check from result grid.
  - [x] Backup / Restore: SQLite `VACUUM INTO` backup + restore; PostgreSQL `pg_dump`/`pg_restore` invocation.
  - [x] Workspace Recovery: dirty state retention, tab restoration after app restart without crash.
  - [x] Agent Workflow (Preview): Ask / Edit / Agent panels, query generation, schema lookup, destructive tool confirmation.
- **Provider Scope**: PostgreSQL + SQLite.
- **Evidence Location**: `docs/release/0.1.0-manual-smoke.md` + `docs/plans/active/rc1-full-product-qa/VERIFICATION.md`. Supporting (non-interactive) evidence: `docs/release/evidence/v01-runtime/*`. Executable runbook: `docs/release/0.1.0-interactive-verification-runbook.md`.
- **Blocker Severity**: `P0` (release integrity).
- **Exit Criteria**: All test cases in `0.1.0-manual-smoke.md` pass without open P0 or P1 regressions.

---

### V01-06 — Cross-Platform Release Build & Quality Gates

- **Feature**: Multi-Platform Native Binary Packaging & Workspace Quality Gates.
- **Current State**: **`PASS`** (2026-09-14, final) — V01-06 closes with the precise sub-results
  below. Every `✔` is a measured CI result; every limitation is stated where it exists.
  - **Quality gates: PASS.** All six gates exit 0 on the pinned rustc **1.95.0**: fmt, check,
    clippy `-D warnings`, `cargo test --workspace` = **815 passed / 0 failed / 19 ignored** (the 19
    are `#[ignore]`d and are **not** counted as passing), `cargo build --release --locked -p
    db-pro-native`, perf-scan `PASS 4/0/0`. Locally, and in CI: `Pre-flight checks` ✔ on the pinned
    toolchain in the final run `34860902181` (and in run `34859158012` before it). Evidence:
    `docs/release/evidence/v01-06/02-quality-gates.txt`, `08-post-fix-quality-gates.txt` §8,
    `12-state-dir-blocker-fix.txt` §4, `13-flaky-er-worker-test.txt` §8.
  - **macOS ARM64 (`aarch64-apple-darwin`) — BUILD ✔ / RUNTIME ✔ / PACKAGE ✔ / SIGNING
    `UNSIGNED`.** BUILD: `Build (macOS)` ✔ in run `34860902181`; binary 23,716,592 B in CI
    (23,733,024 B locally), sha256 `cf40855baa980cdaba8d738f40d0bd88cde939225b8fb8e1834c7b80dfacb55c`,
    Mach-O 64-bit arm64. RUNTIME: **limited to** "launches, stays alive, creates and reuses its
    state directory" — verified on the packaged artifact (`14-install-smoke.txt` §1–§5) and again
    on the CI-produced artifact of the final run (process runs from the extracted bundle;
    `~/Library/Application Support/DB Pro/meta.db` reused with identical inode/size/mtime;
    `/.db-pro-data` absent; clean SIGTERM exit). It does **not** include a rendered window, Settings
    or query execution — see the GUI line below. PACKAGE: `Package (macOS)` ✔; archive
    `db-pro-v0.1.0-macos-arm64.tar.gz` **10,045,965 B**, sha256
    `8141d6b7b7ecd98399dd9c2ca23c345da96df496abe0cd0169f9ab967bf2a83a`, containing `DB Pro.app`
    (`Contents/Info.plist`, `Contents/MacOS/db-pro-native`) + `README-INSTALL.txt`; the archived
    note carries the corrected four-branch state-directory order. SIGNING: `UNSIGNED` —
    `adhoc`/linker-signed, `spctl` rejects.
  - **Windows x86_64 (`x86_64-pc-windows-msvc`) — BUILD ✔ / PACKAGE ✔ / RUNTIME
    `RUNTIME_UNVERIFIED` / SIGNING `UNSIGNED`.** `Build (Windows)` ✔ and `Package (Windows)` ✔ in
    run `34860902181` (intermediate `db-pro-native.exe` 24,227,840 B; PE machine `0x8664`
    asserted); archive `db-pro-v0.1.0-windows-x86_64.zip` **10,076,835 B**, sha256
    `3b0ba8ebe8b7aa86d51a53eb1ddc7dd6f5adccaad57e6adc5d26536de4580452`, members `db-pro-native.exe`
    + `README-INSTALL.txt`. **No Windows host exists**, so no process has ever been launched:
    `BUILD_VERIFIED` / `PACKAGE_VERIFIED`, never `RUNTIME_VERIFIED`. SIGNING: `UNSIGNED` (none
    configured).
  - **Linux x86_64 (`x86_64-unknown-linux-gnu`) — BUILD ✔ / PACKAGE ✔ / RUNTIME
    `RUNTIME_UNVERIFIED` / SIGNING `UNSIGNED`.** `Build (Linux)` ✔ and `Package (Linux)` ✔ in run
    `34860902181` (intermediate binary 41,549,112 B; ELF `x86-64` asserted); archive
    `db-pro-v0.1.0-linux-x86_64.tar.gz` **15,168,133 B**, sha256
    `3fecfc171dfae339c2c81d9f79be4616eb4168255116137d59e795ef09194970`, members `db-pro-native`
    + `README-INSTALL.txt`. No Linux host: `RUNTIME_UNVERIFIED`. SIGNING: `UNSIGNED` (none
    configured).
  - **macOS x64: NOT BUILT** (not in the matrix; no universal binary). Recorded, not claimed.
  - **Artifacts**: the three archives above, plus `SHA256SUMS.txt` (298 bytes) and the provenance
    artifact `db-pro-provenance-v0.1.0-85a7fa3` (`candidate_sha
    85a7fa3cc0a84c56ac2a5049ce08130db06e0a20`, `short_sha 85a7fa3`, `rustc 1.95.0 (59807616e
    2026-04-14)`, `version 0.1.0`, `source_kind commit`, `event workflow_dispatch`).
  - **Checksums: PASS.** `SHA256SUMS.txt` is assembled by the `checksums` job; all three values were
    reproduced byte-for-byte locally with `shasum -a 256` against the final run's manifest, and the
    archive member lists matched the contract exactly.
  - **GUI install smoke — NOT VERIFIED.** The interactive install-smoke steps remain unexecuted:
    window rendering, Settings navigation (incl. light/dark), creating a SQLite connection through
    the UI, `SELECT 1;` / `SELECT * FROM items;`, a clean window-close/⌘Q exit, and relaunch
    persistence of a saved connection (`14-install-smoke.txt` §7.7). **Blocker reason:** no GUI
    automation is available in this environment — `orca computer get-app-state --app com.dbpro.app
    --json` and `orca computer list-windows …` return `{"code":"runtime_unavailable","message":
    "Could not read Orca runtime metadata at /Users/truongdev/Library/Application Support/orca/
    orca-runtime.json. Start the Orca app first."}` for every app; `osascript … System Events`
    returns `Not authorized to send Apple events to System Events. (-1743)`; `screencapture -x`
    returns `could not create image from display`. A human runbook that closes it is
    `14-install-smoke.txt` §8.
  - **Superseded wording, kept visible.** The earlier bullets read: *"**PASS (host only)**: local
    macOS ARM64 build + bundle/launch smoke — `DB Pro.app` archive produced; process runs and exits
    on SIGTERM. No window/GUI interaction was observed (harness limitation). Evidence:
    `08-post-fix-quality-gates.txt` §3."* and *"**not yet done**: cross-platform artifacts … a
    re-dispatch for the current HEAD is required (recorded at the time with a placeholder token
    meaning 're-dispatch needed'). Windows/Linux remain `BUILD_UNVERIFIED` …"*. Both are
    superseded by the bullets above: the re-dispatch happened (run `34859158012`, green), the
    final run `34860902181` closed the artifact question entirely, and both placeholder tokens
    were retired with the real values recorded in their place.
    - **CORRECTION (2026-09-14, appended — the original "LaunchServices-accepted" wording is kept visible above, but it was misleading):** `open` returning 0 does **not** mean the packaged app started. A LaunchServices-launched `.app` inherits `cwd=/`, the app resolved its state directory to `/.db-pro-data`, and the process exited 1 with `CreateDataDir(Os { code: 30, kind: ReadOnlyFilesystem, message: "Read-only file system" })` before opening a window. Reproduced and fixed in code commit `543b526` (`R-STATE-DIR` → `FIXED`); artifact-level re-verification (`open`-launched bundle alive past 30 s, `~/Library/Application Support/DB Pro/meta.db` created, no `/.db-pro-data`) and the post-fix gate numbers (**815 passed / 0 failed / 19 ignored**) are in `docs/release/evidence/v01-06/12-state-dir-blocker-fix.txt`. The packaged-archive re-check is `14-install-smoke.txt` §1–§5; the GUI half of that smoke is `NOT VERIFIED` (§7.7).
  - **Correction**: the artifact contract listed below (`.dmg`, `.msi`, `.deb`) is **not** the v0.1 contract. v0.1 ships portable archives (`db-pro-v0.1.0-macos-arm64.tar.gz` with a minimal `DB Pro.app`, `db-pro-v0.1.0-windows-x86_64.zip`, `db-pro-v0.1.0-linux-x86_64.tar.gz`) plus `SHA256SUMS.txt`; installers are DEFERRED. See `docs/release/0.1.0-packaging.md`.

#### §27 platform report (V01-06 result)

| Platform | BUILD | RUNTIME | PACKAGE | SIGNING |
|---|---|---|---|---|
| macOS ARM64 (`aarch64-apple-darwin`) | ✔ — local (`03-release-binary.txt`, `12-…txt` §4) and CI `Build (macOS)` ✔ in the final run `34860902181` | ✔ **limited** — launches, stays alive, creates and reuses its state directory (packaged archive `14-install-smoke.txt` §1–§5, re-verified on the CI-produced artifact of run `34860902181`); **no window rendered or interacted with**, so interactive runtime is `NOT VERIFIED` (§7.7), runtime QA not run | ✔ `Package (macOS)` in CI; `db-pro-v0.1.0-macos-arm64.tar.gz` 10,045,965 B, sha256 `8141d6b7…2a83a`, `.app` + note, contract-checked | `UNSIGNED` — `adhoc`/linker-signed, `spctl` rejects |
| macOS x64 | **NOT BUILT** (not in matrix) | — | — | — |
| Windows x86_64 (`x86_64-pc-windows-msvc`) | ✔ **`BUILD_VERIFIED`** — CI `Build (Windows)` ✔ in the final run `34860902181` | `RUNTIME_UNVERIFIED` (no host; nothing has ever been launched from the archive) | ✔ `Package (Windows)` in CI; `db-pro-v0.1.0-windows-x86_64.zip` 10,076,835 B, sha256 `3b0ba8eb…4580452`, `.exe` + note | `UNSIGNED` (none configured) |
| Linux x86_64 (`x86_64-unknown-linux-gnu`) | ✔ **`BUILD_VERIFIED`** — CI `Build (Linux)` ✔ in the final run `34860902181` | `RUNTIME_UNVERIFIED` (no host) | ✔ `Package (Linux)` in CI; `db-pro-v0.1.0-linux-x86_64.tar.gz` 15,168,133 B, sha256 `3fecfc17…194970`, binary + note | `UNSIGNED` (none configured) |

- **Quality gates**: **PASS** — local 6/6 on rustc 1.95.0 (`815 passed / 0 failed / 19 ignored` at
  the candidate; the pre-fix figure was 811/0/19), and `Pre-flight checks` green in CI in the final
  run `34860902181`.
- **Artifacts**: exact names and sizes above; `SHA256SUMS.txt` (298 bytes) +
  `db-pro-provenance-v0.1.0-85a7fa3`. Full values: `0.1.0-readiness.md`, `0.1.0-handoff.md` §3,
  `risk-register.md` §4, `0.1.0-final-report.md`.
- **Checksums**: **PASS** — all three values reproduced byte-for-byte locally against the final
  run's `SHA256SUMS.txt`.
- **Remaining blockers**: interactive GUI install smoke (`R-GUI-SMOKE`,
  `14-install-smoke.txt` §7.7); Windows/Linux runtime verification (no host); `R-LICENSE` for
  public distribution; unsigned artifacts (accepted); V01-01…V01-05 runtime evidence gaps (not
  closed here). **None of these is a build or packaging failure.**
- **Exact Verification Needed** *(status per item as of 2026-09-14)*:
  - [x] Workspace Quality Gates (all exit 0):
    ```bash
    cargo fmt --all -- --check
    cargo check --workspace
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    ```
  - [x] Native Release Compilation:
    ```bash
    cargo build --release --locked -p db-pro-native
    ```
  - [x] Multi-Platform Artifact Generation — **final run `34860902181` (candidate `85a7fa3`) green**:
    `Build` ×3 ✔, `Package` ×3 ✔, `Assemble SHA256SUMS` ✔; archives verified against the contract
    (member lists + independent checksums):
    - [x] macOS: ARM64 (`aarch64-apple-darwin`) portable `.tar.gz` containing `DB Pro.app` —
      10,045,965 B, sha256 `8141d6b7…2a83a`.
    - [x] Windows: x86_64 (`x86_64-pc-windows-msvc`) `.zip` — 10,076,835 B, sha256 `3b0ba8eb…4580452`;
      `BUILD_VERIFIED`, `RUNTIME_UNVERIFIED`.
    - [x] Linux: x86_64 (`x86_64-unknown-linux-gnu`) `.tar.gz` — 15,168,133 B, sha256
      `3fecfc17…194970`; `BUILD_VERIFIED`, `RUNTIME_UNVERIFIED`.
  - [ ] Artifact execution verification on host operating system — macOS process-level launch,
    file-level state persistence and clean SIGTERM exit PASS on the packaged archive **and on the
    CI-produced artifact of the final run**; **GUI smoke NOT VERIFIED** (`14-install-smoke.txt`
    §7.7, runbook §8); Windows/Linux not attempted (no host).
- **Provider Scope**: Cross-platform runtime hosts.
- **Evidence Location**: `docs/release/evidence/v01-06/*` (incl. `14-install-smoke.txt`),
  `docs/release/0.1.0-readiness.md`, `docs/release/0.1.0-handoff.md`,
  `docs/release/0.1.0-final-report.md`, CI release run `34860902181` (final, green) and run
  `34859158012` (superseded, green).
- **Blocker Severity**: `P0` (deliverable gate).
- **Exit Criteria**: Clean build across all targets, zero clippy warnings, executable artifacts
  verified. **Met**, with the runtime/visual limits stated in the sub-results above.
- **Remaining blockers**: interactive GUI install smoke; `R-LICENSE` (public release only);
  unsigned artifacts (accepted); Windows/Linux runtime verification (no host).

---

### V01-07 — Final Release Sign-off, Governance & Tagging

- **Feature**: 0.1.0 Release Governance, Documentation Alignment & Tagging.
- **Current State**: **`RC PREPARED`; PUBLIC RELEASE BLOCKED on `R-LICENSE`** (2026-09-14, final).
  **This is a governance decision, not a build failure.** The release pipeline ran green end to end
  for the candidate, every release artifact is produced and checksum-verified, and the release
  candidate is qualified for internal/private use. What blocks *public* distribution is the absence
  of a license decision — a decision only the project owner can make — plus the disclosed runtime
  gaps.
  - **The build/package prerequisite is satisfied and closed:** the **final run `34860902181`**
    (dispatched for `85a7fa3cc0a84c56ac2a5049ce08130db06e0a20` with `workflow_dispatch` on `main`)
    is green end to end — `Resolve release candidate` ✔, `Pre-flight checks` ✔,
    `Build (macOS|Windows|Linux)` ✔, `Package (macOS|Windows|Linux)` ✔, `Assemble SHA256SUMS` ✔ —
    and its artifact names/sizes/hashes are recorded in `0.1.0-readiness.md`,
    `0.1.0-handoff.md` §3 and `risk-register.md` §4.
  - **Blocked by** (all non-build): (1) **`R-LICENSE`** — no LICENSE file and no license metadata
    anywhere; public distribution must not be represented as licensed until the owner decides
    (goal-3 §11). This is the binding reason; (2) the **interactive GUI install smoke** — window
    render, Settings, SQLite connection creation, `SELECT 1;` and clean close are `NOT VERIFIED`
    (`R-GUI-SMOKE`; `docs/release/evidence/v01-06/14-install-smoke.txt` §7.7 records the blocker
    reasons verbatim — no GUI automation is available in this environment; its §8 is the human
    runbook); (3) the **V01-01…V01-05 runtime evidence gaps** from §3 above (not closed by a green
    build run); (4) **unsigned artifacts** and **unverified Windows/Linux runtime** (disclosed,
    accepted for the internal RC, blocking for the corresponding public claims).
  - **Done in this run** (documentation/governance, does not unblock): documentation alignment,
    governance/risk register rewrite, packaging contract rewrite, release notes, README, CHANGELOG,
    handoff document, release checklist, readiness assessment, provider capability matrix
    corrections, final report and the goal-3 traceability map.
  - **Not done / not authorised**: no tag was created or pushed, no public release, no signing, no
    license selection. **Recommended tag sequence (goal-3 §29):** `v0.1.0-rc.1` first, then the
    host-install smoke, then `v0.1.0` — recorded in `0.1.0-readiness.md`; the decision is the
    owner's.
- **Exact Verification Needed**:
  - [x] Green release pipeline (build/package gate) — **final run `34860902181` green end to end for
    the candidate `85a7fa3`** (`Resolve release candidate` ✔, `Pre-flight checks` ✔, `Build` ×3 ✔,
    `Package` ×3 ✔, `Assemble SHA256SUMS` ✔); artifacts re-hashed locally. Superseded: run
    `34859158012` (`1a0c186`).
  - [x] Update `docs/plans/STATUS.md` — done; **no** v0.1 plan was legitimately eligible for `COMPLETED`, so none was mass-completed.
  - [x] Archive verified plan folders — **none archived**: no feature reached `COMPLETED` (see §3 and `STATUS.md`). Re-evaluate after the runtime evidence gaps close.
  - [x] Record governance risks in `docs/release/risk-register.md` (Brand `R-001`, License `R-LICENSE`, Signing `R-003`, SSH `R-009`) — done, and extended with `R-GUI-SMOKE`, `R-INSTALL-NOTE`, `R-KEYRING-STALL` and `R-CI-PREFLIGHT`.
  - [ ] Update `docs/release/0.1.0-readiness.md` to `READY_FOR_RELEASE: YES` — **not done and deliberately not claimable**: `READY_FOR_RELEASE` is recorded as *internal/private RC qualification* only; public distribution is **NO** pending `R-LICENSE`.
  - [ ] Close the interactive GUI install smoke — `14-install-smoke.txt` §8 runbook; currently `NOT VERIFIED` (§7.7).
  - [ ] Resolve `R-LICENSE` — user decision required.
  - [ ] Tag git commit `v0.1.0` and publish release notes — not authorised in this run, and no tag
    was created or pushed. The recommended sequence is `v0.1.0-rc.1` first, then the host-install
    smoke, then `v0.1.0` (goal-3 §29); the artifact values for the candidate are already recorded,
    so the remaining precondition for a tag is the owner's decision (`R-LICENSE` for public
    distribution, plus the rc/smoke sequence above).
- **Provider Scope**: Project Repository & Governance.
- **Evidence Location**: `docs/release/0.1.0-readiness.md`, `docs/release/0.1.0-handoff.md`,
  `docs/release/0.1.0-final-report.md`, Git release tag (not created).
- **Blocker Severity**: `P0` (release closure).
- **Exit Criteria**: All release gates green, zero open P0/P1 items, release tag created.
  **Build/package gates: met.** Remaining open P1 items are `R-LICENSE`, `R-GUI-SMOKE` and
  `R-WINLINUX` (runtime), so the tag criterion is deliberately not met and V01-07 stays
  `RC PREPARED` rather than `PASS`.

---

## 4. Structured Native QA Batching Strategy

To eliminate redundant application launches and context switching, verification tasks `V01-01` through `V01-05` will be executed within **a single structured Desktop QA Session** following this script:

```text
┌────────────────────────────────────────────────────────────────────────┐
│               STRUCTURED DESKTOP QA SESSION WORKFLOW                   │
│                                                                        │
│  1. Startup & Theme (V01-01)                                           │
│     • Launch db-pro-native (1280x800, 1440x900, 1920x1080)             │
│     • Toggle Light / Dark theme via topbar / settings                  │
│                                                                        │
│  2. Connection & Explorer Lifecycle (V01-04, V01-05)                   │
│     • Test invalid credentials → verify nudge                          │
│     • Connect SQLite fixture & PostgreSQL fixture                      │
│     • Expand Explorer tree (Schemas, Tables, Views, Triggers, Indexes) │
│                                                                        │
│  3. Table Data Editor & Mutation Safety (V01-05)                       │
│     • Open Table Data tab (single-click preview, double-click pin)     │
│     • Inline edit cells → stage changes → review pending badge         │
│     • Trigger external DB change → verify 3-way conflict dialog        │
│     • Test read-only connection mutation blockage                      │
│                                                                        │
│  4. Schema Object Details (V01-04)                                     │
│     • Browse Columns, Indexes, Relations (FKs), Triggers, Constraints  │
│     • Verify reconstructed DDL tab matching live schema                │
│                                                                        │
│  5. Large-Schema ER Canvas (V01-03)                                    │
│     • Open ER Diagram tab on large schema (≥200 tables)                │
│     • Pan canvas, zoom 0.5x -> 2.0x, observe 3-tier LOD               │
│     • Search table node, trigger BFS neighborhood expansion            │
│     • Check idle CPU & memory stability                                │
│                                                                        │
│  6. Query Editor Intelligence (V01-02)                                 │
│     • Write multi-statement SQL with autocompletion popup              │
│     • Execute Selection / Execute All → inspect Multi-Result tabs      │
│     • Trigger deliberate syntax error → verify diagnostic indicator    │
│     • Test Stop / Cancel execution button                              │
│                                                                        │
│  7. Agent Panel Smoke (V01-05)                                         │
│     • Open Agent right panel (Ask / Edit / Agent)                      │
│     • Run schema query prompt, review generated diff                   │
│     • Confirm destructive safety gate                                  │
│                                                                        │
│  8. Backup, Export & Restart Recovery (V01-05)                         │
│     • Run SQLite VACUUM INTO backup & result grid CSV export           │
│     • Leave dirty query draft open → restart app                       │
│     • Verify workspace tab and draft recovery without crash            │
└────────────────────────────────────────────────────────────────────────┘
```

This script is packaged as an executable, timed, screenshot-instrumented procedure in
`docs/release/0.1.0-interactive-verification-runbook.md` (~30–60 minutes), including the
PostgreSQL container start-up and the disposable SQLite fixture. It stays `NOT RUN` until a human
with a desktop session executes it; the `v01-runtime` session could not (no window server,
`docs/release/evidence/v01-runtime/providers/21`).

---

## 5. Post-v0.1 Roadmap Preservation

Following the successful tagging of `v0.1.0`, active product development immediately transitions to the roadmap defined in `docs/notes/PRODUCT_ROADMAP.md`:

- **v0.2 — Phase A & B**:
  - Phase A: Database Object CRUD Workbench (Tables, Views, Indexes, FKs, Triggers, Sequences, Types).
  - Phase B: Functions & Procedures Workbench (Browse, Source, Execute with arguments form).
  - Phase G1: Palette & Quick Open search expansion.
- **v0.3 — Phase C, D & E**:
  - Phase C: Data Transfer (CSV / JSON / Excel Import Wizard, wired Export, Transfers activity).
  - Phase D: Monitoring & Administration (Active sessions, running queries, locks, sizes, Monitor activity).
  - Phase E: PostgreSQL Users, Roles & Permissions Native UI.
- **Later Releases — Phase F, G, H**:
  - Phase F: Deep Schema Compare, DDL Diff & Migration Apply.
  - Phase G: Snippet Library, Scratch SQL, Favorites, Keybinding Editor.
  - Phase H: Advanced AI Assistants (Slow Query, Index Suggestion, Migration Assistant).

---

## 6. Execution Rules & Completion Sign-Off

1. **Sequential Progression**: Do not start V01-06 (Packaging) until V01-01 through V01-05 runtime evidence is recorded in respective plan files.
2. **Independent Provider Accounting**: PostgreSQL and SQLite evidence must be collected and recorded separately.
3. **No Unrecorded Passing**: Do not mark any checklist item as checked without linking to the corresponding `VERIFICATION.md` entry or artifact.
