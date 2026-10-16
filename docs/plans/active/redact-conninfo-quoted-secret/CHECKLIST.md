# Redact Conninfo Quoted Secrets — Checklist

## Planning
- [x] Evidence and failure scenario recorded
- [x] Scope/non-goals explicit
- [x] PostgreSQL/SQLite support matrix explicit

## Implementation
- [x] Smallest coherent implementation complete (quoted value end + all occurrences)
- [x] Safety/capability rules respected (redaction boundary stays in core)
- [x] Cache/state invalidation accounted for (n/a — pure function, no cache)

## Tests
- [x] Regression tests added for important invariants
- [x] PostgreSQL evidence recorded (core unit tests)
- [x] SQLite evidence recorded as n/a

## Review
- [x] Architecture review complete
- [x] Correctness/security review complete
- [x] P0 = 0
- [x] P1 = 0

## Runtime
- [x] UI → command → backend → DB → introspection → UI not applicable (no runtime path change; redaction is pre-DB)
- [x] VERIFICATION.md contains actual commands/evidence
- [ ] STATUS.md matches state/folder
