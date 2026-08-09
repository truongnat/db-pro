# DB Pro — Plans Index

**Status updated:** 2026-08-09  
**Current status authority:** [`07-current-status.md`](07-current-status.md)

---

## Plan Files

| # | File | Description | Status role |
|---|---|---|---|
| 00 | `00-index.md` | Index and phase overview | CURRENT |
| 01 | `01-overview.md` | Master product/release plan | CURRENT |
| 02 | `02-backend-tasks.md` | Detailed Rust backend task definitions | HISTORICAL / REFERENCE |
| 03 | `03-frontend-tasks.md` | Detailed React/TS frontend task definitions | HISTORICAL / REFERENCE |
| 04 | `04-testing-tasks.md` | Testing strategy/task definitions | HISTORICAL / REFERENCE |
| 05 | `05-cicd-tasks.md` | CI/CD/build/release task definitions | HISTORICAL / REFERENCE |
| 06 | `06-database-tasks.md` | Database/connector/meta-store task definitions | HISTORICAL / REFERENCE |
| 07 | `07-current-status.md` | Authoritative live implementation + release status | CURRENT / AUTHORITATIVE |

> The detailed `02–06` files were created before several architecture and release-scope changes. Their unchecked rows are **not** a reliable measure of current completion. Use `07-current-status.md` for live status and keep `02–06` as implementation history/backlog reference.

---

## Current Phase Status

| Phase | Name | Status | Notes |
|---|---|---|---|
| 0 | Scaffolding | DONE | Rust/Tauri/frontend workspace established |
| 1 | Core Domain + Ports | DONE | Domain types and ports implemented |
| 2 | Infrastructure Adapters | DONE | PostgreSQL, SQLite, secrets, metadata implemented |
| 3 | Application Services | DONE | Connection/query/schema/export/data services implemented |
| 4 | Tauri Commands | DONE | Frontend/backend command boundary implemented |
| 5 | FE Scaffolding | DONE | React/Vite/TanStack/shadcn shell implemented |
| 6 | FE Core Utilities | DONE | DI, stores, errors, i18n, Action Platform foundation |
| 7 | Connection Module | DONE | 0.1.0 source scope implemented |
| 8 | Query Module | DONE | 0.1.0 source scope implemented |
| 9 | Schema Module | DONE | DB Object inspection/workbench implemented |
| 10 | DataGrid Module | DONE / PARTIAL historical | Read/update/delete release scope done; insert deferred |
| 11 | Export Module | DONE (release subset) | Runtime smoke still required |
| 12 | Advanced Features | PARTIAL / DEFERRED | Not required in full for 0.1.0 |
| 13 | Testing | BLOCKED | Current release SHA still needs a fully green suite |
| 14 | CI/CD + Packaging | BLOCKED | Workflow exists; release matrix/artifacts not yet proven |

---

## Release Train Status

| Track | Status |
|---|---|
| Action Platform 6.1 | CLOSED |
| Action Platform 6.2 | CLOSED |
| Action Platform 6.3 | CLOSED |
| Wave A — interaction consistency | CLOSED in source |
| Wave B — core usability/data safety | Source mostly closed; verification pending |
| Wave C / new productivity features | HOLD until 0.1.0 release gates close |
| MCP / Agent execution | POST-0.1 |

---

## Quick Reference — Original Task Ranges

| Area | Tasks |
|---|---|
| Backend (Rust) | B-001 to B-105 |
| Frontend (React/TS) | F-001 to F-150 |
| Database Layer | D-001 to D-107 |
| Testing | T-001 to T-078 |
| CI/CD | C-001 to C-060 |

## Reference Documents

| Doc | Path |
|---|---|
| Current Status | `plans/07-current-status.md` |
| Release Readiness | `docs/release/0.1.0-readiness.md` |
| Release Checklist | `docs/release/0.1.0-release-checklist.md` |
| Verification | `docs/release/0.1.0-verification.md` |
| Manual Smoke | `docs/release/0.1.0-manual-smoke.md` |
| Features | `docs/01-features.md` |
| Rules | `docs/02-rules.md` |
| Types | `docs/03-types.md` |
| System Design | `docs/04-system-design.md` |
| BE Architecture | `docs/05-be-architecture.md` |
| DB Architecture | `docs/06-db-architecture.md` |
| FE Architecture | `docs/07-fe-architecture.md` |
| Technology Decisions | `docs/08-technology-decisions.md` |
| Architecture Decisions | `docs/09-architecture-decisions.md` |
