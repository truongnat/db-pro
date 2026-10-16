# Context

Baseline: 76b120724a7542de8b587a002811ddacad28156a. Owner delegated scope approval with continuous issue work.
- infrastructure/src/sqlite/monitoring.rs ignored schema and emitted per-table VACUUM; VacuumAnalyze ran VACUUM only.
- infrastructure/src/error.rs mapped unknown extended constraints to Check; core ConstraintType lacked PrimaryKey/Other.
- Full literal enum-consumer search found no exhaustive consumers requiring migration; LSP references failed (server exits). Extend taxonomy without changing existing error code/message identifiers.
- Reuse real SQLiteConnector/MonitoringPort in integration tests and rusqlite engine errors for mapper unit tests. No mocks/source-text tests.
- Existing core/infra error behavior for PostgreSQL unchanged. SQLite-specific checks do not prove PG behavior; #403 UI/provider acceptance separate.
- Prefer explicit unsupported table VACUUM over a dangerous database fallback. VACUUM plus ANALYZE are ordered, not atomic.
