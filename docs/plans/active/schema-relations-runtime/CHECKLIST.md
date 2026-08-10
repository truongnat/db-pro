# S3 — Schema Relations Runtime Checklist

## Analysis
- [x] evidence-backed composite-FK failure scenario recorded
- [x] scope/non-goals explicit
- [x] PostgreSQL/SQLite provider matrix explicit

## Implementation
- [x] SQLite FK rows use shared PRAGMA constraint identity
- [x] composite column order preserved
- [x] reconstructed DDL emits one multi-column constraint
- [x] relation UI groups mapping rows into one constraint
- [x] target-table navigation preserved
- [x] SQLite add-FK limitation remains capability-gated

## Regression tests added
- [x] frontend composite grouping test
- [x] backend reconstructed composite DDL test
- [x] SQLite composite FK introspection/enforcement test
- [ ] tests actually executed and recorded

## Provider/runtime evidence
- [ ] SQLite composite relation test PASS
- [ ] PostgreSQL composite FK introspection verified against live provider
- [ ] PostgreSQL reconstructed DDL checked for one ordered constraint
- [ ] Foreign Keys UI displays one composite relation row
- [ ] target navigation opens the correct table

## Review gate
- [ ] independent reviewer result recorded
- [ ] P0 = 0
- [ ] P1 = 0
- [ ] quality gates actually executed
- [ ] VERIFICATION.md updated with observed evidence
