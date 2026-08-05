# DB Client — Frontend (React/TypeScript) Tasks

---

## Phase 5: Frontend Scaffolding

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-001 | Initialize Vite + React + TypeScript project | `npm create vite@latest frontend -- --template react-ts` | — | 1h |
| F-002 | Install core dependencies | `react`, `react-dom`, `typescript`, `vite`, `@types/react`, `@types/react-dom` | F-001 | 30m |
| F-003 | Install MUI 7 + dependencies | `@mui/material`, `@mui/icons-material`, `@emotion/react`, `@emotion/styled` | F-002 | 30m |
| F-004 | Install TanStack Query 5 | `@tanstack/react-query` | F-002 | 15m |
| F-005 | Install Zustand + persist | `zustand`, `zustand/middleware` | F-002 | 15m |
| F-006 | Install Monaco editor | `@monaco-editor/react` | F-002 | 15m |
| F-007 | Install virtualized grid | `@tanstack/react-virtual` | F-002 | 15m |
| F-008 | Install i18next + react-i18next | `i18next`, `react-i18next`, `i18next-browser-languagedetector` | F-002 | 15m |
| F-009 | Install TanStack Router | `@tanstack/react-router`, `@tanstack/router-generator` | F-002 | 15m |
| F-010 | Install react-hook-form + Zod | `react-hook-form`, `@hookform/resolvers`, `zod` | F-002 | 15m |
| F-011 | Install Tauri 2 dependencies | `@tauri-apps/api`, `@tauri-apps/cli` | F-001 | 30m |
| F-012 | Configure `tsconfig.json` | `strict: true`, `noImplicitAny`, path aliases (`@/` → `src/`) | F-001 | 30m |
| F-013 | Configure ESLint 9 + Prettier | `.eslintrc.cjs`, `.prettierrc`, `eslintignore` | F-001 | 1h |
| F-014 | Configure Vite + Tauri build | `vite.config.ts`, `tauri.conf.json`, build scripts in `package.json` | F-011 | 1h |
| F-015 | Set up folder structure | `src/app/`, `src/commons/`, `src/modules/`, `src/routes/`, `src/locales/` | F-014 | 30m |
| F-016 | Verify `npm run dev` passes | Dev server starts, React app renders | F-015 | 30m |

## Phase 6: FE Core Utilities

### 6.1 DI Container

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-017 | Create `commons/di/container.ts` | `DIContainer` class with `register()` and `resolve()` methods | F-016 | 1h |
| F-018 | Create `commons/di/registry.ts` | `SERVICE_NAMES` constant, `ServiceRegistry` type, `ServiceName` type | F-017 | 1h |

### 6.2 API Wrapper

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-019 | Create `commons/utils/api.ts` | `apiInvoke<T>()` wrapper around `tauri.invoke` with error handling | F-016 | 1h |
| F-020 | Create `commons/utils/server-error-normalize.ts` | `normalizeServerError()` maps any error to `NormalizedError` shape | F-019 | 1h |
| F-021 | Create `commons/utils/server-error-translate.ts` | `translateError()` resolves i18n key from `NormalizedError` | F-020 | 1h |
| F-022 | Create `commons/utils/validation.ts` | Zod schemas for input validation (connection config, query params) | F-016 | 1h |
| F-023 | Create `commons/utils/clipboard.ts` | `copyToClipboard()` with column headers for grid data | F-016 | 30m |
| F-024 | Create `commons/utils/date-formatter.ts` | `formatDate()`, `formatDuration()` using `Intl.DateTimeFormat` | F-016 | 30m |

### 6.3 Error Handling Infrastructure

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-025 | Create `commons/utils/error-types.ts` | `TranslatedError` type, `AppError` class | F-020 | 30m |
| F-026 | Create `commons/components/ErrorBoundary.tsx` | React error boundary component for each module | F-025 | 1h |

### 6.4 State Management

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-027 | Create `commons/stores/connection.store.ts` | Zustand + persist: connections list, active connection, loading, error | F-005 | 2h |
| F-028 | Create `commons/stores/query-history.store.ts` | Zustand: query history entries | F-005 | 1h |
| F-029 | Create `commons/stores/theme.store.ts` | Zustand: light/dark mode, persisted | F-005 | 1h |
| F-030 | Create `commons/stores/settings.store.ts` | Zustand: language, default connection, page size | F-005 | 1h |
| F-031 | Create `commons/stores/screen-context.store.ts` | Zustand: PgId, FormId for audit trail | F-005 | 1h |

