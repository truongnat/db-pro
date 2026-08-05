# DB Client — CI/CD & Infrastructure Tasks

---

## 1. Repository Setup

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| C-001 | Initialize git repository | `git init`, create `.gitignore` for Rust, Node, Tauri | — | 30m |
| C-002 | Create `.gitignore` | Ignore `target/`, `node_modules/`, `dist/`, `.env`, `Cargo.lock` (keep it), `package-lock.json` (keep it) | C-001 | 30m |
| C-003 | Create `README.md` | Project overview, setup instructions, architecture overview | C-001 | 1h |
| C-004 | Create `CHANGELOG.md` | Changelog template with `[FEATURE]` / `[FIX]` / `[CHORE]` / `[BREAKING]` format | C-001 | 30m |

## 2. GitHub Actions Workflow

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| C-005 | Create `.github/workflows/ci.yml` | Rust: clippy, fmt, test, tarpaulin, deny, madge, audit, doc check | C-001 | 2h |
| C-006 | Create `.github/workflows/ci.yml` TS section | ESLint, prettier, tsc, vitest, coverage, playwright | C-005 | 1h |
| C-007 | Create `.github/workflows/release.yml` | Release branch, build .deb + AppImage, sign, publish | C-005 | 2h |
| C-008 | Create `.github/workflows/pr.yml` | PR validation: lint, format, typecheck, test, coverage | C-005 | 1h |

## 3. Pre-commit Hooks

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| C-009 | Install `pre-commit` framework | `pip install pre-commit` or use `cargo install pre-commit` | C-001 | 30m |
| C-010 | Create `.pre-commit-config.yaml` | Rust fmt, TS fmt, Rust lint, TS lint, TS typecheck | C-009 | 1h |
| C-011 | Configure `rustfmt` auto-format | `.rustfmt.toml` with project conventions | C-009 | 30m |
| C-012 | Configure `prettier` auto-format | `.prettierrc` with project conventions | C-009 | 30m |
| C-013 | Test pre-commit hooks | Make commit, verify hooks run | C-010 | 30m |

## 4. Linting & Formatting Setup

### 4.1 Rust

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| C-014 | Configure `rust-toolchain.toml` | Pin stable Rust version | C-001 | 15m |
| C-015 | Configure `.rustfmt.toml` | Edition 2021, max_width 120, tab_spaces 4 | C-014 | 15m |
| C-016 | Configure `clippy` warnings | `#![warn(clippy::all)]` in lib.rs, deny in CI | C-014 | 30m |
| C-017 | Set up `cargo deny` | `cargo-deny` installed, `deny.toml` configured | C-014 | 1h |
| C-018 | Set up `cargo audit` | `cargo-audit` installed, CI check | C-014 | 30m |
| C-019 | Set up `madge` for cycle detection | `madge --cycles src/` in CI | C-014 | 30m |
| C-020 | Set up `cargo tarpaulin` | `cargo tarpaulin` config, coverage thresholds | C-014 | 30m |

### 4.2 TypeScript

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| C-021 | Configure `eslint 9` | Flat config, `@typescript-eslint`, rules from `02-rules.md` | F-013 | 1h |
| C-022 | Configure `prettier` | `.prettierrc`, `.prettierignore` | F-013 | 30m |
| C-023 | Configure `tsconfig.json` | `strict: true`, `noImplicitAny`, path aliases, `baseUrl: "."` | F-012 | 30m |
| C-024 | Set up `vitest` | `vitest.config.ts`, test environment, coverage config | F-016 | 1h |
| C-025 | Set up `playwright` | `playwright.config.ts`, browsers installed, test fixtures | F-016 | 1h |

## 5. Build Configuration

### 5.1 Tauri Build

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| C-026 | Create `tauri.conf.json` | App name, identifier, version, build config, Tauri config | F-014 | 1h |
| C-027 | Create `build.rs` | Build script for Tauri app | C-026 | 30m |
| C-028 | Configure `.deb` packaging | `tauri.conf.json` bundle settings for .deb | F-149 | 1h |
| C-029 | Configure `.AppImage` packaging | `tauri.conf.json` bundle settings for AppImage | F-150 | 1h |
| C-030 | Configure `.flatpak` packaging (future) | Placeholder config for Flatpak | C-029 | 30m |

