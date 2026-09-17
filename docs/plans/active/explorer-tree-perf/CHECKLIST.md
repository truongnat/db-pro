# Explorer Tree Perf — Checklist

- [x] Add `ExplorerNavCache` keyed by connection/schema/search
- [x] Clear cache on schema load and schema activation
- [x] Schema-scoped table count / matching count without name Vec alloc
- [x] Tables folder default closed; auto-open on search
- [x] Viewport cull + `EXPLORER_MAX_TABLES` truncate hint
- [x] Defer Views/Functions/Triggers filter until folder open
- [x] Schema-scoped folder persistence IDs
- [x] Unit test for matching count
- [ ] Runtime verify on large schema (PostgreSQL + SQLite)
- [x] Quality gates: check / clippy / targeted tests