### 6.5 i18n Setup

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-032 | Create `commons/locales/en.json` | English translations for all i18n keys | F-008 | 2h |
| F-033 | Create `commons/locales/ja.json` | Japanese translations for all i18n keys | F-032 | 2h |
| F-034 | Create `commons/locales/i18n.ts` | i18next initialization, language detection | F-032 | 1h |
| F-035 | Create `commons/locales/useTranslation.ts` | Custom hook wrapping `useTranslation` from react-i18next | F-034 | 30m |

### 6.6 Providers

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-036 | Create `app/providers/theme.provider.tsx` | MUI theme provider with dark/light mode | F-029 | 1h |
| F-037 | Create `app/providers/query.provider.tsx` | TanStack QueryClient provider | F-004 | 30m |
| F-038 | Create `app/providers/snackbar.provider.tsx` | Snackbar for notifications (success, error, warning) | F-026 | 1h |
| F-039 | Create `app/providers/modal.provider.tsx` | Modal dialog management | F-038 | 1h |
| F-040 | Create `app/App.tsx` | Providers tree: Theme > Query > Snackbar > Modal > Router | F-036-F-039 | 1h |
| F-041 | Create `app/app.module.ts` | `bootstrapServices()` — register all services in DI container | F-018, F-019, F-027-F-031 | 2h |

## Phase 7: Connection Module (CO)

### 7.1 Types

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-042 | Create `modules/connection/types/connection.types.ts` | `ConnectionId`, `ConnectionConfig`, `Connection`, `DriverType`, `SslMode`, `SshTunnelConfig` | F-012 | 1h |

### 7.2 Service

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-043 | Create `modules/connection/services/connection.service.ts` | `IConnectionService` implementation calling `tauri.invoke` for all CRUD + connect/disconnect/test | F-042, F-019 | 3h |
| F-044 | Create `modules/connection/services/connection.agent.ts` | Mock implementation for tests | F-043 | 1h |

### 7.3 Queries (TanStack Query)

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-045 | Create `modules/connection/queries/CO01001.queries.ts` | `useConnectionList()` hook with TanStack Query | F-043 | 1h |
| F-046 | Create `modules/connection/queries/CO03001.queries.ts` | `useCreateConnection()`, `useUpdateConnection()`, `useDeleteConnection()`, `useTestConnection()` | F-043 | 1h |
| F-047 | Create `modules/connection/queries/CO01002.queries.ts` | `useConnect()`, `useDisconnect()` | F-043 | 1h |

### 7.4 Components

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-048 | Create `modules/connection/components/connection-list.tsx` | Table/list of saved connections with status indicators | F-045 | 2h |
| F-049 | Create `modules/connection/components/connection-editor.tsx` | Form: name, host, port, database, user, password, SSL, SSH tunnel | F-046 | 3h |
| F-050 | Create `modules/connection/components/connection-status.tsx` | Visual indicator: connected/disconnected/connecting/error | F-047 | 1h |
| F-051 | Create `modules/connection/components/connection-form/` | Reusable form atoms: `HostInput`, `PortInput`, `SslSelect`, `SshToggle` | F-049 | 2h |

### 7.5 Pages

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-052 | Create `modules/connection/pages/connections-page.tsx` | List page with connection list + create/edit buttons | F-048 | 1h |
| F-053 | Create `modules/connection/pages/connection-edit-page.tsx` | Editor page with save/test/cancel actions | F-049 | 1h |

### 7.6 State

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-054 | Create `modules/connection/state/connection.store.ts` | Local Zustand store for connection module state | F-027 | 1h |

### 7.7 Testing

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-055 | Write unit tests for `connection.service.ts` | Mock `tauri.invoke`, test all service methods | F-043 | 2h |
| F-056 | Write component tests for `connection-list.tsx` | Test rendering, click actions, loading states | F-048 | 1h |
| F-057 | Write component tests for `connection-editor.tsx` | Test form validation, submit, error display | F-049 | 1h |

## Phase 8: Query Module (QU)

### 8.1 Types

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-058 | Create `modules/query/types/query.types.ts` | `ColumnMeta`, `Row`, `QueryResult`, `QueryError`, `QueryHistoryEntry`, `ExplainPlan` | F-012 | 1h |

### 8.2 Service

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-059 | Create `modules/query/services/query.service.ts` | `IQueryService` implementation: `execute()`, `getHistory()`, `saveToHistory()` | F-058, F-019 | 2h |
| F-060 | Create `modules/query/services/query.agent.ts` | Mock implementation for tests | F-059 | 1h |

