# 06 — DB Client — Code Quality Rules

---

## 1. Rust Rules

### 1.1 Error Handling

| Rule | Level | Detail |
|---|---|---|
| No `unwrap()` / `expect()` in production code | deny | Use `?` operator or `Result::map_err` for all fallible operations |
| Use `thiserror` for domain errors | enforce | All error types must derive `thiserror::Error` with `#[derive(thiserror::Error, Debug)]` |
| No raw `String` errors | deny | Every error variant must carry contextual data for debugging |
| `Display` for user-facing messages, `Debug` for logs | enforce | `#[error("user message")]` for Display, `#[error("internal: {0:?}")]` for Debug |
| All `Result` types annotated with explicit error type | enforce | `Result<T, DbError>` not `Result<T, _>` |

### 1.2 Type Safety

| Rule | Level | Detail |
|---|---|---|
| No `String` for IDs | deny | Use `uuid::Uuid` wrapped in newtype: `ConnectionId(pub Uuid)` |
| No raw `&str` for configuration | deny | Use typed structs: `ConnectionConfig`, `QueryParams` |
| `Option` for nullable fields | enforce | Never use sentinel values (empty string, -1, etc.) for null |
| `enum` for closed sets | enforce | `DriverType { Postgres, SQLite }` not `String` |
| `#[derive(Clone, Copy, PartialEq, Eq, Hash)]` where possible | enforce | Value types should be cheap to copy and comparable |
| `Serialize`/`Deserialize` for boundary types | enforce | All types crossing Tauri boundary must derive serde |

### 1.3 Async Patterns

| Rule | Level | Detail |
|---|---|---|
| `#[async_trait::async_trait]` only for trait methods | enforce | Not needed for inherent `impl` async methods |
| `Send + Sync` on all trait objects | enforce | `Box<dyn DbConnector + Send + Sync>` |
| No `.await` in `Drop` impls | deny | Use explicit `close()` / `disconnect()` methods |
| `tokio::spawn` only for truly concurrent work | enforce | Prefer `futures::join!` or `tokio::task::spawn` for parallelism |

### 1.4 Module Organization

| Rule | Level | Detail |
|---|---|---|
| One module per file | enforce | `connection.rs` contains `Connection` module only |
| `mod.rs` only for re-exports | enforce | Prefer `mod connection;` in parent, not `mod.rs` |
| `pub(crate)` for internal visibility | enforce | Never use `pub` unless crossing module boundary intentionally |
| `pub use` for re-exporting | enforce | Re-export at module root, not deep in tree |

### 1.5 Documentation

| Rule | Level | Detail |
|---|---|---|
| All public items documented | deny in CI | `///` doc comment on every `pub` item |
| Doc comments explain WHY, not WHAT | enforce | Code shows WHAT; docs explain intent and invariants |
| Example usage in public API docs | enforce | `/// # Example\n/// ```rust\n/// let conn = Connection::new(config);\n/// ``` |

### 1.6 Testing

| Rule | Level | Detail |
|---|---|---|
| Unit tests for all public functions | deny | `#[cfg(test)] mod tests` in same file |
| Integration tests for trait implementations | deny | `tests/` directory for cross-module tests |
| Mock all external dependencies in unit tests | enforce | Use `mockall` or manual mocks for `DbConnector`, `MetaStore`, etc. |
| Test error paths, not just happy path | enforce | Every `Result`-returning function must have error case tests |
| Test coverage ≥ 80% on `core` | deny in CI | `cargo tarpaulin` |

### 1.7 Performance

| Rule | Level | Detail |
|---|---|---|
| No unnecessary clones | enforce | Use references (`&`) unless ownership is required |
| Streaming for large result sets | enforce | `>10k rows` must use event streaming, not `collect()` into `Vec` |
| Connection pooling for repeated queries | enforce | Use `bb8` or `mobc` for connection pool, not per-query connect |

### 1.8 Safety

