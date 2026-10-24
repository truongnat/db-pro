---
schema_version: 1
id: "MEM-SRC-0a64327a44679f78"
title: "Source mirror: docs/plans/active/postgres-autocommit-search-path/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/postgres-autocommit-search-path/PLAN.md"
source_sha256: "0a64327a44679f783c80af0657added1aaec532e143f020214dd82af61892dc5"
mirror_body_sha256: "0a64327a44679f783c80af0657added1aaec532e143f020214dd82af61892dc5"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Postgres autocommit search_path — PLAN

- Status: REVIEW (backend fix verified live on PostgreSQL; app UAT pending)
- Reported: with tenant `tenant1` selected in the query tab, `SELECT * FROM m_ctrl_universal_value;`
  failed with `relation "m_ctrl_universal_value" does not exist`.
- Root cause: `push_session_search_path` installed the editor schema with
  `set_config('search_path', ..., is_local = true)` unconditionally. PostgreSQL applies a local
  setting only to the current transaction; outside one it expires at the end of the `set_config`
  statement, so the following `query` / `execute` / `explain` still resolved names against the old
  path. The unconditional local scope came from #456 (`424258ce`), which needed it to stop
  search_path leakage when a timed-out future is dropped before the restore.
- Fix:
  - `push_session_search_path` / `pop_session_search_path` take the connection's transaction state:
    `is_local = true` inside a held tab transaction, `false` on an autocommit connection
    (`crates/infrastructure/src/postgres/search_path.rs:18`, `:51`).
  - An autocommit run that installed a session-wide path arms
    `SessionLease::discard_if_abandoned()` (`crates/infrastructure/src/postgres/connector.rs:176`,
    the shared `begin_scoped_run`; EXPLAIN at `:1014`); a completed restore calls `allow_reuse()`
    (`:193`). A future dropped between push and
    pop closes the connection through `close_on_drop` instead of returning the leaked path to the
    pool (`crates/infrastructure/src/transaction_session.rs:101`-`:117`), preserving #456.
  - Batch and joined transaction paths stay transaction-local (`connector.rs:217`, `:280`).
- Providers: PostgreSQL only. SQLite, MySQL and SQL Server paths are untouched.
- Out of scope: the explorer-vs-document schema fallback in run preparation still sends the
  document schema only; the query-tab picker sets that document schema, so this report is fixed.
