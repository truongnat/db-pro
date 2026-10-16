# RLS Policy Expression Validation — Checklist

## Planning
- [x] Evidence and failure scenario recorded
- [x] Scope/non-goals explicit
- [x] PostgreSQL/SQLite support matrix explicit

## Implementation
- [x] Smallest coherent implementation complete for #507
- [x] PostgreSQL parser validates a single complete expression
- [x] Both CREATE and ALTER paths validate USING and WITH CHECK
- [x] SQLite remains capability-gated as unsupported

## Tests
- [x] Regression tests for statement injection and valid expressions
- [x] PostgreSQL quote/backslash parser cases
- [x] Focused builder and UI preview tests (see VERIFICATION.md)
- [x] Live PostgreSQL mutation — not required; no database mutation was made
- [x] SQLite unsupported boundary is covered by existing source gate/tests

## Review
- [ ] Independent architecture/correctness/security review
- [ ] P0 = 0
- [ ] P1 = 0 after independent review

## Runtime
- [x] No database mutation performed; UI path produces a preview only
- [x] VERIFICATION.md records actual commands/evidence
- [x] STATUS.md matches this plan's state
