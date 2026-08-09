# DB Pro — Master Plan

**Updated:** 2026-08-09  
**Release target:** `0.1.0`  
**Current status source:** [`07-current-status.md`](07-current-status.md)

---

## 1. Product Summary

DB Pro is a Tauri 2 desktop Database IDE built with a Rust backend and React/TypeScript frontend. The `0.1.0` release scope targets PostgreSQL and SQLite on macOS, Windows, and Linux.

The product direction is **Database IDE + Agent-ready architecture**, not a web-admin UI. Human UI, keyboard shortcuts, Command Palette, future Agent and MCP integrations are designed to converge on the same Action Platform rather than duplicate business logic.

Core user flow:

```text
CONNECT
→ EXPLORE
→ OPEN TABLE / RESOURCE
→ WORK INSIDE TABS
→ QUERY / INSPECT / EDIT DATA
→ SWITCH CONTEXT WITHOUT LOSING STATE
```

---

## 2. Release Scope

### Included in 0.1.0

- PostgreSQL and SQLite connection lifecycle
- Session/startup reconnect source flow
- Schema/table/view Explorer
- Data-first table navigation
- DB Object workbench: Data, Columns, Indexes, Relations, DDL inspection
- Monaco SQL editor
- Current statement / selection / run-all execution
- Query cancellation
- Explain and SQL formatting
- Query history
- Result grid
- Data Grid filtering, sorting, pagination, column resize, selection and copy
- PK-based staged row update/delete
- Patch-style multi-cell row editing with revision-safe apply semantics
- Result export release subset
- Workspace tabs, preview/pin, Quick Open, Command Palette
- Action Platform runtime and confirmation/cancellation lifecycle
- Agent panel as Preview only

### Explicitly deferred from 0.1.0

- MCP server
- Production Agent execution
- Additional DB drivers
- Full schema mutation workbench
- Users/roles workbench
- Complete row-insert workflow
- Advanced automation

---

## 3. Phase Status

| Phase | Focus | Current Status | Release Notes |
|---|---|---|---|
| 0 | Project scaffolding | DONE | Workspace/build structure complete |
| 1 | Core domain + ports | DONE | Stable Rust domain and port layer |
| 2 | Infrastructure adapters | DONE | PostgreSQL, SQLite, metadata, secrets |
| 3 | Application services | DONE | Connection/query/schema/export/data services |
| 4 | Tauri commands | DONE | Command/DTO/error boundary implemented |
| 5 | Frontend scaffolding | DONE | React/Vite/TanStack/shadcn shell |
| 6 | FE core utilities | DONE | API, stores, i18n, Action Platform foundation |
| 7 | Connection module | DONE | Release source scope complete |
| 8 | Query module | DONE | Release source scope complete |
| 9 | Schema module | DONE | Release inspection/workbench scope complete |
| 10 | DataGrid module | DONE / PARTIAL historical | Read/update/delete release scope done; insert deferred |
| 11 | Export module | DONE (release subset) | Manual smoke required |
| 12 | Advanced features | PARTIAL / DEFERRED | Full historical scope intentionally not required for 0.1.0 |
| 13 | Testing | BLOCKED | Current release SHA must be fully green |
| 14 | CI/CD + packaging | BLOCKED | Workflow exists; current cross-platform artifacts not proven |

---

## 4. Milestone Status

| Milestone | Original Gate | Status |
|---|---|---|
| M1 Scaffold complete | Rust + frontend build foundations | DONE |
| M2 Core domain + ports | Domain/traits defined | DONE |
| M3 Infrastructure adapters | DB adapters available | DONE |
| M4 Application services | Service layer available | DONE |
| M5 Tauri commands | Frontend can invoke backend contracts | DONE |
| M6 FE scaffold ready | Desktop shell/workspace loads | DONE |
| M7 Connection module | Connection lifecycle source complete | DONE |
| M8 Query module | SQL editor + execution/result workflow | DONE |
| M9 DataGrid release scope | Browse/filter/sort/update/delete | DONE in source |
| M10 All historical advanced features | Original full feature list | NOT REQUIRED for 0.1.0; split into deferred roadmap |
| M11 Testing complete | Full green suite + coverage target | BLOCKED / PENDING current HEAD evidence |
| M12 CI/CD + packaging | Installers build successfully | BLOCKED / PENDING release matrix |

---

## 5. Current Release Gates

A feature being present in source is not enough to mark release-ready. The release SHA must pass all four layers:

```text
SOURCE COMPLETE
↓
AUTOMATED VERIFIED
↓
TAURI / CROSS-PLATFORM BUILD VERIFIED
↓
MANUAL RUNTIME VERIFIED
```

### Automated gate

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

### Desktop release gate

- macOS Tauri build + artifact
- Windows Tauri build + artifact
- Linux Tauri build + artifact
- packaged/runtime smoke
- release notes/readiness updated to exact SHA

---

## 6. Immediate Priority

Do **not** start Wave C, MCP, or production Agent execution before the following are closed:

1. Current frontend full test suite is green at exact release SHA.
2. `format:check` is compatible with the release workflow.
3. Release Build matrix is green on macOS, Windows, Linux.
4. Manual smoke validates connection, query, cancel, safety, Data Grid staged mutations, restart/session restore.
5. Final release docs point to the exact tag candidate SHA.

---

## 7. Current Risk Register

| Risk | Status | Impact | Mitigation / release action |
|---|---|---|---|
| Carrying old verification results to new HEAD | ACTIVE | High | Verification docs now require exact-SHA evidence |
| Data mutation correctness regressions | REDUCED | High | Patch model + revision IDs implemented; run focused runtime smoke/tests |
| Cross-platform packaging failure | ACTIVE | High | Run release matrix before tag |
| Prettier drift blocking preflight | ACTIVE | Medium/High | Resolve format gate before release workflow sign-off |
| Startup/session reconnect edge cases | ACTIVE | Medium | Manual restart/reconnect smoke |
| SQLite native dialog runtime | ACTIVE | Medium | Verify packaged/dev Tauri Browse flow |
| SSH tunnel portability | ACTIVE / DEFERRED QUALITY | Medium | Do not market as fully verified; E2E later |
| Unsigned installers | ACCEPTED 0.1 | Medium | Document OS warnings; signing post-0.1 unless required |
| Scope creep | CONTROLLED | High | Release freeze: no Wave C/MCP before 0.1.0 |

---

## 8. Release Decision

See [`07-current-status.md`](07-current-status.md) and `docs/release/0.1.0-readiness.md`.

**Current decision: NOT READY TO TAG.**

The intended 0.1.0 feature scope is largely implemented. Remaining work is dominated by verification, packaging evidence, manual runtime QA, and a small known frontend test issue rather than new feature development.
