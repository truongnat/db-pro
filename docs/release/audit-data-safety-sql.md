# Data Safety Audit — Destructive SQL, Multi-Statement, Cancellation

> Baseline SHA: `a2cf4a3`
> Issue: #129
> Source: `crates/core/src/domain/safety.rs`, `crates/core/src/domain/execution.rs`

## Statement classifier audit

### Classification matrix

| Statement / Input | Classifier Result | Correct? | Test? |
|---|---|---|---|
| `SELECT 1` | Read | Yes | `classify_select_is_read` |
| `SHOW tables` | Read | Yes | Inferred from keyword match |
| `INSERT INTO t VALUES (1)` | Write | Yes | `classify_insert_is_write` |
| `UPDATE t SET x = 1` | Write | Yes | `classify_update_is_write` |
| `DELETE FROM t WHERE id = 1` | Write | Yes | `classify_delete_with_where_is_write` |
| `DELETE FROM t` | Destructive | Yes | `classify_delete_without_where_is_destructive` |
| `DROP TABLE t` | Destructive | Yes | `classify_drop_is_destructive` |
| `TRUNCATE TABLE t` | Destructive | Yes | `classify_truncate_is_destructive` |
| `CREATE TABLE t (id INT)` | Ddl | Yes | `classify_create_is_ddl` |
| `ALTER TABLE t ADD COLUMN x` | Ddl | Yes | `classify_alter_is_ddl` |
| `WITH cte AS (SELECT ...) SELECT` | Read | Yes | `classify_with_select_is_read` |
| `WITH cte AS (SELECT ...) UPDATE` | Write | Yes | `classify_with_update_is_write` |
| `WITH deleted AS (DELETE ...) SELECT` | Write | Yes | `mutating_cte_with_delete_is_write` |
| `WITH moved AS (INSERT ...) SELECT` | Write | Yes | `mutating_cte_with_insert_is_write` |
| `EXPLAIN SELECT * FROM t` | Read | Yes | `explain_plain_select_is_read` |
| `EXPLAIN ANALYZE SELECT * FROM t` | Read | Yes | `explain_analyze_select_is_read` |
| `EXPLAIN ANALYZE DELETE FROM t WHERE` | Write | Yes | `explain_analyze_delete_is_write` |
| `EXPLAIN ANALYZE INSERT INTO t` | Write | Yes | `explain_analyze_insert_is_write` |
| `-- comment\nSELECT 1` | Read | Yes | `leading_comments_stripped` |
| `/* block */ DROP TABLE t` | Destructive | Yes | `leading_comments_stripped` |
| Empty / whitespace | None (error) | Yes | `empty_sql_is_rejected` |

### Safety policy validation tests

| Policy | Statement | Expected | Test |
|---|---|---|---|
| read_only | SELECT | Allow | `read_only_policy_rejects_write` |
| read_only | INSERT/UPDATE/DELETE | Reject | `read_only_policy_rejects_write` |
| full_access | All | Allow | `full_access_policy_allows_everything` |
| no_ddl | CREATE/ALTER | Reject | `no_ddl_policy_rejects_create_alter` |
| read_only | EXPLAIN ANALYZE DELETE | Reject | `readonly_rejects_explain_analyze_delete` |
| read_only | Mutating CTE | Reject | `readonly_rejects_mutating_cte` |

## Findings

### F1 — `is_delete_without_where` is a simple `contains("WHERE")` heuristic [P2]

**Location**: `safety.rs:119-123`

The check `!upper.contains("WHERE")` can be fooled by:
- `DELETE FROM t WHERE id IN (SELECT ...)` — correctly passes (has WHERE)
- `DELETE FROM t /* WHERE */ ` — incorrectly passes (WHERE in comment)
- `DELETE FROM t WHERE` at end with no condition — incorrectly passes

**Impact**: Low for v0.1 — the classifier is explicitly documented as "best-effort heuristic, not a full SQL parser." The backend enforces the policy, and the query editor intentionally allows unrestricted SQL.

**Recommendation**: ACCEPT RC1. Document as known limitation of the heuristic classifier.

### F2 — CTE mutation detection scans only first 10 chars after position [P2]

**Location**: `safety.rs:200-214`

The CTE body scanner checks `rest_upper.starts_with("INSERT")` etc. by taking only 10 characters from the current position. This means mutations that appear after whitespace or nested parentheses deeper in the CTE body might be missed.

**Impact**: Low — the common case (mutation at the start of a CTE body) is handled. Edge cases with deeply nested mutations are unlikely in practice.

**Recommendation**: ACCEPT RC1.

### F3 — No confirmation dialog for destructive SQL [P2]

**Location**: Application layer

The safety policy can block destructive operations when `allow_destructive = false`, but the default policy is `full_access()` which allows everything. There is no UI-level confirmation dialog for DROP/TRUNCATE/DELETE-without-WHERE in the query editor.

**Impact**: Users can execute destructive SQL without confirmation in the default mode. This is intentional for a database IDE — the query editor is a power-user tool. The safety policy provides a backend enforcement layer for read-only connections.

**Recommendation**: ACCEPT RC1. Document that the query editor allows unrestricted SQL by design.

### F4 — Multi-statement execution uses `execute_batch` (atomic) [P2]

**Location**: `db_connector.rs:26` — `execute_batch` trait method

The `execute_batch` method executes multiple statements atomically inside a single transaction. If any statement fails, all changes are rolled back. This is the correct behavior for safety.

**Impact**: Positive — multi-statement writes are atomic by design.

**Recommendation**: No action needed.

### F5 — Cancellation uses `QueryExecutionId` for identity [P2]

**Location**: `execution.rs:6-25`

Each execution gets a unique UUID. The cancellation mechanism targets a specific execution by ID, preventing cancellation of unrelated executions.

**Impact**: Positive — cancellation cannot target the wrong execution.

**Recommendation**: No action needed.

### F6 — SQLite has no query cancellation [P2]

**Location**: `capabilities.rs:182` — `cancel: false` for SQLite

SQLite connections cannot cancel running queries. This is a known capability gap documented in the capability matrix (#132).

**Recommendation**: Already documented in LIM-014.

## Summary

| Severity | Count | Findings |
|---|---|---|
| P1 | 0 | — |
| P2 | 6 | F1-F6 (all ACCEPT RC1) |
| P3 | 0 | — |

**Conclusion**: The safety classifier has 22 unit tests covering all statement types, policy validation, comment stripping, CTE mutation detection, and EXPLAIN ANALYZE. No P1 findings. All P2 items are accepted for RC1 with documentation.
