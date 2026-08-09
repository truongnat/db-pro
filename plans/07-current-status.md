# DB Pro — Current Project Status

**Updated:** 2026-08-09  
**Code baseline reviewed:** `669d1fa`  
**Release target:** `0.1.0` Release Candidate  
**Status authority:** this file + `docs/release/0.1.0-readiness.md`

> Documentation-only status commits may exist after `669d1fa`. `669d1fa` is the latest behavior-changing code baseline reviewed for this report. Detailed `02–06` plan files are historical task definitions; do not infer live completion from their unchecked rows.

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
| Schema mutation / users/roles | DEFERRED | post-0.1 |
| Agent | PREVIEW | production Agent execution excluded |
| MCP | DEFERRED | not shipped in 0.1.0 |
| Packaging workflow | DONE definition | current release artifacts not proven |
| Release verification | BLOCKED | current release SHA lacks complete green evidence |

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
| 13 | Testing | BLOCKED / verification incomplete |
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

**SOURCE MOSTLY CLOSED; RELEASE VERIFICATION BLOCKED.**

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

No Wave C/MCP work should begin before release gates close.

---

## Current Release Blockers (P1)

### P1-1 — Frontend suite not fully green/proven at release baseline

Latest behavior-changing commit `669d1fa` explicitly records a pre-existing `tab-factories` test failure. The older `1000/1000` verification predates Wave A/B.

**Exit:** run complete current suite, fix the failing test/root behavior, record exact Test Files / Tests / Passed / Failed with 0 failures.

### P1-2 — `format:check` / release preflight unresolved

Release workflow runs `npm run format:check`; historical verification recorded broad Prettier drift.

**Exit:** make `format:check` PASS at the final SHA, or intentionally change and document the release quality policy.

### P1-3 — Cross-platform release artifacts not proven

Workflow exists but no current release evidence proves macOS + Windows + Linux Tauri artifacts.

**Exit:** green release matrix and retained artifacts for all three platforms.

### P1-4 — Manual desktop runtime smoke pending

Verify startup/reconnect, PostgreSQL error paths, SQLite Browse, Explorer refresh, Data-first navigation, Run/cancel, destructive confirmation/read-only, staged multi-cell updates/deletes, copy isolation, workspace restore and packaged-app stability.

**Exit:** complete `docs/release/0.1.0-manual-smoke.md` and record failures/sign-off.

---

## Known P2 / Post-0.1 Debt

- Unsigned release artifacts unless signing is added.
- SSH tunnel not yet E2E-qualified across release targets.
- Complete row insertion deferred.
- Advanced schema mutation/users/roles deferred.
- Agent remains Preview; MCP deferred.
- JSON cell inspection/context-menu accessibility can improve.
- Clipboard failure feedback can improve.
- Historical coverage percentage targets need re-measurement.
- Public project license is not defined.

---

## Required Final Verification

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

Then:

1. Run the Tauri Release Build matrix.
2. Install/run at least the host packaged artifact.
3. Complete manual smoke + screenshots.
4. Update verification/readiness to the exact tag candidate SHA.
5. Only then tag `v0.1.0`.

---

## Release Decision

**READY_FOR_RELEASE: NO**

The intended 0.1.0 feature scope is sufficiently implemented to stop feature development and enter final RC verification. The remaining work is release evidence, not another feature wave.
