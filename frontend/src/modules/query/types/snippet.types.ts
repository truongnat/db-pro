/**
 * SQL Snippet — a short trigger that expands to a SQL template.
 *
 * Built-in snippets ship with the app and cannot be deleted.
 * Custom snippets are persisted in localStorage.
 */
export interface Snippet {
  /** Short trigger text, e.g. "sel", "ct". */
  trigger: string;
  /** Human-readable label shown in the snippet list. */
  label: string;
  /** SQL template. May contain `$cursor` placeholder. */
  body: string;
  /** Whether this is a built-in (non-deletable) snippet. */
  builtIn: boolean;
}

export const BUILT_IN_SNIPPETS: Snippet[] = [
  {
    trigger: "sel",
    label: "SELECT *",
    body: "SELECT * FROM $cursor;",
    builtIn: true,
  },
  {
    trigger: "selw",
    label: "SELECT … WHERE",
    body: "SELECT * FROM $cursor\nWHERE ",
    builtIn: true,
  },
  {
    trigger: "ins",
    label: "INSERT INTO",
    body: "INSERT INTO $cursor (\n  \n) VALUES (\n  \n);",
    builtIn: true,
  },
  {
    trigger: "upd",
    label: "UPDATE … SET",
    body: "UPDATE $cursor\nSET \nWHERE ;",
    builtIn: true,
  },
  {
    trigger: "del",
    label: "DELETE FROM",
    body: "DELETE FROM $cursor\nWHERE ;",
    builtIn: true,
  },
  {
    trigger: "ct",
    label: "CREATE TABLE",
    body: "CREATE TABLE $cursor (\n  id SERIAL PRIMARY KEY,\n  \n);",
    builtIn: true,
  },
  {
    trigger: "ati",
    label: "ALTER TABLE ADD",
    body: "ALTER TABLE $cursor ADD COLUMN ",
    builtIn: true,
  },
  {
    trigger: "dti",
    label: "DROP TABLE",
    body: "DROP TABLE IF EXISTS $cursor;",
    builtIn: true,
  },
  {
    trigger: "ci",
    label: "CREATE INDEX",
    body: "CREATE INDEX idx_$cursor ON $cursor ();",
    builtIn: true,
  },
  {
    trigger: "cnt",
    label: "COUNT",
    body: "SELECT COUNT(*) FROM $cursor;",
    builtIn: true,
  },
  {
    trigger: "grp",
    label: "GROUP BY",
    body: "SELECT $cursor, COUNT(*)\nFROM \nGROUP BY ;",
    builtIn: true,
  },
  {
    trigger: "joi",
    label: "JOIN … ON",
    body: "JOIN $cursor ON ",
    builtIn: true,
  },
  // Diagnostic queries (PostgreSQL)
  {
    trigger: "pgsize",
    label: "DB Size (PG)",
    body: "SELECT pg_size_pretty(pg_database_size(current_database())) AS database_size;",
    builtIn: true,
  },
  {
    trigger: "pgts",
    label: "Table Sizes (PG)",
    body: `SELECT
  schemaname,
  tablename,
  pg_size_pretty(pg_total_relation_size(schemaname || '.' || tablename)) AS total_size,
  pg_size_pretty(pg_relation_size(schemaname || '.' || tablename)) AS data_size,
  pg_size_pretty(pg_indexes_size(schemaname || '.' || tablename::regclass)) AS index_size
FROM pg_tables
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY pg_total_relation_size(schemaname || '.' || tablename) DESC
LIMIT 50;`,
    builtIn: true,
  },
  {
    trigger: "pgsess",
    label: "Active Sessions (PG)",
    body: `SELECT
  pid,
  usename AS username,
  datname AS database,
  state,
  query_start,
  NOW() - query_start AS duration,
  LEFT(query, 100) AS query
FROM pg_stat_activity
WHERE state != 'idle'
ORDER BY query_start DESC;`,
    builtIn: true,
  },
  {
    trigger: "pgidx",
    label: "Index Usage (PG)",
    body: `SELECT
  schemaname,
  tablename,
  indexname,
  idx_scan AS scans,
  idx_tup_read AS tuples_read,
  idx_tup_fetch AS tuples_fetched
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;`,
    builtIn: true,
  },
  {
    trigger: "pglock",
    label: "Locks (PG)",
    body: `SELECT
  l.pid,
  l.mode,
  l.granted,
  a.usename,
  a.query
FROM pg_locks l
JOIN pg_stat_activity a ON l.pid = a.pid
WHERE NOT l.granted
ORDER BY a.query_start;`,
    builtIn: true,
  },
  // Diagnostic queries (SQLite)
  {
    trigger: "slsize",
    label: "Table Sizes (SQLite)",
    body: `SELECT
  name AS table_name,
  (SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = m.name) AS exists_flag
FROM sqlite_master m
WHERE type = 'table'
ORDER BY name;`,
    builtIn: true,
  },
  {
    trigger: "slidx",
    label: "Indexes (SQLite)",
    body: `SELECT
  m.name AS table_name,
  i.name AS index_name,
  i.sql AS index_sql
FROM sqlite_master i
JOIN sqlite_master m ON i.tbl_name = m.name
WHERE i.type = 'index'
ORDER BY m.name, i.name;`,
    builtIn: true,
  },
];
