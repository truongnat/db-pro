# Traceability Matrix — Capabilities → Tests → CI → Smoke → Evidence Owner

> Baseline SHA: `a2cf4a3`
> Issue: #131
> Source: Codebase analysis, CI workflow (`ci.yml`), integration tests, frontend tests, smoke fixtures

## Coverage status legend

| Status | Meaning |
|---|---|
| `PROVEN` | Has unit + integration tests, passes in CI, covered by smoke fixture |
| `TESTED BUT NOT RELEASE-QUALIFIED` | Has tests but not exercised against packaged build |
| `RUNTIME ONLY` | Verified manually or by integration test, no unit test |
| `PARTIAL` | Some aspects tested, gaps remain |
| `MISSING` | No test evidence |
| `DEFERRED/NOT SHIPPED` | Feature not in v0.1 scope |

## Traceability matrix

### Connection lifecycle

| Capability | Source owner | Unit tests | Integration tests | CI job | Smoke row | Blocking issue | Coverage status | Gap |
|---|---|---|---|---|---|---|---|---|
| PG create/connect/disconnect | `infrastructure/postgres/connector.rs` | `domain/connection`, `postgres/connector` | `integration.rs`, `pg_integration.rs` | `Rust checks` | `fixtures/smoke/postgres/` | — | TESTED BUT NOT RELEASE-QUALIFIED | Needs packaged-build verification |
| SQLite create/open/connect | `infrastructure/sqlite/connector.rs` | `domain.connection`, `sqlite/connector` | `integration.rs` | `Rust checks` | `fixtures/smoke/sqlite/` | — | TESTED BUT NOT RELEASE-QUALIFIED | Needs packaged-build verification |
| Startup reconnect / session restoration | `frontend/modules/connection/` | `session-restoration.test.tsx` | — | `Frontend checks` | — | — | TESTED BUT NOT RELEASE-QUALIFIED | No backend session restore test |
| Test connection | `tauri-app/commands/connection.rs` | `commands/connection` | — | `Rust checks` | — | — | PARTIAL | No integration test for test_connection command |
| Connection CRUD | `application/connection_service.rs` | `connection_service` tests | `integration.rs` | `Rust checks` | — | — | TESTED BUT NOT RELEASE-QUALIFIED | — |
| Secret/keyring lifecycle | `infrastructure/secret/keyring_vault.rs` | `domain/secret` | — | `Rust checks` | — | #126 | PARTIAL | No integration test for keyring fallback path |

### Schema introspection

| Capability | Source owner | Unit tests | Integration tests | CI job | Smoke row | Blocking issue | Coverage status | Gap |
|---|---|---|---|---|---|---|---|---|
| Explorer schemas/tables/views | `infrastructure/postgres/introspect.rs`, `sqlite/introspect.rs` | `postgres/introspect`, `sqlite/introspect` | `integration.rs`, `pg_integration.rs` | `Rust checks` | Smoke: 10 tables, 2 views (PG), 10 tables, 2 views (SQLite) | — | PROVEN | — |
| Columns introspection | `introspect.rs` | — | `schema_columns_atomicity_regression.rs` | `Rust checks` | Smoke tables have various column types | — | PROVEN | — |
| Indexes introspection | `introspect.rs` | — | `schema_indexes_runtime_verification.rs` | `Rust checks` | Smoke: 8 indexes (PG), 6 indexes (SQLite) | — | PROVEN | — |
| Relations/FK introspection | `introspect.rs` | — | `schema_relations_runtime_verification.rs` | `Rust checks` | Smoke tables have FK chains | — | PROVEN | — |
| Triggers introspection | `introspect.rs` | — | `schema_triggers_runtime_verification.rs` | `Rust checks` | Smoke: 1 trigger (PG), 1 trigger (SQLite) | — | PROVEN | — |
| DDL introspection (get_table_ddl) | `tauri-app/commands/` | — | `pg_integration.rs` | `Rust checks` | — | — | RUNTIME ONLY | No dedicated unit test |

### Query execution

| Capability | Source owner | Unit tests | Integration tests | CI job | Smoke row | Blocking issue | Coverage status | Gap |
|---|---|---|---|---|---|---|---|---|
| Current statement execution | `application/query_service.rs` | `query_service` tests | `integration.rs` | `Rust checks` | — | — | TESTED BUT NOT RELEASE-QUALIFIED | — |
| Query selection (partial execute) | `application/query_service.rs` | `query_service` tests | — | `Rust checks` | — | — | PARTIAL | Selection extraction not integration-tested |
| Run-all / multi-statement | `application/query_service.rs`, `execute_batch` | `safety.rs` (classifier) | `integration.rs` | `Rust checks` | — | #129 | TESTED BUT NOT RELEASE-QUALIFIED | — |
| Cancellation | `tauri-app/cancel.rs`, `domain/execution.rs` | `domain/execution` | — | `Rust checks` | — | — | PARTIAL | No integration test for cancel mid-query |
| EXPLAIN | `application/query_service.rs` | `safety.rs` (EXPLAIN classify) | `pg_integration.rs` | `Rust checks` | — | — | TESTED BUT NOT RELEASE-QUALIFIED | SQLite EXPLAIN not tested |
| Result metadata/value rendering | `domain/query.rs`, `frontend/modules/query/` | `domain/query` | — | `Frontend checks` | — | — | PARTIAL | Frontend rendering tests exist but not E2E |

