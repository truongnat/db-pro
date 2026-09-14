# DB Pro — Current Project Status

**Updated:** 2026-08-11 (amended 2026-09-11; V01-06 status correction 2026-09-14)  
**Code baseline reviewed:** `a2ce14c` (historical frontend-era pass) / `main@7794196` for the 2026-09-14 correction  
**Release target:** `0.1.0` Release Candidate  
**Status authority:** this file + `docs/plans/STATUS.md` + `docs/release/0.1.0-readiness.md`

> **Correction (2026-09-14 — V01-06).** The V01-01…V01-05 `PASS` claims are
> `EVIDENCE_GAP` / `PARTIAL`, not verified (see
> `docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md`). The current measured
> workspace result on `main` is **815 passed / 0 failed / 19 ignored** (811 before the
> `543b526` state-directory fix) — the 19 are
> `#[ignore]`d (18 PostgreSQL integration + 1 SSH backup) and are never "passing". The
> release build and the six quality gates are green on the host (macOS ARM64); the
> **final release run `34860902181` (candidate `85a7fa3`) is green end to end** — all three
> platforms build and package, and the archives + `SHA256SUMS.txt` were independently
> re-hashed (runs `34845148946` / `34847235273` / `34849517304` / `34851704292` /
> `34859158012` are superseded history). Row-level
> corrections are marked `[CORRECTED 2026-09-14]` below.

> **Amendment (2026-09-11) — native UI direction.** The React/TypeScript/Vite frontend and
> its Tauri WebView host were retired. The UI is now native `eframe`/`egui`
> (`crates/ui` + `crates/native-app`), the frontend is archived under `_archive/frontend/`,
> and CI/release no longer run any Node or pnpm step. Rows below that mention React,
> TypeScript, Vite, Monaco, TanStack, shadcn, Radix, or frontend test counts describe the
> **archived** presentation layer and are retained as history. Live UI status is in
> `docs/plans/STATUS.md`.

> `a2ce14c` fixes Rust 1.96.0 clippy `needless_borrow` lint (release preflight blocker). PR #9 (ER Diagram IA refactor) merged at preceding commit. Frontend: 106 files, 1324 tests. Exact-SHA automated verification complete. Cross-platform artifacts and manual smoke in progress.

---

## Executive Status

