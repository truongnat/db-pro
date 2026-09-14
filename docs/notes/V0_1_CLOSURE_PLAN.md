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
> corrections in §3. Nothing was deleted. V01-06 is `PARTIAL` and V01-07 is `BLOCKED` — see
> §3.6/§3.7. Do not read this plan as a list of closed gates.

> **Pipeline status — 2026-09-14 (release-pipeline pass).** The release pipeline itself is now
> verified end to end: run **`34859158012`** completed **fully green** for SHA `1a0c186`
> (`Resolve` ✔, `Pre-flight checks` ✔ on the pinned 1.95.0 toolchain, `Build (macOS|Windows|Linux)`
> ✔, `Package (macOS|Windows|Linux)` ✔, `Assemble SHA256SUMS` ✔), its three archives and
> `SHA256SUMS.txt` were independently checksum-verified, and the provenance record named
> `candidate_sha 1a0c186…`, `rustc 1.95.0 (59807616e 2026-04-14)`, `version 0.1.0`. The
> pre-flight blocker chain (toolchain drift → Linux D-Bus → flaky ER worker test) is closed by
> `fbf9fda`, `e22a498`, `1a0c186` (`risk-register.md` `R-CI-PREFLIGHT`). The **final** release run
> `34860902181` is in flight for the current HEAD `85a7fa3` (which adds the packaged install-note
> fix, `85a7fa3`, `R-INSTALL-NOTE`); every final artifact value below reads `PENDING_FINAL_CI_RUN_34860902181`.
> **Green CI does not close V01-01…V01-05:** their gaps are runtime evidence, not build evidence
> (§3.6 and `04-v01-01-05-evidence-audit.md`).

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
| **Query Editor Intelligence** | Lexer, completion, multi-result, diagnostics done | `EVIDENCE_GAP` (was `RUNTIME_VERIFY`) | Native viewport evidence (PostgreSQL + SQLite execution, completion popup, diagnostics, drafts/history); live provider run for the 18 `#[ignore]`d PG tests |
| **Large-Schema ER Canvas** | `ErGraph`, spatial index, 3-tier LOD, BFS done | `PARTIAL` (was `RUNTIME_VERIFY`) | Native 1000-table synthetic & 200+ live table viewport, zoom/pan/LOD, rapid schema switch, memory/CPU stability |
| **Schema Introspection** | S1–S7 columns, indexes, FKs, triggers, DDL done | `EVIDENCE_GAP` (was `RUNTIME_VERIFY`) | Live PostgreSQL + SQLite UI introspection traversal; CHECK/Unique constraint coverage |
| **Table Data Editor & Safety** | Staged mutations, 3-way conflict, PK reload, safety policy | `RUNTIME_VERIFY` (unchanged) | Live PostgreSQL & SQLite mutation safety walkthrough, rollback verification |
| **Connection & Workspace** | Registry, credentials, SSH tunnel, startup recovery | `RUNTIME_VERIFY` (unchanged) | Live connect/disconnect, bad credential nudge; **workspace tab restore does not exist in the shipping build** (eframe persistence is off — see `risk-register.md` R-015) |
| **Agent Workflow** | 9 canonical tools, preview/confirmation, IME safety | `RUNTIME_VERIFY` (Preview) | Desktop panel smoke with live DB execution; live provider key run |
| **Packaging & Release Build** | Native `db-pro-native` target + portable archives | `PARTIAL` | **Builds and packages verified in CI**: run `34859158012` green end to end for `1a0c186`, with all three archives + `SHA256SUMS.txt` independently checksum-verified. Windows/Linux remain `BUILD_VERIFIED` / `RUNTIME_UNVERIFIED` (no host). Host install smoke = extraction + launch + state persistence verified; GUI interaction **NOT VERIFIED** (`14-install-smoke.txt` §7.7). Final artifact values for HEAD `85a7fa3`: `PENDING_FINAL_CI_RUN_34860902181`. Contract is portable archives + `SHA256SUMS.txt` — **no `.dmg`/`.msi`/`.deb`/`.rpm`/AppImage** (deferred) |

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
  - **Missing evidence**: no live-provider execution record exists; the PostgreSQL half rests on the 18 `#[ignore]`d `pg_integration` tests (ignored, not passing); cancellation is SQLite-interrupt-only with PostgreSQL capability-gated `Unsupported`, so the claimed PASS overstates it; the source doc's own §"live provider evidence remains pending" paragraphs were never retracted.
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
  - **Missing evidence**: the live PostgreSQL half rests on `#[ignore]`d tests (0 passed / 18 ignored without `DATABASE_URL`); no native-UI schema traversal artifact exists; CHECK/Unique constraint inspection has no dedicated test and its disposition is still recorded as unresolved in the risk register (`R005`).
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
  - **What is real**: the source flows exist and the automated suite is green (811 passed / 0 failed / 19 ignored on HEAD).
  - **Missing evidence**: the only document that could record these walkthroughs, `docs/release/0.1.0-manual-smoke.md`, has **0 of 165 checklist items ticked** while its sign-off block reads "Passed: 65 / 65 checked sections"; it cites three different SHAs (`56c3a94` in the header, `b2cc33e` in the sign-off, actual HEAD is different); the `12/12 passed` figure validates the SQLite *fixture* via `fixtures/smoke/verify-smoke.sh`, not the application; no provider run artifacts (container log, connection trace, version query output) exist for the asserted PostgreSQL 18.2 session.
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
- **Evidence Location**: `docs/release/0.1.0-manual-smoke.md` + `docs/plans/active/rc1-full-product-qa/VERIFICATION.md`.
- **Blocker Severity**: `P0` (release integrity).
- **Exit Criteria**: All test cases in `0.1.0-manual-smoke.md` pass without open P0 or P1 regressions.

