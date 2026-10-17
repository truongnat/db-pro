# Issue #458 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/458
- Fix: SqlServerConnector::query_uncapped bypasses user query max_rows cap so catalog introspection never truncates large databases with >500 tables/columns.