### Data Grid

| Capability | Source owner | Unit tests | Integration tests | CI job | Smoke row | Blocking issue | Coverage status | Gap |
|---|---|---|---|---|---|---|---|---|
| Filter/sort/pagination | `frontend/modules/data-grid/` | `data-grid.test.tsx`, `sort.test.ts`, `filter-parser.test.ts`, `tab-grid-state.test.ts` | — | `Frontend checks` | — | — | TESTED BUT NOT RELEASE-QUALIFIED | — |
| Staged update/delete | `frontend/modules/data-grid/state/staged-changes.store.ts` | `staged-changes-store.test.ts`, `staged-changes.test.ts`, `change-bar.test.tsx` | — | `Frontend checks` | — | — | TESTED BUT NOT RELEASE-QUALIFIED | — |
| No-PK readonly | `application/table_data_service.rs`, `domain/capabilities.rs` | `table_data_service` tests | — | `Rust checks` | Smoke: `smoke_audit_log` (no PK) | — | PARTIAL | No explicit no-PK integration test |
| Cell editing | `frontend/modules/data-grid/` | `cell-editor.test.tsx`, `data-grid.service.test.ts` | — | `Frontend checks` | — | — | TESTED BUT NOT RELEASE-QUALIFIED | — |
| Dirty/close guard | `frontend/commons/stores/close-guard.store.ts`, `use-tab-close-guard.ts` | `close-guard.test.ts` | — | `Frontend checks` | — | — | TESTED BUT NOT RELEASE-QUALIFIED | — |

### Persistence

| Capability | Source owner | Unit tests | Integration tests | CI job | Smoke row | Blocking issue | Coverage status | Gap |
|---|---|---|---|---|---|---|---|---|
| Workspace/tab persistence | `frontend/commons/stores/workspace.store.ts` | `workspace.store.test.ts` (780 lines) | — | `Frontend checks` | — | #127 | TESTED BUT NOT RELEASE-QUALIFIED | — |
| Workspace migrations | `frontend/commons/services/workspace-migrations.ts` | `workspace-migrations.test.ts` (239 lines) | — | `Frontend checks` | — | #127 | PROVEN | — |
| Query history persistence | `frontend/commons/stores/query-history.store.ts` | `query-history-store.test.ts` | — | `Frontend checks` | — | — | TESTED BUT NOT RELEASE-QUALIFIED | — |

### Quick Open / Command Palette

| Capability | Source owner | Unit tests | Integration tests | CI job | Smoke row | Blocking issue | Coverage status | Gap |
|---|---|---|---|---|---|---|---|---|
| Quick Open | `frontend/commons/components/quick-open.tsx`, `services/quick-open-*` | `quick-open.test.tsx` (389 lines), `quick-open-index.test.ts`, `quick-open-rank.test.ts`, `quick-open.store.test.ts` | — | `Frontend checks` | — | — | PROVEN | — |
| Command Palette | `frontend/commons/commands/`, `command.store.ts` | `command-palette.test.tsx`, `command.store.test.ts` | — | `Frontend checks` | — | — | TESTED BUT NOT RELEASE-QUALIFIED | — |

### ER Diagram

| Capability | Source owner | Unit tests | Integration tests | CI job | Smoke row | Blocking issue | Coverage status | Gap |
|---|---|---|---|---|---|---|---|---|
| ER small/medium (<200 tables) | `frontend/modules/er-diagram/` | `approximate-layout.test.ts`, `edge-builder.test.ts`, `cytoscape-renderer.test.ts`, `cytoscape-view.test.tsx` | — | `Frontend checks` | — | — | TESTED BUT NOT RELEASE-QUALIFIED | — |
| ER 201/500/1000 search/neighborhood/overview | `frontend/modules/er-diagram/` | `er-large-schema.test.ts` (if exists), benchmark tests | — | `Frontend checks` | `fixtures/smoke/large-er/` (250 tables) | #115, #116 | PARTIAL | No runtime test with 500/1000 tables in CI |
| ER position persistence | `frontend/modules/er-diagram/components/er-diagram.tsx` | — | — | — | — | — | MISSING | localStorage positions not unit-tested |

### Export / Import / Backup