---

### V01-06 — Cross-Platform Release Build & Quality Gates

- **Feature**: Multi-Platform Native Binary Packaging & Workspace Quality Gates.
- **Current State**: `PARTIAL` (2026-09-14) — accurate composite. What is verified, in order:
  - **PASS — local quality gates.** All six gates exit 0 on the pinned rustc 1.95.0: fmt, check,
    clippy `-D warnings`, `cargo test --workspace` = **815 passed / 0 failed / 19 ignored**,
    `cargo build --release --locked -p db-pro-native`, perf-scan `PASS 4/0/0`. Evidence:
    `docs/release/evidence/v01-06/02-quality-gates.txt`, `08-post-fix-quality-gates.txt` §8,
    `12-state-dir-blocker-fix.txt` §4, `13-flaky-er-worker-test.txt` §8.
  - **PASS — release pre-flight in CI.** `Pre-flight checks` ✔ (fmt/clippy/tests) on the pinned
    1.95.0 toolchain in run `34859158012`. The blocker chain that delayed it — toolchain drift →
    Linux D-Bus → flaky ER worker test — is `FIXED` by `fbf9fda`, `e22a498` and `1a0c186`
    (`risk-register.md` `R-CI-PREFLIGHT`; `13-flaky-er-worker-test.txt`).
  - **PASS — all three platform builds and packages in CI.** Run `34859158012` (SHA `1a0c186`):
    `Build (macOS)` ✔, `Build (Windows)` ✔, `Build (Linux)` ✔, `Package (macOS|Windows|Linux)` ✔,
    `Assemble SHA256SUMS` ✔ — the workflow is green end to end. Windows/Linux are therefore
    `BUILD_VERIFIED` (and packaged); they remain **`RUNTIME_UNVERIFIED`** because no Windows/Linux
    host exists in this project. The two states are deliberately not blurred.
  - **PASS — artifact checksums independently verified (run `34859158012` only).** The three
    archives were downloaded and their SHA-256 values reproduced byte-for-byte with
    `shasum -a 256` against the run's `SHA256SUMS.txt`; archive member lists matched the contract;
    provenance named `candidate_sha 1a0c186…`, `rustc 1.95.0 (59807616e 2026-04-14)`,
    `version 0.1.0`. **These are not the final release values** — the labelled table lives in
    `risk-register.md` §4.
  - **PARTIAL — host install smoke.** Extraction, LaunchServices launch and **file-level** state
    persistence are verified on the packaged archive built from `1a0c186` (`14-install-smoke.txt`
    §1–§5); **every GUI step is `NOT VERIFIED`** (§7.7) because the GUI was unreachable in that
    environment for four separately recorded reasons (§7.1–§7.6). Human runbook to close it: §8.
    `docs/release/0.1.0-ui-visual-description.md` is a code-derived surface description and does
    **not** close this gap.
  - **PENDING — final artifact values for the current HEAD.** The final release run `34860902181`
    is in flight for `85a7fa3` (which adds the packaged install-note fix, `R-INSTALL-NOTE`); its
    sizes and hashes read `PENDING_FINAL_CI_RUN_34860902181` everywhere in these documents.
  - **Superseded wording, kept visible.** The earlier bullets read: *"**PASS (host only)**: local
    macOS ARM64 build + bundle/launch smoke — `DB Pro.app` archive produced; process runs and exits
    on SIGTERM. No window/GUI interaction was observed (harness limitation). Evidence:
    `08-post-fix-quality-gates.txt` §3."* and *"**PENDING**: cross-platform artifacts … a
    re-dispatch for the current HEAD is required (`PENDING_CI_RUN_RE_DISPATCH`). Windows/Linux
    remain `BUILD_UNVERIFIED` …"*. Both are superseded by the bullets above: the re-dispatch
    happened (run `34859158012`, green) and the `PENDING_CI_RUN_RE_DISPATCH` token is retired in
    favour of `PENDING_FINAL_CI_RUN_34860902181`.
    - **CORRECTION (2026-09-14, appended — the original "LaunchServices-accepted" wording is kept visible above, but it was misleading):** `open` returning 0 does **not** mean the packaged app started. A LaunchServices-launched `.app` inherits `cwd=/`, the app resolved its state directory to `/.db-pro-data`, and the process exited 1 with `CreateDataDir(Os { code: 30, kind: ReadOnlyFilesystem, message: "Read-only file system" })` before opening a window. Reproduced and fixed in code commit `543b526` (`R-STATE-DIR` → `FIXED`); artifact-level re-verification (`open`-launched bundle alive past 30 s, `~/Library/Application Support/DB Pro/meta.db` created, no `/.db-pro-data`) and the post-fix gate numbers (**815 passed / 0 failed / 19 ignored**) are in `docs/release/evidence/v01-06/12-state-dir-blocker-fix.txt`. The packaged-archive re-check is `14-install-smoke.txt` §1–§5; the GUI half of that smoke is `NOT VERIFIED` (§7.7).
  - **Correction**: the artifact contract listed below (`.dmg`, `.msi`, `.deb`) is **not** the v0.1 contract. v0.1 ships portable archives (`db-pro-v0.1.0-macos-arm64.tar.gz` with a minimal `DB Pro.app`, `db-pro-v0.1.0-windows-x86_64.zip`, `db-pro-v0.1.0-linux-x86_64.tar.gz`) plus `SHA256SUMS.txt`; installers are DEFERRED. See `docs/release/0.1.0-packaging.md`.