### 8.3 Queries

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-061 | Create `modules/query/queries/QU01001.queries.ts` | `useExecuteQuery()` mutation, `useQueryResult()` query | F-059 | 1h |
| F-062 | Create `modules/query/queries/QU01002.queries.ts` | `useQueryHistory()` query | F-059 | 1h |
| F-063 | Create `modules/query/queries/QU03001.queries.ts` | `useExplainPlan()` mutation | F-059 | 1h |

### 8.4 Components

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-064 | Create `modules/query/components/query-editor.tsx` | Monaco editor with SQL syntax highlighting, line numbers | F-006, F-058 | 4h |
| F-065 | Create `modules/query/components/result-grid.tsx` | Virtualized grid with sortable columns, column resize, hide/show, reorder | F-007, F-058 | 6h |
| F-066 | Create `modules/query/components/explain-plan.tsx` | Tree view for EXPLAIN ANALYZE cost tree | F-061 | 2h |
| F-067 | Create `modules/query/components/transaction-bar.tsx` | BEGIN / COMMIT / ROLLBACK buttons, auto-commit toggle indicator | F-059 | 1h |
| F-068 | Create `modules/query/components/query-toolbar.tsx` | Execute selected, execute all, clear, format SQL buttons | F-064 | 1h |
| F-069 | Create `modules/query/components/query-history-panel.tsx` | Searchable, clickable query history list | F-062 | 2h |
| F-070 | Create `modules/query/components/parameter-input-dialog.tsx` | Dialog for binding `$1`, `$2` parameters before execution | F-064 | 1h |

### 8.5 Pages

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-071 | Create `modules/query/pages/query-page.tsx` | Layout with editor (top) + result grid (bottom) + transaction bar | F-064-F-068 | 1h |

### 8.6 State

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-072 | Create `modules/query/state/query.store.ts` | Local Zustand store for query module state | F-028 | 1h |

### 8.7 Testing

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-073 | Write unit tests for `query.service.ts` | Mock `tauri.invoke`, test execute, history, error paths | F-059 | 2h |
| F-074 | Write component tests for `query-editor.tsx` | Test Monaco rendering, value changes | F-064 | 1h |
| F-075 | Write component tests for `result-grid.tsx` | Test virtualized rendering, sort, filter, pagination | F-065 | 2h |

## Phase 9: Schema Module (SC)

### 9.1 Types

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-076 | Create `modules/schema/types/schema.types.ts` | `Schema`, `Table`, `Column`, `Index`, `ForeignKey`, `View` | F-012 | 1h |

### 9.2 Service

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-077 | Create `modules/schema/services/schema.service.ts` | `ISchemaService` implementation: `introspect()`, `getTableDdl()` | F-076, F-019 | 1h |
| F-078 | Create `modules/schema/services/schema.agent.ts` | Mock implementation for tests | F-077 | 30m |

### 9.3 Queries

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-079 | Create `modules/schema/queries/SC01001.queries.ts` | `useIntrospect()`, `useTableDdl()` | F-077 | 1h |

### 9.4 Components

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-080 | Create `modules/schema/components/schema-tree.tsx` | Tree view: schemas → tables → columns, expandable/collapsible | F-079 | 3h |
| F-081 | Create `modules/schema/components/table-detail.tsx` | Click table → columns, indexes, foreign keys, triggers | F-079 | 2h |
| F-082 | Create `modules/schema/components/ddl-viewer.tsx` | Syntax-highlighted DDL display with copy button | F-079 | 1h |
| F-083 | Create `modules/schema/components/erd-diagram.tsx` | Visual ERD diagram using foreign key relationships | F-080 | 3h |

### 9.5 Pages

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-084 | Create `modules/schema/pages/schema-page.tsx` | Layout with schema tree (left) + table detail (right) | F-080-F-083 | 1h |

### 9.6 Testing

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-085 | Write unit tests for `schema.service.ts` | Mock `tauri.invoke`, test introspect, DDL | F-077 | 1h |
| F-086 | Write component tests for `schema-tree.tsx` | Test tree rendering, expand/collapse, selection | F-080 | 1h |

## Phase 10: DataGrid Module (DG)

### 10.1 Types

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-087 | Create `modules/data-grid/types/data-grid.types.ts` | `GridColumn`, `GridRow`, `GridFilter`, `GridSort`, `GridState` | F-012 | 1h |

### 10.2 Service

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-088 | Create `modules/data-grid/services/data-grid.service.ts` | `IDataGridService` implementation: CRUD operations via `tauri.invoke` | F-087, F-019 | 2h |
| F-089 | Create `modules/data-grid/services/data-grid.agent.ts` | Mock implementation for tests | F-088 | 30m |

