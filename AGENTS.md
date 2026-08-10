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

## Working model

Never implement a non-trivial feature directly on main.

For every feature:

1. Read:
   - docs/plans/STATUS.md
   - relevant docs/plans/active/<feature>/*
   - architecture documentation

2. Create or work on:
   feature/<feature-slug>

3. Maintain:
   docs/plans/active/<feature>/
     PLAN.md
     CHECKLIST.md
     FINDINGS.md
     VERIFICATION.md

4. Implement in coherent commits.

5. Do not silently expand scope.

6. Before publishing a PR:
   - self-review architecture
   - review correctness/security
   - review test coverage
   - record remaining P0/P1/P2

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

All provider-specific behavior must respect capabilities.

## Frontend quality gates

```bash
cd frontend

npm run typecheck
npm run lint
npm run format:check
npm run test
npm run build
```

## Rust quality gates

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets
cargo test --workspace
```

Do not claim a command passed unless it was actually executed.

## PR workflow

Publish a PR instead of pushing directly to main.

PR must reference:
- PLAN.md
- CHECKLIST.md
- VERIFICATION.md

Report:
- commits
- files changed
- tests executed
- runtime evidence
- P0/P1/P2
- known limitations

## Review Council

Every PR goes through an adversarial review pipeline before merge.

```
PR opened
  │
  ├─ CI (lint / test / build / clippy)
  │
  └─ Cubic AI Review (auto on PR open/update)
        Focus: security, bugs, testing, performance
        Reads REVIEW.md from base branch
              │
              ▼
        ARBITER (ChatGPT)
        Review Cubic findings
        ACCEPT / REJECT / DOWNGRADE each with evidence
              │
              ▼
        Implementer (Jules) fixes accepted findings
              │
              ▼
        Cubic re-reviews on new commit
              │
              ▼
        P0/P1 = 0 → MERGE
```

### Roles

| Role | Agent | Responsibility |
|------|-------|----------------|
| Implementer | Jules / Qoder | Write code, fix accepted findings |
| AI reviewer | Cubic AI | Auto-review on PR open/update |
| Arbiter | ChatGPT | Accept/reject findings, resolve disagreements |
| Human reviewer | Owner | Final merge decision |

### Principles

- Implementer never self-judges; arbiter never codes
- Source reasoning != runtime evidence
- REVIEW.md lives on base branch (code under review cannot rewrite its own rubric)
- Cubic AI is free for public repos (unlimited PR reviews)

### Cubic AI setup

```
cubic.dev → sign in with GitHub
Install Cubic AI GitHub App on truongnat/db-pro
Auto-review: ON
```

## Runtime verification

Source inspection alone is not runtime evidence.

For database features verify when environment allows:

```
UI
→ command
→ backend
→ database
→ introspection
→ refreshed UI state
```

Never mark runtime verification complete based only on source reasoning.
