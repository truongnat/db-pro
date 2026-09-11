# DB Pro — Current Project Status

**Updated:** 2026-08-11 (amended 2026-09-11)  
**Code baseline reviewed:** `a2ce14c`  
**Release target:** `0.1.0` Release Candidate  
**Status authority:** this file + `docs/release/0.1.0-readiness.md`

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
| Data Grid insert | DEFERRED | not a complete 0.1.0 workflow |
| Export | DONE release subset | manual smoke required |
| SSH tunnel | PARTIAL | plumbing exists; cross-platform E2E not complete |
| Schema mutation / users/roles | PARTIAL | Column editing workbench shipped (P2.7); full schema mutation/users/roles post-0.1 |
| Agent | PREVIEW | production Agent execution excluded |
| MCP | DEFERRED | not shipped in 0.1.0 |
| UI quality gates | NATIVE | React/TS typecheck/lint/format gates retired with the archived frontend; native UI now gated by `cargo fmt/check/clippy/test` + `cargo build --release -p db-pro-native` |
| P2 Hardening Program | DONE | P2.0–P2.11 all complete; see docs/quality/p2-hardening-code-audit.md |
| Packaging workflow | DONE definition | cross-platform build IN PROGRESS |
| Release verification | IN PROGRESS | exact-SHA automated verification DONE; artifacts + manual smoke pending |

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

**SOURCE CLOSED FOR THE 0.1.0 SCOPE; RELEASE VERIFICATION PENDING.**

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

### P1-1 — Exact-SHA automated verification is incomplete

The known `tab-factories` test failure has been fixed in `120b250`. The former frontend
gates (`frontend/pnpm-lock.yaml`, pnpm typecheck/lint/format/test/build) were retired when
the React frontend was archived, so they no longer gate the release. Rust gates still need
an exact-SHA rerun on the current HEAD.

**Exit:** run and record exact results with 0 failures:

```bash
cargo fmt --all --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release --locked -p db-pro-native
```

Record exact test counts. Do not carry forward old counts.

### P1-2 — Cross-platform release artifacts not proven

The release workflow now builds the native `db-pro-native` binary on macOS, Windows, and
Linux, but no current run proves the matrix is green. Installer/packaging formats
(DMG, MSI/NSIS, DEB/RPM/AppImage) and code signing are not implemented yet for the native
app.

**Exit:** green release matrix and retained `db-pro-native` artifacts for all three
platforms, plus a documented packaging decision.

### P1-3 — Manual desktop runtime smoke pending

Verify startup/reconnect, PostgreSQL error paths, SQLite Browse, Explorer refresh, Data-first navigation, Run/cancel, destructive confirmation/read-only, staged multi-cell updates/deletes, copy isolation, workspace restore and packaged-app stability.

**Exit:** complete `docs/release/0.1.0-manual-smoke.md` and record failures/sign-off.

---

## Closed Release Blockers

- `tab-factories` known failure root cause: **FIXED** at `120b250`.
- Frontend Prettier drift / `format:check`: **FIXED** at `912588f` (253 files normalized; local check reported PASS). *Historical — the frontend was archived on 2026-09-11.*
- Darwin-only Rollup root dependency: **FIXED** by portable lockfile regeneration. *Historical — frontend archived.*

---

## Known P2 / Post-0.1 Debt

- Unsigned release artifacts unless signing is added.
- SSH tunnel not yet E2E-qualified across release targets.
- Complete row insertion deferred.
- Advanced schema mutation/users/roles deferred (column editing workbench shipped in P2.7).
- Agent remains Preview; MCP deferred.
- JSON cell inspection/context-menu accessibility can improve.
- Clipboard failure feedback can improve.
- Historical coverage percentage targets need re-measurement.
- Public project license is not defined.
- Grid context menu uses a custom fixed overlay rather than a shared menu widget (future improvement).
- Query key stale time could be tuned for introspection data (low-risk).

---

## Final Release Sequence

1. Run exact-SHA full Rust automated verification and record exact counts.
2. Trigger the native Release Build matrix (`db-pro-native`).
3. Retain/download macOS, Windows, and Linux artifacts.
4. Install/run at least the host artifact.
5. Complete manual smoke + screenshots.
6. Update verification/readiness to the final tag candidate SHA.
7. Only then tag `v0.1.0`.

---

## Release Decision

**READY_FOR_RELEASE: NO**

**Current P1 blockers: 3.**

The intended 0.1.0 feature scope is sufficiently implemented and Wave A/B source work is closed for release scope. Remaining work is exact-SHA automated verification, cross-platform artifact proof, and runtime/manual sign-off — not another feature wave.
