import { splitStatementsWithRanges } from "../services/statement-splitter";

import type { ActionRisk } from "@/commons/actions/types";

/**
 * Canonical SQL safety classifier.
 *
 * Classifies a single SQL statement into a risk level:
 *   - read:        SELECT, WITH...SELECT, EXPLAIN — no side effects
 *   - write:       INSERT, UPDATE, ALTER, CREATE, REPLACE — modifies data/schema
 *   - destructive: DELETE, DROP, TRUNCATE — potentially irreversible
 *
 * Handles:
 *   - Leading comments (-- and block comments)
 *   - WITH mutating CTE (INSERT/UPDATE/DELETE inside WITH)
 *   - EXPLAIN [ANALYZE] [VERBOSE] mutation (both bare and parenthesized)
 *   - String literals (don't false-positive on keywords inside strings)
 */
export function classifySqlRisk(sql: string): ActionRisk {
  // Strip leading block and line comments.
  let stripped = sql.trim();
  while (true) {
    if (stripped.startsWith("--")) {
      const nl = stripped.indexOf("\n");
      if (nl === -1) return "read"; // only comments
      stripped = stripped.slice(nl + 1).trim();
      continue;
    }
    if (stripped.startsWith("/*")) {
      const end = stripped.indexOf("*/");
      if (end === -1) return "read"; // unclosed comment
      stripped = stripped.slice(end + 2).trim();
      continue;
    }
    break;
  }

  if (!stripped) return "read";

  const upper = stripped.toUpperCase();

  // EXPLAIN [ANALYZE] [VERBOSE] — classify the inner statement.
  // Handles both:
  //   EXPLAIN ANALYZE DELETE FROM users
  //   EXPLAIN (ANALYZE, VERBOSE) DELETE FROM users
  if (/^EXPLAIN\b/i.test(upper)) {
    let inner = stripped.replace(/^EXPLAIN\b/i, "").trim();
    // Strip optional parenthesized options: (ANALYZE, VERBOSE, ...)
    if (inner.startsWith("(")) {
      const closeParen = inner.indexOf(")");
      if (closeParen !== -1) {
        inner = inner.slice(closeParen + 1).trim();
      }
    }
    // Strip optional ANALYZE / VERBOSE keywords (non-parenthesized form)
    inner = inner
      .replace(/^\s*ANALYZE\b/i, "")
      .replace(/^\s*VERBOSE\b/i, "")
      .trim();
    if (inner) return classifySqlRisk(inner);
    return "read";
  }

  // WITH ... mutating CTE detection.
  // Look for INSERT/UPDATE/DELETE inside WITH blocks.
  if (/^WITH\b/i.test(upper)) {
    // Check if the WITH body contains mutating keywords.
    // This is a heuristic — we look for DML keywords after the WITH ... AS.
    const body = upper.replace(/^WITH\b[\s\S]*?\bAS\b\s*\(/, "");
    if (/\b(INSERT|UPDATE|DELETE)\b/.test(body)) return "destructive";
    return "read";
  }

  // Strip string literals to avoid false positives on keywords inside strings.
  const noStrings = upper.replace(/'[^']*'/g, "''");

  // Destructive: DROP, TRUNCATE, DELETE
  if (/^(DROP|TRUNCATE)\b/.test(noStrings)) return "destructive";
  if (/^DELETE\b/.test(noStrings)) return "destructive";

  // Write: INSERT, UPDATE, ALTER, CREATE, REPLACE
  if (/^(INSERT|UPDATE|ALTER|CREATE|REPLACE)\b/.test(noStrings)) return "write";

  // Default: read-only
  return "read";
}

/**
 * Risk level ordering for comparison.
 */
const RISK_SEVERITY: Record<ActionRisk, number> = {
  read: 0,
  write: 1,
  destructive: 2,
  dynamic: 0, // dynamic is not a concrete risk level
};

/**
 * Classify the risk of a multi-statement SQL script.
 *
 * Splits the script into individual statements, classifies each,
 * and returns the maximum risk level across all statements.
 *
 * Example:
 *   SELECT 1; DROP TABLE users;
 *   → statements: ["SELECT 1", "DROP TABLE users"]
 *   → risks: ["read", "destructive"]
 *   → result: "destructive"
 */
export function classifyScriptRisk(sql: string): ActionRisk {
  const statements = splitStatementsWithRanges(sql);
  if (statements.length === 0) return "read";

  let maxRisk: ActionRisk = "read";
  for (const stmt of statements) {
    const risk = classifySqlRisk(stmt.sql);
    if (RISK_SEVERITY[risk] > RISK_SEVERITY[maxRisk]) {
      maxRisk = risk;
    }
  }
  return maxRisk;
}
