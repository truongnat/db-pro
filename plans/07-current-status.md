# DB Pro — Current Project Status

**Updated:** 2026-08-09  
**Authoritative remote baseline:** `main @ 669d1fa`  
**Release target:** `0.1.0` Release Candidate  
**Status authority:** This file is the current status source. `02-backend-tasks.md` through `06-database-tasks.md` remain implementation/backlog definitions and must not be treated as live completion trackers.

---

## Status Legend

| Status | Meaning |
|---|---|
| DONE | Implemented in source for the current release scope |
| PARTIAL | Useful implementation exists, but the full historical plan is not complete |
| BLOCKED | Must be resolved before release |
| PENDING | Requires verification/evidence rather than more feature work |
| DEFERRED | Explicitly outside DB Pro 0.1.0 |

---

## Executive Status

| Area | Status | Notes |
|---|---|---|
| Rust architecture / backend foundation | DONE | Core/application/ports/infrastructure/Tauri command boundary implemented |
| PostgreSQL | DONE | Release scope implemented; runtime smoke still required |
| SQLite | DONE | Native file picker and dialog plugin are wired; runtime smoke still required |
| Connection lifecycle | DONE | Create/test/connect/disconnect/reconnect/delete + startup reconnect source path implemented |
| Explorer / metadata | DONE | Schemas/tables/views, targeted refresh, Data-first table navigation |
| Query workbench | DONE | Monaco, current statement/selection/all, cancel, explain, format, history |
| Action Platform | DONE | Canonical action runtime, confirmation lifecycle, cancellation identity and audit source implemented |
| DB Object workbench | DONE | Data, Columns, Indexes, Relations, DDL inspection |
| Data Grid read path | DONE | Virtualized rows, filter/sort/page, column resize, row selection/copy, persisted layout |
| Data Grid update/delete | DONE (source) | PK-based staged patch updates/deletes with revision-safe apply model; exact HEAD test/runtime verification still pending |
| Data Grid insert | DEFERRED | Store type exists, but release UI/apply path is not wired as a complete feature |
| Export | DONE (release subset) | Query result export path implemented; verify in manual smoke |
| SSH tunnel | PARTIAL | Backend/plumbing exists; end-to-end release-target verification is not complete |
| Schema mutation / DDL execution UI | DEFERRED | Inspection ships; advanced mutation is post-0.1 |
| Users / roles | DEFERRED | Hidden from 0.1 navigation |
| Agent workspace | PARTIAL / PREVIEW | Preview only; no production Agent execution |
| MCP server | DEFERRED | Explicitly excluded from 0.1.0 |
| Packaging workflow | DONE (definition) | macOS/Windows/Linux workflow exists; artifacts have not been proven at current HEAD |
| Release verification | BLOCKED | Current HEAD has no complete green verification evidence |

---

## Phase Status

| Phase | Original focus | Current status | Release interpretation |
|---|---|---|---|
| 0 | Scaffolding | DONE | Workspace/build structure exists |
| 1 | Core domain + ports | DONE | Stable Rust domain/port foundation |
| 2 | Infrastructure adapters | DONE | PostgreSQL, SQLite, metadata, secrets implemented |
| 3 | Application services | DONE | Connection/query/schema/export/data services implemented |
| 4 | Tauri commands | DONE | Frontend/backend command boundary implemented |
| 5 | Frontend scaffolding | DONE | React/Vite/TanStack/shadcn application shell exists |
| 6 | FE core utilities | DONE | Stores, DI, errors, i18n, Action Platform foundation |
| 7 | Connection module | DONE | Release source scope complete |
| 8 | Query module | DONE | Release source scope complete |
| 9 | Schema module | DONE | Inspection/workbench release scope complete |
| 10 | DataGrid module | DONE / PARTIAL historical | Read/update/delete release scope implemented; row insert remains deferred |
| 11 | Export | DONE (release subset) | Runtime smoke still required |
| 12 | Advanced features | PARTIAL | Intentionally not required in full for 0.1.0 |
| 13 | Testing | PARTIAL / BLOCKED | Large suite exists, but current HEAD is not yet demonstrated fully green |
| 14 | CI/CD + packaging | PARTIAL / BLOCKED | Workflow exists; cross-platform artifacts and current-HEAD preflight evidence pending |

