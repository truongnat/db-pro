# Driver-aware SQL String Splitting — Checklist

## Planning
- [x] Evidence and failure scenario recorded from issue #477
- [x] Scope and SQL-mode non-goals explicit
- [x] PostgreSQL, MySQL, SQLite and SQL Server matrix recorded

## Implementation
- [x] Dialect-aware quote scanning and driver-aware split API
- [x] Query single/multi, EXPLAIN, export and DDL use configured driver
- [x] SQLite transaction guard and query row cap use explicit driver
- [x] No unrelated behavior or database statements added

## Tests
- [x] PostgreSQL standard-string payload splits and safety gate rejects it
- [x] PostgreSQL E-string and MySQL escaped-string regressions
- [x] Focused domain, policy and row-limit tests
- [x] Workspace check and formatting/diff checks
- [ ] Live PostgreSQL/MySQL/SQLite/SQL Server runtime probes (pending)

## Review
- [x] Self architecture/correctness/security review
- [x] Clean-code scan; no new errors (three warnings reviewed as existing complexity/size)
- [x] P0 = 0; P1 = 0 in reviewed source diff
- [ ] Independent reviewer verdict

## Runtime
- [ ] Provider runtime evidence
- [x] VERIFICATION.md records commands and gaps
- [x] STATUS.md reflects review state
