# DB Client — Plans Index

---

| # | File | Description |
|---|---|---|
| 00 | `00-index.md` | This file — index of all plans |
| 01 | `01-overview.md` | Master plan: phases, milestones, timeline, risk register |
| 02 | `02-backend-tasks.md` | Detailed Rust backend tasks (105 tasks across 5 phases) |
| 03 | `03-frontend-tasks.md` | Detailed React/TS frontend tasks (162 tasks across 10 phases) |
| 04 | `04-testing-tasks.md` | Testing strategy and tasks (78 tasks across Rust + TS + E2E) |
| 05 | `05-cicd-tasks.md` | CI/CD, linting, build, release infrastructure (60 tasks) |
| 06 | `06-database-tasks.md` | Database connector, crypto, meta-store tasks (107 tasks) |

## Quick Reference

### By Phase

| Phase | Name | Tasks |
|---|---|---|
| 0 | Scaffolding | B-001 to B-005, F-001 to F-016 |
| 1 | Core Domain + Ports | B-006 to B-028 |
| 2 | Infrastructure Adapters | B-029 to B-060 |
| 3 | Application Services | B-061 to B-078 |
| 4 | Tauri Commands | B-089 to B-105 |
| 5 | FE Scaffolding | F-001 to F-016 |
| 6 | FE Core Utilities | F-017 to F-041 |
| 7 | Connection Module (CO) | F-042 to F-057 |
| 8 | Query Module (QU) | F-058 to F-075 |
| 9 | Schema Module (SC) | F-076 to F-086 |
| 10 | DataGrid Module (DG) | F-087 to F-105 |
| 11 | Export Module (EX) | F-106 to F-113 |
| 12 | Advanced Features | F-114 to F-150 |
| 13 | Testing | F-151 to F-156, T-001 to T-078 |
| 14 | CI/CD + Packaging | C-001 to C-060 |

### By Feature Area

| Area | Tasks |
|---|---|
| Backend (Rust) | B-001 to B-105 |
| Frontend (React/TS) | F-001 to F-150 |
| Database Layer | D-001 to D-107 |
| Testing | T-001 to T-078 |
| CI/CD | C-001 to C-060 |

### By Module

| Module | Screen IDs | Tasks |
|---|---|---|
| Connection (CO) | CO01001, CO01002, CO03001, CO03002 | F-042 to F-057 |
| Query (QU) | QU01001-QU02031 | F-058 to F-075 |
| Schema (SC) | SC01001-SC02013 | F-076 to F-086 |
| DataGrid (DG) | DG01001-DG03012 | F-087 to F-105 |
| Export (EX) | EX01001-EX01005 | F-106 to F-113 |
| Advanced | All remaining | F-114 to F-150 |

## Reference Documents

| Doc | Path |
|---|---|
| Features | `docs/01-features.md` |
| Rules | `docs/02-rules.md` |
| Types | `docs/03-types.md` |
| System Design | `docs/04-system-design.md` |
| BE Architecture | `docs/05-be-architecture.md` |
| DB Architecture | `docs/06-db-architecture.md` |
| FE Architecture | `docs/07-fe-architecture.md` |
