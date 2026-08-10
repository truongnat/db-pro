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

Every PR goes through an adversarial multi-agent review pipeline before merge.

```
PR opened
  │
  ├─ CI (lint / test / build / clippy)
  │
  ├─ Gemini Security Reviewer
  │     Focus: SQL injection, safety bypass, input validation
  │
  ├─ Gemini Correctness Reviewer
  │     Focus: transaction atomicity, PG vs SQLite, precision, rollback
  │
  └─ Kilo Code Review
        Focus: bugs, architecture, tests (uses REVIEW.md from base branch)
              │
              ▼
        ADVERSARIAL ROUND
        @kilocode-bot challenges all P0/P1 findings
        CONFIRM / REJECT / DOWNGRADE each with evidence
              │
              ▼
        ARBITER (ChatGPT)
        Deduplicate, reject false positives
        Output: ACCEPTED_P0, ACCEPTED_P1, REJECTED, DISAGREEMENTS
              │
              ▼
        Implementer (Jules) fixes only accepted findings
              │
              ▼
        All reviewers rerun on new commit
              │
              ▼
        P0/P1 = 0 → MERGE
```

### Roles

| Role | Agent | Responsibility |
|------|-------|----------------|
| Implementer | Jules | Write code, fix accepted findings |
| Security reviewer | Gemini | Red-team SQL injection, safety bypass |
| Correctness reviewer | Gemini | DB semantics, transactions, precision |
| Independent reviewer | Kilo | Bugs, architecture, tests (REVIEW.md) |
| Adversarial challenger | Kilo (@kilocode-bot) | Challenge prior findings |
| Arbiter | ChatGPT | Deduplicate, reject false positives |

### Principles

- Reviewer diversity > reviewer count
- Reviewers do NOT read each other's conclusions (blind review)
- Implementer never self-judges; arbiter never codes
- Source reasoning != runtime evidence
- REVIEW.md lives on base branch (code under review cannot rewrite its own rubric)

### Kilo setup

```
app.kilo.ai → Integrations → GitHub → install Kilo Code Bot
Code Reviews → Enable AI Code Review: ON
Repository: truongnat/db-pro
Style: Strict
Focus: Security, Bugs, Testing, Performance
Use REVIEW.md: ON
```

### Gemini setup

```
GitHub repo → Settings → Secrets → GEMINI_API_KEY
Workflow: .github/workflows/gemini-review.yml
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