#### §27 platform report (V01-06 result)

| Platform | BUILD | RUNTIME | PACKAGE | SIGNING |
|---|---|---|---|---|
| macOS ARM64 (`aarch64-apple-darwin`) | PASS — local (`03-release-binary.txt`, `12-…txt` §4) and CI run `34859158012` | `PARTIAL` — process launch/idle PASS and file-level state reuse PASS on the packaged archive (`14-install-smoke.txt` §1–§5); **GUI interaction NOT VERIFIED** (§7.7); runtime QA not run | PASS in CI (run `34859158012`); final values for HEAD `85a7fa3`: `PENDING_FINAL_CI_RUN_34860902181` | `UNSIGNED` — `adhoc`/linker-signed, `spctl` rejects |
| macOS x64 | **NOT BUILT** (not in matrix) | — | — | — |
| Windows x86_64 (`x86_64-pc-windows-msvc`) | **`BUILD_VERIFIED`** — CI run `34859158012` | `RUNTIME_UNVERIFIED` (no host) | PASS in CI (run `34859158012`); final values `PENDING_FINAL_CI_RUN_34860902181` | `UNSIGNED` (none configured) |
| Linux x86_64 (`x86_64-unknown-linux-gnu`) | **`BUILD_VERIFIED`** — CI run `34859158012` | `RUNTIME_UNVERIFIED` (no host) | PASS in CI (run `34859158012`); final values `PENDING_FINAL_CI_RUN_34860902181` | `UNSIGNED` (none configured) |