### 10.3 Queries

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-090 | Create `modules/data-grid/queries/DG01001.queries.ts` | `useTableData()` query with pagination, sorting, filtering | F-088 | 1h |
| F-091 | Create `modules/data-grid/queries/DG01002.queries.ts` | `useInlineEdit()`, `useAddRow()`, `useDeleteRow()` mutations | F-088 | 1h |

### 10.4 Components

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-092 | Create `modules/data-grid/components/data-grid.tsx` | Main grid component: virtualized, sortable, filterable, resizable, reorderable | F-065, F-087 | 6h |
| F-093 | Create `modules/data-grid/components/cell-editor.tsx` | Inline cell editor with validation, error highlight | F-092 | 3h |
| F-094 | Create `modules/data-grid/components/row-actions.tsx` | Edit/delete row actions with confirmation dialogs | F-092 | 2h |
| F-095 | Create `modules/data-grid/components/filter-bar.tsx` | Column-level filter inputs (text, number, date) | F-092 | 2h |
| F-096 | Create `modules/data-grid/components/pagination.tsx` | Page navigation with page size selector (25/50/100/200) | F-092 | 1h |
| F-097 | Create `modules/data-grid/components/column-header.tsx` | Sort indicator, resize handle, visibility toggle | F-092 | 2h |
| F-098 | Create `modules/data-grid/components/empty-state.tsx` | Empty state illustration when no data | F-092 | 30m |
| F-099 | Create `modules/data-grid/components/error-cell.tsx` | Cell highlight red with error tooltip on edit/delete failure | F-093 | 1h |
| F-100 | Create `modules/data-grid/components/loading-overlay.tsx` | Spinner overlay when loading data | F-092 | 30m |

### 10.5 Pages

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-101 | Create `modules/data-grid/pages/data-page.tsx` | Full data page with grid, filter bar, pagination, toolbar | F-092-F-100 | 1h |

### 10.6 State

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-102 | Create `modules/data-grid/state/data-grid.store.ts` | Local Zustand store for grid state (filters, sort, page, columns) | F-027 | 1h |

### 10.7 Testing

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-103 | Write unit tests for `data-grid.service.ts` | Mock `tauri.invoke`, test CRUD operations | F-088 | 2h |
| F-104 | Write component tests for `data-grid.tsx` | Test rendering, sort, filter, pagination, inline edit | F-092 | 3h |
| F-105 | Write component tests for `cell-editor.tsx` | Test edit flow, validation, error display | F-093 | 2h |

## Phase 11: Export Module (EX)

### 11.1 Types

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-106 | Create `modules/export/types/export.types.ts` | `ExportFormat`, `ExportResult`, `ExportConfig` | F-012 | 30m |

### 11.2 Service

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-107 | Create `modules/export/services/export.service.ts` | `IExportService` implementation: `exportCsv()`, `exportJson()`, `exportExcel()` | F-106, F-019 | 2h |
| F-108 | Create `modules/export/services/export.agent.ts` | Mock implementation for tests | F-107 | 30m |

### 11.3 Queries

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-109 | Create `modules/export/queries/EX01001.queries.ts` | `useExport()` mutation | F-107 | 1h |

### 11.4 Components

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-110 | Create `modules/export/components/export-dialog.tsx` | Dialog: select format, choose SQL, configure options | F-109 | 2h |
| F-111 | Create `modules/export/components/export-progress.tsx` | Progress indicator for streaming export | F-110 | 1h |

### 11.5 Pages

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-112 | Create `modules/export/pages/export-page.tsx` | Export page with dialog and progress | F-110-F-111 | 1h |

### 11.6 Testing

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-113 | Write unit tests for `export.service.ts` | Mock `tauri.invoke`, test CSV/JSON export | F-107 | 1h |

