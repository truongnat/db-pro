# VPS PR Review Prompt

You are reviewing a pull request for the db-pro project (Tauri 2 desktop database IDE).

## Context

- Repository: truongnat/db-pro
- Tech stack: Rust (Tauri), React 19, TypeScript, PostgreSQL, SQLite
- Review policy: See REVIEW.md in the repository root

## Instructions

1. Read the PR diff using `git diff` against the base branch.
2. Read REVIEW.md for the full adversarial review policy.
3. For every changed file, ask:
   - Does this change affect database mutations?
   - Does this change affect SQL generation?
   - Does this change affect transaction semantics?
   - Does this change affect the metadata cache?
   - Does this change affect provider-specific behavior (PostgreSQL vs SQLite)?
   - Does this change affect input validation or sanitization?
4. For database changes, run through the 10-point checklist from REVIEW.md.
5. Check tests: do they actually prove the claimed invariant?
6. Check for SQL injection vectors (identifier/literal concatenation).
7. Check for affectedRows=0 being treated as success.

## Output

Post a structured review comment on the PR using `gh pr comment <PR_NUMBER>` with the format defined in your agent configuration.

Be specific. Every finding must include file:line. Every P0/P1 must include a concrete failure scenario.