- **Quality gates**: **PASS** — local 6/6 on rustc 1.95.0 (`815 passed / 0 failed / 19 ignored` after the `543b526` state-directory fix; the pre-fix candidate figure was 811/0/19), and `Pre-flight checks` green in CI run `34859158012`.
- **Artifacts**: exact names above; run `34859158012` (SHA `1a0c186`) produced all three and their sizes/hashes were independently verified (labelled table: `risk-register.md` §4). Final values for HEAD `85a7fa3`: `PENDING_FINAL_CI_RUN_34860902181`.
- **Checksums**: `SHA256SUMS.txt` is assembled by the `checksums` job and its run-`34859158012` values were reproduced byte-for-byte locally; final values `PENDING_FINAL_CI_RUN_34860902181`.
- **Remaining blockers**: interactive GUI install smoke (`R-GUI-SMOKE`, `14-install-smoke.txt` §7.7); Windows/Linux runtime verification (no host); `R-LICENSE` for public distribution; unsigned artifacts (accepted); V01-01…V01-05 runtime evidence gaps (not closed here).
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
  - [x] Multi-Platform Artifact Generation — **run `34859158012` (SHA `1a0c186`) green**: `Build` ×3 ✔, `Package` ×3 ✔, `Assemble SHA256SUMS` ✔; archives verified against the contract (member lists + independent checksums). Final values for the current HEAD (`85a7fa3`, run `34860902181`) are `PENDING_FINAL_CI_RUN_34860902181`:
    - [x] macOS: ARM64 (`aarch64-apple-darwin`) portable `.tar.gz` containing `DB Pro.app` — run `34859158012` (10,045,739 B, sha256 `911335d0…`); final values `PENDING_FINAL_CI_RUN_34860902181`.
    - [x] Windows: x86_64 (`x86_64-pc-windows-msvc`) `.zip` — run `34859158012` (10,076,654 B, sha256 `62870d9a…`); `BUILD_VERIFIED`, `RUNTIME_UNVERIFIED`; final values `PENDING_FINAL_CI_RUN_34860902181`.
    - [x] Linux: x86_64 (`x86_64-unknown-linux-gnu`) `.tar.gz` — run `34859158012` (15,167,954 B, sha256 `d28bbf3a…`); `BUILD_VERIFIED`, `RUNTIME_UNVERIFIED`; final values `PENDING_FINAL_CI_RUN_34860902181`.
  - [ ] Artifact execution verification on host operating system — macOS process-level launch and file-level state persistence PASS on the packaged archive; **GUI smoke NOT VERIFIED** (`14-install-smoke.txt` §7.7, runbook §8); Windows/Linux not attempted (no host).