| Rule | Level | Detail |
|---|---|---|
| No `unsafe` blocks | deny | Rust's safety guarantees must be preserved |
| No `panic!` in library code | deny | Use `Result` for all recoverable errors |
| No `println!` in production code | deny | Use `tracing` for all logging |

---

## 2. TypeScript Rules

### 2.1 Type Safety

| Rule | Level | Detail |
|---|---|---|
| No `any` type | error | `@typescript-eslint/no-explicit-any` |
| No implicit `any` in function params | error | `noImplicitAny: true` |
| Strict null checks | error | `strict: true` in `tsconfig` |
| No `as` type assertions without comment | warn | Must explain why assertion is safe |
| All function params typed explicitly | error | No inferred param types in public functions |
| Return type on all public functions | error | `function execute(): Promise<QueryResultDto>` not `function execute() {` |

### 2.2 Component Patterns

| Rule | Level | Detail |
|---|---|---|
| Functional components only | enforce | No class components |
| Hooks only at top level | error | No hooks inside loops, conditions, or nested functions |
| Custom hooks for reusable logic | enforce | `useConnection`, `useQueryResult`, etc. |
| No inline object literals in dependencies | warn | Extract to `useMemo` / `useCallback` |
| Components must be pure (no side effects in render) | error | Side effects in `useEffect` only |

### 2.3 State Management

| Rule | Level | Detail |
|---|---|---|
| Zustand only in `commons/stores` or `modules/*/state` | enforce | No ad-hoc state stores |
| TanStack Query for server state | enforce | All async data fetching through `useQuery` / `useMutation` |
| No global mutable state outside Zustand | deny | All shared state must be in a store |
| Screen context (`PgId`, `FormId`) attached to Tauri invoke headers | enforce | Consistent audit trail |

### 2.4 Error Handling

| Rule | Level | Detail |
|---|---|---|
| All `tauri.invoke` calls handle error | enforce | Never ignore `Result` from Tauri commands |
| Error boundary component for each module | enforce | `ErrorBoundary` wraps each feature module |
| User-facing errors use i18n keys | enforce | Never show raw error messages to users |
| Log errors with `console.error` + context | enforce | Include timestamp, action, connection_id |

### 2.5 Testing

| Rule | Level | Detail |
|---|---|---|
| Unit tests for all services | deny | `*.service.test.ts` alongside service |
| Unit tests for all hooks | deny | `*.hook.test.ts` alongside hook |
| Integration tests for Tauri command boundary | deny | Test command → service → port chain |
| E2E tests for critical user flows | deny | `playwright` for CRUD flows |
| Test coverage ≥ 70% on `modules/` | deny in CI | `vitest --coverage` |

### 2.6 File Organization

| Rule | Level | Detail |
|---|---|---|
| One component per file | enforce | `connection-list.tsx` exports `ConnectionList` only |
| One hook per file | enforce | `use-connection.ts` exports `useConnection` only |
| One service per file | enforce | `connection.service.ts` exports `ConnectionService` only |
| Test files alongside source files | enforce | `connection.service.ts` + `connection.service.test.ts` |

---

## 3. Naming Conventions

### 3.1 Rust

| Element | Convention | Example |
|---|---|---|
| Struct / Enum / Type alias | PascalCase | `ConnectionConfig`, `DriverType`, `QueryResult` |
| Trait | PascalCase | `DbConnector`, `SecretStore`, `MetaStore` |
| Method / Function | snake_case | `connect()`, `execute_query()` |
| Variable | snake_case | `connection_id`, `query_result` |
| Constant | SCREAMING_SNAKE_CASE | `MAX_RETRY_COUNT`, `DEFAULT_PAGE_SIZE` |
| File | snake_case | `db_connector.rs`, `query_service.rs` |
| Module | snake_case | `mod connection { }` in `connection.rs` |
| Enum variant | PascalCase | `Postgres`, `SQLite`, `ConnectionLost` |
| Error variant | PascalCase | `ConnectionNotFound`, `PermissionDenied` |
| Lifetime | single letter | `'a`, `'b` |
| Generic type parameter | single uppercase letter | `T`, `E`, `C` where `C: DbConnector` |