---

## Wave Status

### Wave A — Interaction / release UX closure

**Status: DONE in source.**

Closed areas include:
- Run split-button interaction
- Shortcut semantics (`Cmd/Ctrl+P`, command palette, query execution)
- Data-first table navigation
- Session reconnect source flow
- Connection/status semantics
- SQLite native Browse wiring
- Dead/preview controls removed or clearly labeled
- Object refresh semantics

Runtime smoke remains part of the release gate, not Wave A feature work.

### Wave B — Core usability / data safety

**Status: SOURCE MOSTLY DONE; RELEASE VERIFICATION BLOCKED.**

Implemented:
- Grid column resizing/layout persistence
- Row selection/copy
- Dirty SQL overwrite guards
- Orphan-tab recovery
- Explorer connection context menu
- Staged mutation confirmation and partial-failure handling
- Stable/revision-safe staged mutation IDs
- Patch-style row updates (only changed columns are sent)
- Same-row patch composition (`{...previousChanges, ...newChanges}`)
- Targeted metadata refresh
- Query-context reset on connection reassignment
- Collision-aware tab titles and compact pinned tabs
- Portable frontend lockfile root dependencies

Remaining work is verification and a small known test issue, not a new UX feature wave.

---

## Current Release Blockers (P1)

### P1-1 — Current frontend suite is not proven green

The latest remote commit `669d1fa` explicitly records a pre-existing `tab-factories` test failure. The historical verification report (`92d9db9`) cannot be used as evidence for current HEAD after Wave A/B changes.

**Exit:** run the complete frontend suite at current HEAD and get **0 failed tests**. Fix the `tab-factories` failure rather than excluding it.

### P1-2 — `format:check` must match the release workflow

The release workflow runs `npm run format:check`. Existing release documentation records broad Prettier drift. Unless this is resolved, the preflight job can fail even if runtime code is correct.

**Exit:** either format the checked frontend source and commit it, or deliberately change the release quality gate with an explicit decision. Do not leave docs claiming green while CI is expected to fail.

### P1-3 — No cross-platform release artifact evidence at current HEAD

The workflow definition exists, but current `main` has no attached green status evidence proving macOS, Windows and Linux Tauri builds.

**Exit:** execute Release Build and retain successful artifacts/logs for all target platforms.

### P1-4 — Manual runtime smoke has not been signed off

Source review cannot prove desktop runtime behavior. Before tagging, verify at minimum:
- startup/restart/session reconnect
- PostgreSQL good/bad connection paths
- SQLite Browse native dialog
- Explorer metadata refresh
- table single-click → Data preview; double-click → permanent Data
- Run current/selection/all + Stop/Escape
- destructive confirmation/read-only protection
- Data Grid multi-cell same-row patch composition and update/delete apply
- row selection → Cmd/Ctrl+C without stealing input/editor copy
- workspace restore and no stale/orphan crash

**Exit:** complete `docs/release/0.1.0-manual-smoke.md` on a packaged/dev Tauri runtime and record results.

---

## Known P2 / Post-release Debt

- Release artifacts are unsigned (macOS/Windows warning expected).
- SSH tunnel is not yet proven end-to-end across release targets.
- Row insertion is not a complete 0.1.0 workflow.
- Schema mutation/DDL execution UI is deferred.
- Agent workspace remains Preview; MCP is not shipped.
- JSON cell inspection/context-menu accessibility can be improved.
- Clipboard failure feedback can be improved.
- Historical coverage targets (Rust >=80%, TS >=70%) have not been re-measured at current HEAD.
- Public project license is still not defined.

---

## Verification Evidence Policy

Do not carry forward a PASS from an older commit after behavior-changing commits.

For release sign-off, record results against the exact release SHA for:

```bash
cd frontend
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

Then run the Tauri release workflow matrix and manual smoke.

---

## Release Decision

**READY_FOR_RELEASE: NO**

The product is feature-complete enough for the intended `0.1.0` scope, but it is not yet release-evidence complete. Do not start Wave C or MCP work before the current test/preflight/artifact/runtime gates are green.