| Area | Status | Notes |
|---|---|---|
| Rust/backend foundation | DONE | Core/application/ports/infrastructure/Tauri boundary implemented |
| PostgreSQL | DONE source | Runtime smoke required |
| SQLite | DONE source | Native Browse/plugin/capability wired; runtime smoke required |
| Connection lifecycle | DONE source | CRUD/test/connect/disconnect/reconnect/startup reconnect |
| Explorer / metadata | DONE source | schemas/tables/views, targeted refresh, Data-first navigation |
| Query workbench | DONE source | Native SQL editor; current/selection/all, cancel, explain, format, history |
| Action Platform | DONE | canonical execution, confirmation, cancellation identity |
| DB Object workbench | DONE source | Data, Columns, Indexes, Relations, Triggers, DDL inspection |
| ER Diagram | DONE source | Schema-level workspace tab (PR #9); composite FK; position persistence |
| Data Grid read/productivity | DONE source | virtualized rows, filter/sort/page, resize, selection/copy |
| Data Grid update/delete | DONE source | PK staged patch mutations + revision-safe apply model |
| Data Grid insert | SHIPPED (narrow) | **[CORRECTED 2026-09-14]** row insert **is** wired in the native Table Data Editor (staged insert + dialogs, `crates/ui/src/table_editor_view.rs`); it is narrower than a full insert workflow because complex column types (JSON/array/UUID) have incomplete input widgets — LIM-003. The earlier "DEFERRED / not a complete 0.1.0 workflow" wording was inaccurate for the native UI |
| Export | DONE release subset | **[CORRECTED 2026-09-14]** the native UI uses its own local CSV/TSV writer from the result grid; the richer backend exporters (CSV/JSON/XLSX) have no native trigger — see `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §4 |
| SSH tunnel | PARTIAL | plumbing exists; cross-platform E2E not complete; no host to verify on |
| Schema mutation / users/roles | PARTIAL | Column editing workbench shipped (P2.7); full schema mutation/users/roles post-0.1 |
| Agent | PREVIEW (ships) | **[CORRECTED 2026-09-14]** the Agent panel **ships in 0.1.0 as Preview** (Ask/Edit/Agent, confirmation-gated). What is excluded is production/autonomous Agent execution, not the panel. Both statements are true and are stated separately here |
| MCP | DEFERRED | not shipped in 0.1.0 |
| UI quality gates | NATIVE | React/TS typecheck/lint/format gates retired with the archived frontend; native UI now gated by `cargo fmt/check/clippy/test` + `cargo build --release -p db-pro-native` |
| P2 Hardening Program | DONE | P2.0–P2.11 all complete; see docs/quality/p2-hardening-code-audit.md *(frontend-era program; the RC1 P2 findings against the native UI are tracked separately and are not closed — see `06-rc1-p2-dispositions.md`)* |
| Packaging workflow | DONE + VERIFIED IN CI | **[CORRECTED 2026-09-14]** portable-archive contract implemented (macOS `.app` tar.gz, Windows zip, Linux tar.gz + `SHA256SUMS.txt`); **final run `34860902181` green end to end** for the candidate `85a7fa3`, with all three archives independently re-hashed; installers and signing are DEFERRED, not "in progress" |
| Release verification | **[CORRECTED 2026-09-14]** six gates + release build green on host (815 passed / 0 failed / 19 ignored, `12-state-dir-blocker-fix.txt` §4); final run `34860902181` green end to end with independently re-hashed artifacts; runtime smoke remains `NOT VERIFIED` for the GUI half and V01-01…05 are audited `EVIDENCE_GAP`/`PARTIAL` |

---

## Phase Status

| Phase | Focus | Status |
|---|---|---|
| 0 | Scaffolding | DONE |
| 1 | Core domain + ports | DONE |
| 2 | Infrastructure adapters | DONE |
| 3 | Application services | DONE |
| 4 | Tauri commands | DONE |
| 5 | Frontend scaffolding | DONE |
| 6 | FE core utilities / Action foundation | DONE |
| 7 | Connection | DONE source |
| 8 | Query | DONE source |
| 9 | Schema inspection | DONE source |
| 10 | DataGrid | DONE release scope / PARTIAL historical full scope |
| 11 | Export | DONE release subset |
| 12 | Advanced features | PARTIAL / DEFERRED for 0.1 |
| 13 | Testing | DONE source / final full-suite verification DONE at `a2ce14c` |
| 14 | CI/CD + packaging | IN PROGRESS / cross-platform artifacts building |

---

## Release-wave Status

### Action Platform

- 6.1 — CLOSED
- 6.2 — CLOSED
- 6.3 — CLOSED

### Wave A — Interaction consistency

**CLOSED in source.**

Includes split Run interaction, shortcut semantics, Data-first table navigation, connection/status corrections, startup reconnect source flow, SQLite native Browse wiring, explicit Structure/Data semantics and dead/preview-control cleanup.

### Wave B — Core usability / data safety

**SOURCE CLOSED FOR THE 0.1.0 SCOPE; RELEASE VERIFICATION NOT VERIFIED (manual smoke not run).**

Implemented:
- grid column resize/layout persistence
- row selection + copy isolation
- dirty SQL guards
- orphan-tab recovery
- richer Explorer connection actions
- staged destructive confirmation
- exact-ID partial-success/failure cleanup
- patch-style updates that send only changed columns
- same-row patch composition
- in-flight staged revision safety
- targeted metadata refresh
- query context cleanup on connection reassignment
- collision-aware titles / compact pinned tabs
- portable frontend lockfile root dependencies
- collision-title regression test setup corrected (`120b250`)
- frontend Prettier normalization (`912588f`)

> Frontend-specific items above (lockfile root dependencies, Prettier normalization,
> frontend test setup) refer to the archived React frontend and no longer apply.

No Wave C/MCP work should begin before release gates close.

---

## P2 Hardening Program — Complete

All 11 waves of the P2 Hardening Program are complete. See `docs/quality/p2-hardening-code-audit.md` for the full audit report.

| Wave | Focus | Status |
|---|---|---|
| P2.0 | Fixture database (postgres-p2-hardening/) | DONE |
| P2.1 | Theme system/light/dark + text selection audit | DONE |
| P2.2 | Workspace tab IDE context actions | DONE |
| P2.3 | DB Object header action hierarchy | DONE |
| P2.4 | Filter + Sort draft/apply rearchitecture | DONE |
| P2.5 | Data Grid scroll/layout/cell UX | DONE |
| P2.6 | Edit modes + batch delete + transaction feedback | DONE |
| P2.7 | Column/schema editing safety workbench | DONE |
| P2.8 | Index/Relation/Trigger/DDL hardening | DONE |
| P2.9 | ER Diagram (native painter; formerly React Flow + dagre) | DONE |
| P2.10 | Code quality / modern API audit | DONE |
| P2.11 | Full regression (39 Rust integration tests) | DONE |

**Test counts (at the time of the archived-frontend baseline):** 105 frontend test files,
1319 frontend tests + 39 Rust integration tests (30 SQLite + 9 PG), all passing.
Frontend test counts are historical — that suite was retired with the frontend on
2026-09-11. Current gates are Rust-only.

---

## Current Release Blockers (P1)

### P1-1 — Exact-SHA automated verification — **CLOSED at `fbf9fda` / `7794196`** [CORRECTED 2026-09-14]

The six gates were re-run on `main` and all exit 0: `cargo fmt --all -- --check`,
`cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace` (**815 passed / 0 failed / 19 ignored**; 811 before the `543b526` fix),
`cargo build --release --locked -p db-pro-native`, and the perf scan (`PASS 4 / 0 / 0`).
Evidence: `docs/release/evidence/v01-06/02-quality-gates.txt` and
`08-post-fix-quality-gates.txt` §8. The 19 ignored tests are 18 `#[ignore]`d PostgreSQL
integration cases and 1 `#[ignore]`d SSH backup case; they are not passing.

### P1-2 — Cross-platform release artifacts — **CLOSED [CORRECTED 2026-09-14]**

The release workflow builds `db-pro-native` on macOS (`macos-14`), Windows
(`windows-latest`) and Linux (`ubuntu-latest`) and packages portable archives with
`SHA256SUMS.txt`. **The matrix is green**: the final release run **`34860902181`**
(candidate `85a7fa3`) completed end to end — `Build` ×3 ✔, `Package` ×3 ✔, `Assemble
SHA256SUMS` ✔ — and the three archives plus `SHA256SUMS.txt` were downloaded and
independently re-hashed. Windows and Linux are therefore `BUILD_VERIFIED` and remain
`RUNTIME_UNVERIFIED` (no Windows/Linux host exists in this project). Installer formats
(DMG, MSI/NSIS, DEB/RPM/AppImage) and code
signing are **not** implemented for the native app and are DEFERRED out of the v0.1
contract (`docs/release/0.1.0-packaging.md`).

**Exit (met):** green release matrix and retained archives + `SHA256SUMS.txt` for all three
platforms. Final values: `docs/release/0.1.0-readiness.md`, `docs/release/0.1.0-handoff.md`
§3, `docs/release/risk-register.md` §4.

### P1-3 — Manual desktop runtime smoke pending (no retrievable evidence)

`docs/release/0.1.0-manual-smoke.md` is the canonical instrument and has **0 of 165
checklist items ticked** while its sign-off block claims "Passed: 65 / 65 checked
sections". The V01-05 PASS is therefore `EVIDENCE_GAP` (audit §6), and V01-01…V01-04 are
`EVIDENCE_GAP`/`PARTIAL`. Required: perform and record the walkthrough (startup/restart,
PostgreSQL error paths, SQLite Browse, Explorer refresh, Data-first navigation, Run/cancel,
destructive confirmation/read-only, staged multi-cell updates/deletes, copy isolation) and
record results in the exact format the checklist expects, at an exact SHA.

**Exit:** completed `docs/release/0.1.0-manual-smoke.md` with a truthful sign-off, plus
packaged-artifact install smoke (`docs/release/0.1.0-handoff.md` §install smoke).

---

## Closed Release Blockers

- `tab-factories` known failure root cause: **FIXED** at `120b250`.
- Frontend Prettier drift / `format:check`: **FIXED** at `912588f` (253 files normalized; local check reported PASS). *Historical — the frontend was archived on 2026-09-11.*
- Darwin-only Rollup root dependency: **FIXED** by portable lockfile regeneration. *Historical — frontend archived.*

---

## Known P2 / Post-0.1 Debt

- Unsigned release artifacts unless signing is added (`R-003`, accepted for v0.1).
- SSH tunnel not yet E2E-qualified across release targets (`R-009`).
- **[CORRECTED 2026-09-14]** Row insert ships in the native Table Data Editor; complex-type
  input widgets (JSON/array/UUID) are incomplete (LIM-003). It is no longer accurate to say
  "complete row insertion deferred" without that qualification.
- Advanced schema mutation/users/roles deferred (column editing workbench shipped in P2.7).
- Agent ships as Preview; production/autonomous Agent execution and MCP are deferred.
- Workspace tab/settings persistence is **not implemented** in the native build (eframe
  persistence feature is off) — restart recovery of tabs cannot be claimed (`R-015`).
- JSON cell inspection/context-menu accessibility can improve.
- Clipboard failure feedback can improve.
- Historical coverage percentage targets need re-measurement.
- Public project license is not defined (`R-LICENSE`; blocks public distribution).
- Grid context menu uses a custom fixed overlay rather than a shared menu widget (future improvement).
- Query key stale time could be tuned for introspection data (low-risk).

---

## Final Release Sequence

1. Re-run the exact-SHA full Rust automated verification and record exact counts — **DONE**
   (815 passed / 0 failed / 19 ignored at the candidate; six gates exit 0).
2. Trigger the native Release Build matrix (`db-pro-native`) — **DONE**; the final run
   `34860902181` is green end to end for the candidate `85a7fa3` (`34859158012` and the
   earlier attempts are superseded history).
3. Retain/download macOS, Windows, and Linux artifacts — **DONE**; independently re-hashed
   (macOS ARM64 10,045,965 B; Windows x86_64 10,076,835 B; Linux x86_64 15,168,133 B).
4. Install/run at least the host artifact — macOS process launch, state-directory reuse and
   clean SIGTERM exit PASS on the packaged archive, including the CI-produced artifact of
   the final run; the GUI steps remain `NOT VERIFIED` (no GUI automation on this host).
5. Complete manual smoke + screenshots with durable capture paths (in-repo, not temp) —
   **NOT DONE** (0 of 165 items in `docs/release/0.1.0-manual-smoke.md`); runbook in
   `docs/release/evidence/v01-06/14-install-smoke.txt` §8.
6. Update verification/readiness to the final tag candidate SHA — **DONE**; the candidate
   is `85a7fa3` and the record is in `docs/release/0.1.0-readiness.md`,
   `docs/release/0.1.0-handoff.md` and `docs/release/0.1.0-final-report.md`.
7. Only then tag `v0.1.0` — **NOT DONE / not authorised**; recommended sequence is
   `v0.1.0-rc.1` first, then the install smoke, then `v0.1.0` (owner decision).

---

## Release Decision

**READY_FOR_RELEASE (internal / private release candidate qualification): YES** —
the exact-HEAD six gates and release build are green, **the final release run `34860902181`
is green end to end for the candidate `85a7fa3` with independently re-hashed artifacts**,
the release contract is documented, and the open items are governance/platform-coverage
items rather than code blockers.

**READY_FOR_RELEASE (public distribution): NO** — `R-LICENSE` is undecided (no LICENSE
file, no license metadata) and is the binding reason; all artifacts are UNSIGNED, the
interactive GUI install smoke is `NOT VERIFIED`, Windows/Linux runtime is `RUNTIME_UNVERIFIED`
(no host), and the V01-01…V01-05 runtime evidence gaps remain open.

**Open blockers to public distribution:** `R-LICENSE` (user decision); the interactive GUI
install smoke (`R-GUI-SMOKE`); runtime evidence gaps for V01-01…V01-05; Windows/Linux
runtime (`R-WINLINUX`); and the tag decision (`v0.1.0-rc.1` recommended first). The intended
0.1.0 feature scope is sufficiently implemented
and Wave A/B source work is closed for release scope; remaining work is runtime evidence,
governance sign-off and the tag — not another feature wave, and not a build failure.