### 3.2 TypeScript

| Element | Convention | Example |
|---|---|---|
| Component | PascalCase | `ConnectionList`, `QueryEditor` |
| Interface / Type | PascalCase | `IConnectionService`, `QueryResultDto` |
| Hook | camelCase with `use` prefix | `useConnection`, `useQueryResult` |
| Service class | PascalCase + `Service` | `ConnectionService`, `QueryService` |
| Variable / function | camelCase | `connectionId`, `executeQuery()` |
| File (component) | kebab-case | `connection-list.tsx` |
| File (service) | kebab-case + `.service.ts` | `connection.service.ts` |
| File (hook) | kebab-case + `.hook.ts` | `use-connection.hook.ts` |
| File (queries) | `[ScreenID].queries.ts` | `CO01001.queries.ts` |
| File (store) | kebab-case + `.store.ts` | `connection.store.ts` |
| File (test) | same as source + `.test.ts` | `connection.service.test.ts` |
| DB column | snake_case | `connection_id`, `created_at` |
| DB PK | `[entity]_cd` or `id` | `connection_cd`, `id` |
| i18n key | dot-separated lowercase | `db.connection.save.success` |

### 3.3 Feature Codes

| Element | Convention | Example |
|---|---|---|
| Feature code (2 chars) | `CO`, `QU`, `SC`, `DG`, `EX`, `TR`, `TU`, `UM`, `BK`, `PK` | — |
| Screen-ID (7 chars) | `[FeatureCode][5 digits]` | `CO01001`, `QU01001`, `DG01001` |
| Feature code prefix meaning | `CO`=Connection, `QU`=Query, `SC`=Schema, `DG`=DataGrid, `EX`=Export, `TR`=Transaction, `TU`=Tunnel, `UM`=UserMgmt, `BK`=Backup, `PK`=Packaging | — |

---

## 4. Architecture Rules

### 4.1 Rust Core Dependency Graph

```
tauri-app (commands)  →  core::application  ✅
core::application        →  core::domain        ✅
core::application        →  core::ports         ✅ (via trait objects)
core::domain           →  *                     ❌ (zero dependency)
infrastructure         →  core::domain        ✅ (implements ports)
infrastructure         →  core::ports         ✅ (implements traits)
infrastructure         →  core::application   ❌
tauri-app              →  infrastructure      ❌
tauri-app              →  core::domain        ❌
```

### 4.2 FE Dependency Graph

```
app/          →  modules/  ✅ (composition root)
commons/      →  modules/  ✅ (cross-cutting)
routes/       →  modules/  ✅ (import page components only)
modules/      →  commons/  ✅ (shared utilities, stores)
modules/      →  modules/[other]  ❌ (move shared to commons/)
```

### 4.3 Boundary Rules

- Domain types never cross the Tauri command boundary — use DTOs at boundary
- Ports (traits) live in `core::ports`, implementations in `infrastructure`
- Services receive ports via constructor injection — never resolve globally
- All `pub` in `core` must be documented with `///`
- All `pub` in `frontend` must have TypeScript types (no implicit `any`)

### 4.4 Feature Module Rules (FE)

Each feature module (`modules/connection/`, `modules/query/`, etc.) must contain:
- `components/` — React components for this feature
- `pages/` — Page-level components (route targets)
- `queries/` — `[ScreenID].queries.ts` for TanStack Query keys and functions
- `services/` — Service layer (calls `tauri.invoke`)
- `services/*.agent.ts` — Mock implementation for tests
- `state/` — Zustand store for this feature (if needed)

### 4.5 Shared Code Rules

Shared code that doesn't belong to a single feature goes to `commons/`:
- `di/` — DI container and service registry
- `stores/` — Global Zustand stores
- `components/` — Atomic Design components (atoms, molecules, organisms, templates)
- `utils/` — Pure utility functions
- `locales/` — i18n translations

---

## 5. Quality Gates (CI)

### 5.1 Rust

