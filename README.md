# DB Pro

DB Pro is a planned Tauri 2 desktop database client for Ubuntu. It combines a Rust backend with a React/TypeScript frontend and targets PostgreSQL first, followed by SQLite.

> Current status: planning/design stage. The repository does not yet contain the Rust workspace, frontend application, tests, or CI pipeline. `demo.html` is currently a UI design prototype.

## Product direction

- DBeaver-like desktop workflow focused on PostgreSQL and SQLite.
- Secure connection management with OS keyring integration.
- SQL editor, query execution, history, cancellation, explain plans, and transactions.
- Schema browsing, table details, DDL inspection, editable data grid, and CSV/JSON/Excel export.
- Tauri 2 shell with Clean Architecture boundaries: domain → ports → infrastructure → application services → Tauri commands.
- React/TypeScript frontend with Monaco, TanStack Query, Zustand, shadcn/ui, Radix UI, Tailwind CSS, i18n, and virtualized data rendering.

## Repository layout

```text
demo.html                 UI design prototype
plans/00-index.md         Plan index and task map
plans/01-overview.md      Product phases, milestones, risks
plans/02-backend-tasks.md Rust backend tasks (B-001…B-105)
plans/03-frontend-tasks.md Frontend tasks (F-001…F-150)
plans/04-testing-tasks.md Testing tasks (T-001…T-078)
plans/05-cicd-tasks.md    CI/CD and packaging tasks (C-001…C-060)
plans/06-database-tasks.md Database, crypto, and meta-store tasks (D-001…D-107)
docs/08-technology-decisions.md Chosen stack, runtime model, security, and query strategy
docs/09-architecture-decisions.md Ratified architecture decisions before implementation
```

## Planned delivery phases

| Phase | Outcome |
|---|---|
| 0 | Rust/Tauri/frontend scaffolding and quality gates |
| 1–2 | Domain model, ports, PostgreSQL/SQLite adapters, vault, meta-store |
| 3–4 | Application services and Tauri command boundary |
| 5–7 | Frontend foundation and connection module |
| 8–11 | SQL/query, schema, data grid, and export modules |
| 12 | SSH tunnel, DDL editing, roles, backup/restore |
| 13–14 | Tests, CI/CD, Linux packaging, and release workflow |

The plan estimates approximately 47 working days. The first useful vertical slice should be smaller: scaffold → save/test PostgreSQL connection → execute a read-only query → render results.

## Proposed first milestone

1. Create the Cargo workspace and minimal Vite/Tauri app.
2. Define shared connection/query/result DTOs and error envelope.
3. Implement PostgreSQL connection testing and one read-only query.
4. Add a real PostgreSQL integration fixture and a basic frontend result table.
5. Add `cargo fmt`, Clippy, TypeScript typecheck, unit tests, and one CI workflow.

## Ratified architecture decisions

The nine pre-implementation decisions are ratified in [`docs/09-architecture-decisions.md`](docs/09-architecture-decisions.md). The technology rationale remains in [`docs/08-technology-decisions.md`](docs/08-technology-decisions.md).

- Add the referenced `docs/` set (`01-features.md` through `07-fe-architecture.md`), or remove those references from the plans.

## Suggested development rules

- Build vertical slices and keep each phase independently runnable.
- Treat `demo.html` as the visual approval baseline; implement the approved design in source-owned shadcn/Radix components.
- Keep database-specific types inside infrastructure adapters; expose stable domain/DTO types to the UI.
- Never log passwords, connection strings, bound parameters, or raw secrets.
- Default to read-only or confirmation-gated destructive operations.
- Test against real PostgreSQL and SQLite fixtures, not only mocks.
- Pin dependency versions and verify Linux packaging in CI.

## License

Not defined yet.