| Capability | Source owner | Unit tests | Integration tests | CI job | Smoke row | Blocking issue | Coverage status | Gap |
|---|---|---|---|---|---|---|---|---|
| CSV export | `application/export_service.rs` | `export_service` tests | — | `Rust checks` | — | #128 | PARTIAL | No integration test writing actual CSV file |
| JSON export | `tauri-app/commands/` | — | — | `Rust checks` | — | — | MISSING | No test evidence |
| Excel export | `application/export_service.rs` | `export_service` tests | — | `Rust checks` | — | #128 | PARTIAL | XLSX output not validated |
| Backup (PG) | `infrastructure/backup/pg_dump.rs` | — | — | — | — | #128 | MISSING | Requires pg_dump binary, not in CI |
| Backup (SQLite) | `infrastructure/backup/sqlite_backup.rs` | — | — | — | — | #128 | MISSING | No test evidence |
| Restore | `infrastructure/backup/` | — | — | — | — | #128 | MISSING | No test evidence, P1 findings in audit |
| Import | — | — | — | — | — | — | DEFERRED/NOT SHIPPED | Not in v0.1 scope |

### Security / Safety

| Capability | Source owner | Unit tests | Integration tests | CI job | Smoke row | Blocking issue | Coverage status | Gap |
|---|---|---|---|---|---|---|---|---|
| Statement classifier | `domain/safety.rs` | 22 tests in `safety.rs` | — | `Rust checks` | — | #129 | PROVEN | — |
| Safety policy validation | `domain/safety.rs` | 6 policy tests | — | `Rust checks` | — | #129 | PROVEN | — |
| Credential handling | `infrastructure/secret/` | `domain/secret` | — | `Rust checks` | — | #122, #126 | PARTIAL | Keyring integration not tested in CI |

### Capabilities / Provider model

| Capability | Source owner | Unit tests | Integration tests | CI job | Smoke row | Blocking issue | Coverage status | Gap |
|---|---|---|---|---|---|---|---|---|
| Provider-value type matrix | `domain/capabilities.rs` | `capabilities` tests | — | `Rust checks` | — | — | TESTED BUT NOT RELEASE-QUALIFIED | — |
| PG-specific capabilities | `postgres/connector.rs`, `introspect.rs` | — | `pg_integration.rs` | `Rust checks` | PG smoke fixture | — | PROVEN | — |
| SQLite-specific capabilities | `sqlite/connector.rs`, `introspect.rs` | — | `integration.rs` | `Rust checks` | SQLite smoke fixture | — | PROVEN | — |

### Packaging / Distribution

| Capability | Source owner | Unit tests | Integration tests | CI job | Smoke row | Blocking issue | Coverage status | Gap |
|---|---|---|---|---|---|---|---|---|
| Packaged launch per platform | `tauri.conf.json`, `release.yml` | — | — | `Release Build` | — | #134 | MISSING | No packaged-build smoke test in CI |
| Install per platform | `tauri.conf.json` bundle targets | — | — | — | — | #134 | MISSING | Manual verification only |
| Public identity/version metadata | `tauri.conf.json` | — | — | — | — | #127 | PARTIAL | No CI check for version consistency |

## Uncovered gaps summary

| Gap | Severity | Recommended action |
|---|---|---|
| No packaged-build smoke test | P1 | Add packaged-launch verification to release workflow |
| Backup/restore has zero test evidence | P1 | Add SQLite backup integration test (no external deps) |
| JSON export has zero test evidence | P2 | Add export integration test |
| ER position persistence untested | P3 | Low priority for v0.1 |
| Cancel mid-query not integration-tested | P2 | Add cancel integration test with timeout |
| No-PK readonly not explicitly tested | P2 | Add no-PK table integration test |
| Keyring fallback path not tested | P2 | Add encrypted-fallback integration test |
| ER 500/1000 table runtime not in CI | P2 | Add benchmark as CI step (optional) |

## CI job reference

| CI Job | Workflow file | What it runs |
|---|---|---|
| `Rust checks` | `.github/workflows/ci.yml` | `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test --all` |
| `Frontend checks` | `.github/workflows/ci.yml` | `tsc --noEmit`, `eslint`, `prettier`, `check:tokens`, `build`, `vitest run` |
| `Release Build` | `.github/workflows/release.yml` | Tauri build matrix (deb, appimage, rpm, dmg, msi, nsis) |
| `VPS PR Review` | `.github/workflows/vps-pr-review.yml` | Read-only VPS code review |

## Test count summary

| Area | Rust tests | Frontend tests | Integration tests |
|---|---|---|---|
| Core domain | ~15 files with tests | — | — |
| Infrastructure | 6 test files | — | 6 integration test files |
| Tauri commands | ~3 files with tests | — | — |
| Frontend commons | — | ~15 test files | — |
| Frontend modules | — | ~50+ test files | — |
| **Total** | **~24 Rust test files** | **~65+ frontend test files** | **6 integration test files** |