| Gate | Tool | Fail condition |
|---|---|---|
| Lint | `cargo clippy --all-targets --all-features -- -D warnings` | Any warning |
| Format | `rustfmt --check` | Any diff |
| Test | `cargo test --all` | Any failure |
| Coverage | `cargo tarpaulin` | < 80% on `core` |
| Dep audit | `cargo deny check all` | Any violation |
| Dep cycle | `madge --cycles` | Any cycle |
| Doc check | `cargo doc --no-deps` | Any missing doc on `pub` item |
| Unsafe audit | `cargo audit` | Any `unsafe` block |

### 5.2 TypeScript

| Gate | Tool | Fail condition |
|---|---|---|
| Lint | `eslint . --ext .ts,.tsx` | Any error |
| Format | `prettier --check .` | Any diff |
| Typecheck | `tsc --noEmit` | Any error |
| Test | `vitest run` | Any failure |
| Coverage | `vitest run --coverage` | < 70% on `modules/` |
| E2E | `playwright test` | Any failure |

### 5.3 Pre-commit Hooks

| Hook | Tool | Action |
|---|---|---|
| Rust fmt | `rustfmt` | Auto-format `.rs` files |
| TS fmt | `prettier` | Auto-format `.ts`, `.tsx` files |
| Rust lint | `cargo clippy` | Block commit on warnings |
| TS lint | `eslint` | Block commit on errors |
| TS typecheck | `tsc --noEmit` | Block commit on type errors |

### 5.4 PR Review Checklist

- [ ] All new `pub` items have doc comments
- [ ] No `unwrap()` / `expect()` in new code
- [ ] No `any` in new TypeScript code
- [ ] All error paths tested
- [ ] No circular dependencies introduced
- [ ] Feature module boundaries respected (no cross-module imports)
- [ ] Domain types don't cross Tauri boundary
- [ ] i18n keys used for all user-facing strings
- [ ] Loading and error states handled in UI
- [ ] Tests pass on CI

---

## 6. Code Review Rules

### 6.1 Must Have
- Every PR must have at least one approving review
- All CI gates must pass before merge
- PR description must explain what changed and why

### 6.2 Must Not
- No merging your own PR
- No force-pushing to shared branches after review
- No committing secrets or credentials
- No `TODO` without issue tracker reference

### 6.3 Must Fix
- Any CI failure before merge
- Any review comment marked "must fix"
- Any new `unwrap()` / `expect()` introduced
- Any new `any` type introduced

---

## 7. Feature Flags & Toggles

| Rule | Detail |
|---|---|
| All new features behind feature flags | Use config-based toggle |
| Flag name follows `dbclient.<feature>` convention | e.g., `dbclient.ssh_tunnel`, `dbclient.erd_diagram` |
| Flags have expiry date | Every flag must have a removal date in the ticket |
| No stale flags in production | Remove flag immediately after feature is stable |
| Flags evaluated at startup, not per-request | Avoid runtime flag checks in hot paths |

---

## 8. Versioning & Changelog

### 8.1 Semantic Versioning

| Version bump | When |
|---|---|
| `PATCH` (1.0.X) | Bug fixes, no API breaking changes |
| `MINOR` (1.X.0) | New features, backward compatible |
| `MAJOR` (X.0.0) | Breaking changes (DB schema, API, UI) |

### 8.2 Changelog Rules

- Every PR must have a changelog entry if it changes user-facing behavior
- Changelog format: `[FEATURE]` / `[FIX]` / `[CHORE]` / `[BREAKING]` prefix
- Changelog is in `CHANGELOG.md` at project root
- Version bump happens at release, not at merge

---

## 9. Branching Strategy

| Branch | Purpose | Protection |
|---|---|---|
| `main` | Production-ready code | Protected, no direct push |
| `develop` | Integration branch | Protected, PR only |
| `feature/<code>-<short-name>` | Feature work | PR to `develop` |
| `fix/<issue-id>-<short-name>` | Bug fix | PR to `develop` |
| `release/<version>` | Release preparation | Protected, PR to `main` |
| `hotfix/<version>-<short-name>` | Emergency fix | PR to `main` |

