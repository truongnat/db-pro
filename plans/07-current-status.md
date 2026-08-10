# DB Pro — Current Project Status

**Updated:** 2026-08-09  
**Code baseline reviewed:** `912588f`  
**Release target:** `0.1.0` Release Candidate  
**Status authority:** this file + `docs/release/0.1.0-readiness.md`

> `120b250` fixed the known `tab-factories` collision-title test setup by using non-preview tabs so collision cases coexist. `912588f` normalized the frontend source with Prettier and cleared the historical format drift. Full Vitest/build/Rust final verification is still required at the exact release SHA.

---

## Executive Status

| Area | Status | Notes |
|---|---|---|
| Rust/backend foundation | DONE | Core/application/ports/infrastructure/Tauri boundary implemented |
| PostgreSQL | DONE source | Runtime smoke required |
| SQLite | DONE source | Native Browse/plugin/capability wired; runtime smoke required |
| Connection lifecycle | DONE source | CRUD/test/connect/disconnect/reconnect/startup reconnect |
| Explorer / metadata | DONE source | schemas/tables/views, targeted refresh, Data-first navigation |
| Query workbench | DONE source | Monaco, current/selection/all, cancel, explain, format, history |
| Action Platform | DONE | canonical execution, confirmation, cancellation identity |
| DB Object workbench | DONE source | Data, Columns, Indexes, Relations, DDL inspection |
| Data Grid read/productivity | DONE source | virtualized rows, filter/sort/page, resize, selection/copy |
| Data Grid update/delete | DONE source | PK staged patch mutations + revision-safe apply model |
| Data Grid insert | DEFERRED | not a complete 0.1.0 workflow |
| Export | DONE release subset | manual smoke required |
| SSH tunnel | PARTIAL | plumbing exists; cross-platform E2E not complete |
| Schema mutation / users/roles | PARTIAL | Column editing workbench shipped (P2.7); full schema mutation/users/roles post-0.1 |
| Agent | PREVIEW | production Agent execution excluded |
| MCP | DEFERRED | not shipped in 0.1.0 |
| Frontend typecheck/lint/format | PASS | 0 typecheck errors; 1319/1319 frontend tests + 39 Rust integration tests pass; Vite build clean |
| P2 Hardening Program | DONE | P2.0–P2.11 all complete; see docs/quality/p2-hardening-code-audit.md |
| Packaging workflow | DONE definition | current release artifacts not proven |
| Release verification | BLOCKED | full automated + artifact + manual runtime evidence incomplete |

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
| 13 | Testing | PARTIAL / final full-suite verification pending |
| 14 | CI/CD + packaging | BLOCKED / artifacts incomplete |

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
| P2.9 | ER Diagram (React Flow + dagre) | DONE |
| P2.10 | Code quality / modern API audit | DONE |
| P2.11 | Full regression (1319 FE + 39 Rust tests, 0 TS errors, build clean) | DONE |

**Test counts:** 105 test files, 1319 frontend tests + 39 Rust integration tests (30 SQLite + 9 PG), all passing.  
**TypeScript:** 0 errors.  
**Vite build:** successful.

---

## Current Release Blockers (P1)

### P1-1 — Exact-SHA automated verification is incomplete

The known `tab-factories` test failure has been fixed in `120b250`, and `typecheck`, `lint`, and `format:check` were reported green after `912588f`. However, the full Vitest suite and production frontend build have not been executed successfully at this exact release-train SHA. Final Rust gates also need an exact-SHA rerun.

**Exit:** run and record exact results with 0 failures:

```bash
cd frontend
rm -rf node_modules
npm ci
npm run typecheck
npm run lint
npm run format:check
npm run test
npm run build

cd ..
cargo fmt --all --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Record exact Test Files / Tests / Passed / Failed. Do not carry forward old counts.

### P1-2 — Cross-platform release artifacts not proven

Workflow exists but no current release evidence proves macOS + Windows + Linux Tauri artifacts.

**Exit:** green release matrix and retained artifacts for all three platforms.

### P1-3 — Manual desktop runtime smoke pending

Verify startup/reconnect, PostgreSQL error paths, SQLite Browse, Explorer refresh, Data-first navigation, Run/cancel, destructive confirmation/read-only, staged multi-cell updates/deletes, copy isolation, workspace restore and packaged-app stability.

**Exit:** complete `docs/release/0.1.0-manual-smoke.md` and record failures/sign-off.

---

## Closed Release Blockers

- `tab-factories` known failure root cause: **FIXED** at `120b250`.
- Frontend Prettier drift / `format:check`: **FIXED** at `912588f` (253 files normalized; local check reported PASS).
- Darwin-only Rollup root dependency: **FIXED** by portable lockfile regeneration.

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
- Grid context menu uses custom `fixed div` rather than Radix ContextMenu (future improvement).
- Query key stale time could be tuned for introspection data (low-risk).

---

## Final Release Sequence

1. Run exact-SHA full automated verification (frontend + Rust) and record exact counts.
2. Trigger the Tauri Release Build matrix.
3. Retain/download macOS, Windows, and Linux artifacts.
4. Install/run at least the host packaged artifact.
5. Complete manual smoke + screenshots.
6. Update verification/readiness to the final tag candidate SHA.
7. Only then tag `v0.1.0`.

---

## Release Decision

**READY_FOR_RELEASE: NO**

**Current P1 blockers: 3.**

The intended 0.1.0 feature scope is sufficiently implemented and Wave A/B source work is closed for release scope. Remaining work is exact-SHA automated verification, cross-platform artifact proof, and runtime/manual sign-off — not another feature wave.
