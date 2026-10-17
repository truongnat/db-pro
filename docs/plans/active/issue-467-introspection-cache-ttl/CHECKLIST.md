# Issue #467 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/467
- Fix: SQLiteMetaStore::get enforces a 10-minute TTL on cached introspection results, invalidating older entries so remote schema changes do not remain cached indefinitely.