### 9.1 PR Rules

- PR must target `develop`, not `main`
- PR must have at least 1 approving review
- PR must pass all CI gates
- PR must not merge if conflicts exist
- PR title follows `[CO01001] Short description` convention
- PR description must include: what changed, why, how to test, screenshots (if UI)

---

## 10. Release Process

| Step | Action |
|---|---|
| 1. Create release branch | `release/1.0.0` from `develop` |
| 2. Run full CI | All gates must pass |
| 3. Update version | `Cargo.toml`, `package.json`, `CHANGELOG.md` |
| 4. Tag commit | `git tag v1.0.0` |
| 5. Build artifacts | `.deb`, `.AppImage` |
| 6. Sign artifacts | GPG sign `.deb` (Phase 3+) |
| 7. Publish changelog | Update GitHub release notes |
| 8. Merge to `main` | After release branch is stable |
| 9. Merge to `develop` | Ensure `develop` has release changes |

---

## 11. Dependency Management

| Rule | Detail |
|---|---|
| Pin all dependency versions | No `^` or `~` in `Cargo.toml` or `package.json` |
| Audit dependencies regularly | `cargo audit`, `npm audit` weekly |
| No unused dependencies | Remove any crate/npm package not imported |
| Prefer MIT/Apache-2.0 license | No GPL dependencies |
| Update dependencies monthly | `cargo update`, `npm update` with PR |
| Lock files committed | `Cargo.lock`, `package-lock.json` always in VCS |

---

## 12. Security Rules

### 12.1 Credential Management

| Rule | Detail |
|---|---|
| No hardcoded credentials | Ever. In code, config, or environment variables |
| Passwords encrypted at rest | AES-256-GCM with key from OS keyring |
| Passwords never logged | `tracing` must redact sensitive fields |
| Connection strings never stored in plaintext | Always encrypt password field |

### 12.2 SQL Security

| Rule | Detail |
|---|---|
| No string interpolation in SQL | Always parameterized queries |
| No dynamic table/column names from user input | Whitelist allowed identifiers |
| Query timeout enforced | Default 30s, max 300s |
| Result set size limited | Default 100k rows, configurable |
| Transaction timeout enforced | Default 60s |

### 12.3 WebView Security

| Rule | Detail |
|---|---|
| No remote URL loading | Only local files served by Tauri |
| No `eval()` or `Function()` | Blocked by Tauri sandbox |
| No Node.js exposure | Tauri 2 default, verify in config |
| CSP headers set | `default-src 'self'` only |
| No external script injection | All JS bundled in build |

### 12.4 Audit Trail

| Rule | Detail |
|---|---|
| Every action logged | Connect, query, export, edit, delete |
| Log format structured | `timestamp + action_type + connection_id + success` |
| Logs stored locally | SQLite meta-store, rotated weekly |
| No PII in logs | No passwords, no full query text with user data |

---

## 13. Performance Budgets

| Metric | Budget |
|---|---|
| App startup time | < 3 seconds cold start |
| Query execution (small result) | < 500ms for < 1k rows |
| Query execution (large result) | < 5s first batch, streaming thereafter |
| Memory usage (Rust core) | < 200 MB baseline |
| Memory usage (WebView) | < 300 MB baseline |
| App bundle size (Linux) | < 50 MB compressed |
| Grid render (10k rows) | < 100ms initial render |
| Grid render (100k rows) | < 200ms initial render (virtualized) |
| SQL autocomplete latency | < 100ms after typing |
| Schema tree load | < 1s for schemas with < 500 tables |

---

## 14. Accessibility (a11y)

| Rule | Detail |
|---|---|
| Keyboard navigation | All features navigable via keyboard |
| Focus indicators visible | `:focus-visible` on all interactive elements |
| Screen reader support | ARIA labels on all interactive elements |
| Color contrast ≥ 4.5:1 | WCAG AA compliance |
| No color-only indicators | Errors also shown as text/icons |
| Alt text for icons | Every icon has `aria-label` or `alt` |
| Heading hierarchy | Logical `h1` → `h2` → `h3` structure |
| Form labels | Every form input has associated `<label>` |
| Error messages | Associated with form fields via `aria-describedby` |
| Reduced motion | Respect `prefers-reduced-motion` media query |

