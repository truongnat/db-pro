# DB Client — Master Plan

---

## 1. Project Summary

OPASS Fab DB Client — a Tauri 2 desktop app (Rust core + React WebView) for Ubuntu that provides DBeaver-like database management capabilities for PostgreSQL (primary) and SQLite. Built on OPASS Fab's existing Clean Architecture conventions.

## 2. Reference Documents

| # | File | Purpose |
|---|---|---|
| 01 | `docs/01-features.md` | Feature list with Screen-IDs |
| 02 | `docs/02-rules.md` | Coding standards and quality gates |
| 03 | `docs/03-types.ts` | TypeScript types synced with Rust |
| 04 | `docs/04-system-design.md` | System design, runtime, deployment |
| 05 | `docs/05-be-architecture.md` | Backend architecture (Rust) |
| 06 | `docs/06-db-architecture.md` | Database layer architecture |
| 07 | `docs/07-fe-architecture.md` | Frontend architecture (React/TS) |

## 3. Phases

| Phase | Focus | Duration (est.) | Deliverables |
|---|---|---|---|
| **Phase 0** | Project scaffolding | 2 days | Cargo.toml, package.json, tauri.conf.json, folder structure |
| **Phase 1** | Core domain + ports | 3 days | Rust domain types, port traits, meta-store |
| **Phase 2** | Infrastructure adapters | 4 days | PostgresConnector, SQLiteConnector, KeyringVault, SQLiteMetaStore |
| **Phase 3** | Application services | 4 days | ConnectionService, QueryService, SchemaService, ExportService |
| **Phase 4** | Tauri commands | 3 days | Command handlers, DTO mapping, error mapping |
| **Phase 5** | Frontend scaffolding | 3 days | Vite+React+TS setup, DI container, routing, providers |
| **Phase 6** | FE core utilities | 2 days | API wrapper, error handling, stores, i18n |
| **Phase 7** | Connection module (CO) | 3 days | Connection list, editor, test, connect/disconnect |
| **Phase 8** | Query module (QU) | 5 days | SQL editor, result grid, explain plan, transaction, history |
| **Phase 9** | Schema module (SC) | 3 days | Schema tree, table details, DDL viewer, ERD |
| **Phase 10** | DataGrid module (DG) | 6 days | CRUD grid, inline edit, add/delete row, filter, sort, paginate |
| **Phase 11** | Export module (EX) | 3 days | CSV/JSON/Excel export, import CSV |
| **Phase 12** | Advanced features | 5 days | SSH tunnel, DDL editor, user/role mgmt, backup/restore |
| **Phase 13** | Testing | 4 days | Unit tests, integration tests, E2E tests, coverage |
| **Phase 14** | CI/CD + packaging | 3 days | GitHub Actions, pre-commit hooks, .deb + AppImage build |

**Total estimated: ~47 working days**

## 4. Milestones

| Milestone | Phase | Gate |
|---|---|---|
| M1: Scaffold complete | Phase 0 | `cargo build` + `npm run dev` both pass |
| M2: Core domain + ports done | Phase 1 | All domain types compile, all traits defined |
| M3: Infrastructure adapters done | Phase 2 | PostgresConnector connects to test DB |
| M4: Application services done | Phase 3 | All 4 services unit-tested |
| M5: Tauri commands working | Phase 4 | Frontend can invoke all commands |
| M6: FE scaffold ready | Phase 5 | React app loads with routing |
| M7: CO module complete | Phase 7 | Connection CRUD works end-to-end |
| M8: QU module complete | Phase 8 | SQL editor + result grid functional |
| M9: DG module complete | Phase 10 | Full CRUD grid functional |
| M10: All features complete | Phase 12 | All features from 01-features.md implemented |
| M11: Testing complete | Phase 13 | Coverage ≥ 80% Rust, ≥ 70% TS |
| M12: CI/CD + packaging | Phase 14 | `.deb` + `.AppImage` build successfully |

## 5. Dependency Graph (Phases)

```
Phase 0 → Phase 1 → Phase 2 → Phase 3 → Phase 4
                                              ↓
Phase 5 → Phase 6 → Phase 7 → Phase 8 → Phase 9 → Phase 10 → Phase 11 → Phase 12
                                                                                    ↓
Phase 13 (testing) ←———————————————————————————————————————————————————————————————+
                                                                                    ↓
Phase 14 (CI/CD + packaging) ←————————————————————————————————————————————————————+
```

## 6. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Tauri 2 API changes | Medium | High | Pin exact Tauri version, monitor changelog |
| PostgreSQL introspection queries fragile | Medium | Medium | Use information_schema (standard), test against PG 14+ |
| Monaco editor integration complexity | Medium | Medium | Start with basic editor, add autocomplete incrementally |
| Virtualized grid performance | Low | High | Use @tanstack/react-virtual from day 1, benchmark early |
| SSH tunnel (openssh) reliability | Medium | Medium | Test on target Ubuntu versions, handle tunnel disconnect gracefully |
| Keyring integration on different DEs | Medium | Medium | Test on GNOME and KDE, fallback to file-based encryption |
| Feature scope creep | High | High | Stick to MVP features first, defer advanced features to later phases |