### 5.2 Build Scripts

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| C-031 | Create `scripts/build.sh` | Build script: cargo build + tauri build | C-026 | 30m |
| C-032 | Create `scripts/test.sh` | Test script: cargo test + npm test | C-005 | 30m |
| C-033 | Create `scripts/lint.sh` | Lint script: cargo clippy + eslint + tsc | C-005 | 30m |
| C-034 | Create `scripts/format.sh` | Format script: cargo fmt + prettier | C-005 | 30m |

## 6. Versioning & Release

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| C-035 | Set initial version | `Cargo.toml` version = `0.1.0`, `package.json` version = `0.1.0` | C-004 | 15m |
| C-036 | Create version bump script | `scripts/bump-version.sh` updating Cargo.toml, package.json, CHANGELOG | C-035 | 30m |
| C-037 | Create release process doc | Document release steps from `02-rules.md` Section 10 | C-035 | 30m |
| C-038 | Set up GitHub releases | Configure `release.yml` to auto-create GitHub releases | C-007 | 1h |

## 7. Dependency Management

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| C-039 | Pin Rust dependencies | No `^` or `~` in `Cargo.toml`, exact versions | C-017 | 30m |
| C-040 | Pin TS dependencies | No `^` or `~` in `package.json`, exact versions | C-021 | 30m |
| C-041 | Create dependency audit script | `cargo audit` + `npm audit` weekly check | C-039, C-040 | 30m |
| C-042 | Create dependency update script | `cargo update` + `npm update` with PR template | C-039, C-040 | 30m |
| C-043 | Verify lock files committed | `Cargo.lock` and `package-lock.json` in VCS | C-039, C-040 | 15m |

## 8. Documentation Infrastructure

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| C-044 | Create `docs/adr/` directory | Architecture Decision Records folder | C-001 | 15m |
| C-045 | Create ADR template | `adr/0001-template.md` with Context, Decision, Consequences sections | C-044 | 30m |
| C-046 | Create ADR for Tauri architecture | Document why Tauri 2 was chosen over Electron | C-045 | 1h |
| C-047 | Create ADR for Clean Architecture | Document the dependency graph and layering decisions | C-045 | 1h |
| C-048 | Create ADR for PostgreSQL-first | Document why PostgreSQL is the primary target | C-045 | 30m |
| C-049 | Create ADR for Monaco editor | Document why Monaco was chosen for SQL editing | C-045 | 30m |
| C-050 | Create ADR for TanStack Query | Document why TanStack Query was chosen for server state | C-045 | 30m |

## 9. Development Environment

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| C-051 | Create `CONTRIBUTING.md` | Contribution guidelines, setup, PR process | C-003 | 1h |
| C-052 | Create `.nvmrc` | Node.js version pin | F-016 | 15m |
| C-053 | Create `Dockerfile` for dev | Dev container with Rust + Node.js | C-001 | 1h |
| C-054 | Create `docker-compose.yml` | PostgreSQL test database + Redis (if needed) | C-053 | 30m |
| C-055 | Create `Makefile` | Common commands: `make build`, `make test`, `make lint`, `make dev` | C-031-C-034 | 1h |
| C-056 | Verify `make dev` works | Dev environment starts with one command | C-055 | 1h |

## 10. Monitoring & Observability (Post-MVP)

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| C-057 | Set up `tracing` subscriber | Structured logging with JSON output | B-055 | 1h |
| C-058 | Create log rotation config | Weekly rotation for SQLite meta-store logs | C-057 | 30m |
| C-059 | Create performance metrics | App startup time, query execution time, memory usage tracking | C-057 | 2h |
| C-060 | Create error reporting | Aggregate errors from meta-store for dashboard | C-058 | 2h |