---

## 15. Internationalization (i18n)

| Rule | Detail |
|---|---|
| All user-facing strings externalized | No hardcoded text in components |
| i18n keys follow `db.<feature>.<action>` convention | e.g., `db.connection.save.success` |
| Support ja and en locales | Minimum for MVP |
| Pluralization handled correctly | Use i18next pluralization, not string concatenation |
| Date/time formatted per locale | `Intl.DateTimeFormat` with locale |
| Number formatting per locale | `Intl.NumberFormat` with locale |
| Translation keys reviewed | No missing keys in any locale |

---

## 16. Documentation Standards

### 16.1 Code Documentation

| Rule | Detail |
|---|---|
| All `pub` items have doc comments | `///` in Rust, JSDoc in TypeScript |
| Doc comments explain intent and invariants | Not just what the code does |
| Example usage in public API docs | `/// # Example` block |
| Module-level docs explain purpose | `//!` at top of each module file |

### 16.2 Architecture Documentation

| Rule | Detail |
|---|---|
| Architecture decisions documented | ADR for each major decision |
| ADR format: Context, Decision, Consequences | Standardized template |
| ADRs stored in `docs/adr/` | Versioned with code |
| Diagrams kept in sync with code | Mermaid diagrams in this document |

### 16.3 Feature Documentation

| Rule | Detail |
|---|---|
| Every feature has a Screen-ID | No feature without ID |
| Feature descriptions are user-facing | Written from user perspective |
| CRUD operations documented | Create, Read, Update, Delete for each entity |
| Error scenarios documented | What happens when feature fails |

---

## 17. Onboarding Rules

### 17.1 New Developer Setup

| Step | Action |
|---|---|
| 1. Clone repo | `git clone` project |
| 2. Install Rust | `rustup default stable` |
| 3. Install Node.js | `nvm use` (version in `.nvmrc`) |
| 4. Install dependencies | `cargo build` + `npm install` |
| 5. Run tests | `cargo test` + `npm test` |
| 6. Read system design | Start with `06-system-design.md` |
| 7. Read feature plan | `06-features.md` for what to build |
| 8. Read rules | `06-rules.md` for how to code |

### 17.2 First PR Checklist

- [ ] Read `06-rules.md`
- [ ] Follow naming conventions
- [ ] Write tests for new code
- [ ] All CI gates pass
- [ ] PR description explains what and why
- [ ] No `unwrap()` / `expect()` introduced
- [ ] No `any` type introduced
- [ ] i18n keys used for user-facing strings

---

## 18. Definition of Done

A feature is "Done" when:

| Criterion | Check |
|---|---|
| Feature implemented | Code written and compiles |
| Tests written | Unit + integration tests pass |
| Documentation updated | Feature plan + rules if needed |
| CI gates pass | Lint, format, typecheck, test, coverage |
| PR reviewed | At least 1 approving review |
| No CI failures | All checks green |
| User-facing strings externalized | i18n keys used, no hardcoded text |
| Error states handled | Loading, error, empty states covered |
| Performance within budget | Meets performance budgets |
| Accessible | Keyboard navigable, screen reader friendly |
| Security reviewed | No hardcoded credentials, SQL injection safe |

---

## 19. Technical Debt Tracking

| Rule | Detail |
|---|---|
| Every tech debt item has a ticket | No anonymous tech debt |
| Tech debt tickets tagged `tech-debt` | Easy to filter and prioritize |
| Tech debt reviewed in sprint planning | Allocate 10-20% of sprint capacity |
| No tech debt without remediation plan | Every debt item must have a fix plan |
| Tech debt ratio tracked | Target < 15% of total codebase |
| No new tech debt without approval | Team lead must approve new tech debt |