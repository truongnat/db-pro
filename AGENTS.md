# DB Pro — Agent Instructions

## Product

DB Pro is a desktop Database IDE / Agent IDE.

Tech:
- Tauri 2
- Rust workspace (core / infrastructure / tauri-app)
- React 19
- TypeScript
- TanStack Query / Router / Virtual
- shadcn/ui + Radix UI
- Tailwind CSS
- PostgreSQL + SQLite

## Canonical feature lifecycle

Read `docs/plans/FEATURE_LIFECYCLE.md` before non-trivial work.

Allowed states:

```text
BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED
```

`BLOCKED` is allowed when an external dependency prevents progress.

Never mark a feature `COMPLETED` when runtime/provider evidence is still pending.
Never leave a `RUNTIME_VERIFY` feature under `docs/plans/completed/`.

## Working model

Never implement a non-trivial feature directly on main.

For every feature:

1. Read:
   - `docs/plans/STATUS.md`
   - `docs/plans/FEATURE_LIFECYCLE.md`
   - relevant `docs/plans/active/<feature>/*`
   - architecture documentation

2. Create or work on:
   - `feature/<feature-slug>` for product work
   - `fix/<feature-slug>` for focused correctness/security fixes

3. Maintain exactly one plan directory:

```text
docs/plans/active/<feature>/
  PLAN.md
  CHECKLIST.md
  FINDINGS.md
  VERIFICATION.md
```

4. Implement in coherent commits.
5. Do not silently expand scope.
6. Before publishing a PR:
   - self-review architecture
   - review correctness/security
   - review test coverage
   - record remaining P0/P1/P2
7. Move the plan to `docs/plans/completed/` only in the change that actually satisfies all completion gates.

## Analyst-first policy

For autonomous or scheduled work, never start coding immediately after identifying a suspicious pattern.

First establish:

1. Evidence — concrete code path, invariant, or failing test
2. Failure scenario — how the behavior fails in a realistic case
3. Severity — P0/P1/P2 with justification
4. Scope — smallest coherent fix
5. Existing coverage — whether another active PR already addresses it
6. Provider impact — PostgreSQL and SQLite independently
7. Runtime testability — what evidence can actually be collected

A suspicious code pattern is not automatically a bug.
Prefer proving one important defect over fixing five speculative issues.
One autonomous analysis run should produce at most one focused PR.

## Severity

P0:
- catastrophic data loss
- security-critical issue
- application unusable

P1:
- data corruption
- wrong database mutation
- unsafe SQL
- broken transaction semantics
- stale state causing incorrect behavior
- provider-specific correctness failure
- major user flow broken

P2:
- non-blocking UX
- maintainability
- missing edge-case coverage
- polish

A PR must not be marked ready with P0/P1 open.

## Database safety

Never:
- concatenate untrusted SQL identifiers
- concatenate untrusted SQL literals
- bypass safety policy
- bypass confirmation policy
- claim atomicity without a transaction
- treat affectedRows=0 as mutation success
- silently coerce precision-sensitive values
- infer one provider's runtime behavior from another provider's tests

All provider-specific behavior must respect explicit capabilities.
Unsupported operations must be capability-gated with a clear reason instead of emitting unsupported SQL.

## Frontend quality gates

```bash
cd frontend
npm run typecheck
npm run lint
npm run format:check
npm run check:tokens
npm run test
npm run build
```

`check:tokens` enforces the design token contract (P3.1): canonical `--surface-*`/`--text-*`/`--border-*`/`--accent-*`/`--state-*` tokens are the single source of truth, the shadcn compatibility layer must only alias them, and components must not reintroduce `--app-*` color tokens or raw shadcn semantic vars. `npx shadcn add` must not modify the token layers in `src/styles/globals.css` — CI catches drift.

## Rust quality gates

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets
cargo test --workspace
```

Do not claim a command passed unless it was actually executed.

## Performance audit

Use the `perf-audit` skill when:
- Working on performance-sensitive features (ER diagrams, query execution, large schemas, data grid)
- Before publishing a PR that touches rendering, bundling, or backend hot paths
- Investigating performance regressions or user-reported slowness
- Adding new dependencies or large modules

Quick check before PR:

```bash
bash ~/.qoder/skills/perf-audit/scripts/perf-scan.sh
```

Performance budgets are enforced in:
- `frontend/src/commons/__tests__/performance-budgets.test.ts`
- `crates/infrastructure/benches/sqlite_benchmarks.rs`
- `docs/architecture/performance-baseline.md`

Never claim performance improved without measurement evidence.

## PR workflow

Publish a PR instead of pushing a non-trivial feature directly to main.

PR must reference:
- PLAN.md
- CHECKLIST.md
- VERIFICATION.md

Report:
- commits
- files changed
- tests actually executed
- runtime evidence actually collected
- P0/P1/P2
- known limitations

## Runtime verification

Source inspection alone is not runtime evidence.

For database features verify when applicable:

```text
UI
→ command
→ backend
→ database
→ introspection
→ refreshed UI state
```

Every database-facing feature must independently account for PostgreSQL and SQLite as supported, unsupported, or pending.

## Review infrastructure

The repository intentionally uses a read-only VPS/Kilo reviewer.
Do not delete these as cleanup unless the task explicitly replaces the review system:

- `REVIEW.md`
- `.kilo/`
- `.github/review-prompts/`
- `.github/workflows/vps-pr-review.yml`

The implementer must not treat its own self-review as independent approval. P0/P1 findings from external reviewers must be resolved, rejected, or downgraded with evidence before merge.