- **Provider Scope**: Cross-platform runtime hosts.
- **Evidence Location**: `docs/release/evidence/v01-06/*` (incl. `14-install-smoke.txt`), `docs/release/0.1.0-readiness.md`, `docs/release/0.1.0-handoff.md`, CI release run `34859158012` (completed, green) and run `34860902181` (in flight, final values pending).
- **Blocker Severity**: `P0` (deliverable gate).
- **Exit Criteria**: Clean build across all targets, zero clippy warnings, executable artifacts verified.
- **Remaining blockers**: interactive GUI install smoke; `R-LICENSE` (public release only); unsigned artifacts (accepted); Windows/Linux runtime verification (no host).

---

### V01-07 — Final Release Sign-off, Governance & Tagging

- **Feature**: 0.1.0 Release Governance, Documentation Alignment & Tagging.
- **Current State**: `BLOCKED` (2026-09-14) — **not blocked on build**.
  - **The build/package prerequisite is satisfied:** run `34859158012` was green end to end for
    `1a0c186` (all three platform builds and packages + `SHA256SUMS.txt`), so the exit criterion's
    "all release gates green, zero open P0/P1 items" is no longer blocked by CI. The final run
    `34860902181` for the current HEAD `85a7fa3` is in flight; once it completes, its values must be
    recorded in place of `PENDING_FINAL_CI_RUN_34860902181` before any tag decision.
  - **Blocked by**: (1) `R-LICENSE` — no LICENSE file and no license metadata anywhere; public
    distribution must not be represented as licensed until the user decides (goal-3 §11);
    (2) the **interactive GUI install smoke** — window render, Settings, SQLite connection
    creation, `SELECT 1;` and clean close are `NOT VERIFIED` (`R-GUI-SMOKE`;
    `docs/release/evidence/v01-06/14-install-smoke.txt` §7.7 records the four environment blockers;
    its §8 is the runbook); (3) the V01-01…V01-05 runtime evidence gaps from §3 above (these are
    not closed by a green build run).
  - **Done in this run** (does not unblock): documentation alignment, governance/risk register
    rewrite, packaging contract rewrite, release notes, README, CHANGELOG, handoff document, release
    checklist, readiness assessment, provider capability matrix corrections.
  - **Not done / not authorised**: no tag, no public release, no signing, no license selection.
- **Exact Verification Needed**:
  - [x] Green release pipeline (build/package gate) — run `34859158012` green end to end for
    `1a0c186`; run `34860902181` in flight for HEAD `85a7fa3` (final values
    `PENDING_FINAL_CI_RUN_34860902181`).
  - [x] Update `docs/plans/STATUS.md` — done; **no** v0.1 plan was legitimately eligible for `COMPLETED`, so none was mass-completed.
  - [x] Archive verified plan folders — **none archived**: no feature reached `COMPLETED` (see §3 and `STATUS.md`). Re-evaluate after the runtime evidence gaps close.
  - [x] Record governance risks in `docs/release/risk-register.md` (Brand `R-001`, License `R-LICENSE`, Signing `R-003`, SSH `R-009`) — done, and extended with `R-GUI-SMOKE`, `R-INSTALL-NOTE`, `R-KEYRING-STALL` and `R-CI-PREFLIGHT`.
  - [ ] Update `docs/release/0.1.0-readiness.md` to `READY_FOR_RELEASE: YES` — **not done and deliberately not claimable**: `READY_FOR_RELEASE` is recorded as *internal/private RC qualification* only; public distribution is **NO** pending `R-LICENSE`.
  - [ ] Close the interactive GUI install smoke — `14-install-smoke.txt` §8 runbook; currently `NOT VERIFIED` (§7.7).
  - [ ] Resolve `R-LICENSE` — user decision required.
  - [ ] Tag git commit `v0.1.0` and publish release notes — not authorised in this run; also requires the final run `34860902181` to complete and its values to be recorded first.
- **Provider Scope**: Project Repository & Governance.
- **Evidence Location**: `docs/release/0.1.0-readiness.md`, `docs/release/0.1.0-handoff.md`, Git release tag (not created).
- **Blocker Severity**: `P0` (release closure).
- **Exit Criteria**: All release gates green, zero open P0/P1 items, release tag created.

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
