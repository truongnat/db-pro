# S2 — Schema Indexes Runtime Checklist

## Source/implementation
- [x] index introspection exists for PostgreSQL
- [x] index introspection exists for SQLite
- [x] create/drop index UI exists
- [x] unique/composite index UI exists
- [x] provider-aware identifier quoting exists
- [x] schema cache invalidation path exists after DDL

## Automated evidence
- [x] SQLite CREATE UNIQUE INDEX introspection
- [x] SQLite composite CREATE INDEX introspection
- [x] SQLite DROP INDEX introspection
- [ ] PostgreSQL create/unique/composite/drop integration evidence

## UI/runtime evidence
- [ ] create index from UI → DB → introspection → refreshed UI
- [ ] drop index from UI → DB → introspection → refreshed UI
- [ ] PostgreSQL live provider matrix recorded
- [ ] SQLite UI round-trip recorded

## Review gate
- [ ] P0 = 0
- [ ] P1 = 0
- [ ] VERIFICATION.md contains all required runtime evidence