## Phase 12: Advanced Features

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-114 | SSH tunnel config in connection editor | Add SSH tunnel section to CO03001 form | F-049 | 2h |
| F-115 | DDL editor (create/edit/delete table) | Form-based DDL editor with column definition, PK, FK, indexes | F-081 | 4h |
| F-116 | SQL formatter | Auto-format SQL using `sql-formatter` or custom formatter | F-064 | 2h |
| F-117 | SQL templates | Code snippets for SELECT, INSERT, UPDATE, DELETE, CREATE TABLE | F-064 | 1h |
| F-118 | Query result copy with headers | Copy grid data including column headers to clipboard | F-065 | 1h |
| F-119 | Query result export to SQL | Export selected rows as INSERT/UPDATE/DELETE statements | F-065 | 2h |
| F-120 | Query cancel/kill | Cancel running query, kill active session | F-064 | 2h |
| F-121 | Multiple query tabs | Each tab has own connection context | F-064 | 3h |
| F-122 | Auto-save in SQL editor | Auto-save scripts to localStorage | F-064 | 1h |
| F-123 | SQL script organization into folders | Folder tree for saved SQL scripts | F-121 | 2h |
| F-124 | Import/export SQL scripts | Import/export `.sql` files | F-122 | 1h |
| F-125 | Local history | Save local history of every query edit | F-122 | 2h |
| F-126 | Run configurations | Save predefined script configurations | F-123 | 2h |
| F-127 | Multiple result sets | Display multiple result sets in separate tabs | F-065 | 2h |
| F-128 | Result set metadata view | Show column type, length, nullable on result set header | F-065 | 1h |
| F-129 | Result set zoom | Zoom in/out on result grid | F-065 | 1h |
| F-130 | User/role management | UM01001-UM01006: user list, create, edit, delete, role list, grant/revoke | F-043 | 5h |
| F-131 | Backup/restore | BK01001-BK01005: backup, restore, schema export, data migration, scheduled tasks | F-043 | 5h |
| F-132 | Connection color coding, tags, groups | CO02001-CO02006 | F-048 | 3h |
| F-133 | WHERE clause filter builder | DG03001 visual filter builder | F-095 | 3h |
| F-134 | Query result charting | DG03002 bar/line/pie charts from grid data | F-065 | 2h |
| F-135 | Copy as SQL INSERT/UPDATE/DELETE | DG03003 | F-119 | 1h |
| F-136 | Data diff | DG03004 compare two result sets | F-133 | 2h |
| F-137 | JSON/Array column support | DG03006 formatted/collapsible rendering | F-065 | 1h |
| F-138 | Result set column freeze | DG03008 | F-092 | 1h |
| F-139 | Generate CRUD SQL from grid | DG03012 | F-119 | 1h |
| F-140 | Database object search | SC02001 quick search (Ctrl+Shift+O) | F-080 | 2h |
| F-141 | Index management | SC02002 create/edit/delete indexes | F-081 | 2h |
| F-142 | Trigger management | SC02003 create/edit/delete triggers | F-081 | 2h |
| F-143 | Schema diff | SC02005 compare schemas, generate sync SQL | F-081 | 3h |
| F-144 | Data diff | SC02006 compare table data, generate sync SQL | F-136 | 2h |
| F-145 | Database object dependency viewer | SC02007 | F-080 | 2h |
| F-146 | Database object rename/refactoring | SC02009 rename table/column, propagate | F-140 | 3h |
| F-147 | Table partition management | SC02012 | F-081 | 2h |
| F-148 | Tablespace management | SC02013 | F-081 | 1h |
| F-149 | Package as .deb | PK01001: tauri-bundler .deb for Ubuntu 22.04+ | F-014 | 2h |
| F-150 | Package as AppImage | PK01002: tauri-bundler AppImage | F-014 | 1h |

## Phase 13: Frontend Testing

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-151 | Set up vitest + @testing-library/react | Install, configure `vitest.config.ts`, create test setup file | F-016 | 1h |
| F-152 | Set up playwright for E2E | Install, configure `playwright.config.ts`, create test fixtures | F-016 | 1h |
| F-153 | Write service unit tests for all 4 modules | connection, query, schema, export services | F-055, F-073, F-085, F-113 | 4h |
| F-154 | Write component tests for all modules | Connection list/editor, query editor/result grid, schema tree, data grid | F-056-F-057, F-074-F-075, F-086, F-104-F-105 | 6h |
| F-155 | Write E2E tests for critical flows | Connection CRUD, query execution, grid CRUD, export | F-152 | 4h |
| F-156 | Achieve ≥ 70% coverage on `modules/` | Run `vitest run --coverage`, fix gaps | F-153-F-155 | 3h |

## Phase 14: CI/CD + Packaging

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| F-157 | Create GitHub Actions workflow | Lint, format, typecheck, test, coverage, build for Rust + TS | F-156 | 2h |
| F-158 | Set up pre-commit hooks | `rustfmt`, `prettier`, `cargo clippy`, `eslint`, `tsc --noEmit` | F-157 | 1h |
| F-159 | Create `.deb` build script | `tauri build` for .deb packaging | F-149 | 1h |
| F-160 | Create `.AppImage` build script | `tauri build` for AppImage packaging | F-150 | 1h |
| F-161 | Create `CHANGELOG.md` template | `[FEATURE]` / `[FIX]` / `[CHORE]` / `[BREAKING]` format | F-157 | 30m |
| F-162 | Create `README.md` | Project overview, setup instructions, architecture overview | F-157 | 1h |
