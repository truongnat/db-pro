import { splitStatementsWithRanges } from "../services/statement-splitter";

import type { ActionRisk } from "@/commons/actions/types";

/**
 * Mask string literals, double-quoted identifiers, line comments,
 * block comments, and Postgres dollar-quoted bodies with spaces.
 *
 * This prevents false positives when mutation keywords appear
 * inside strings, comments, or dollar-quoted function bodies.
 */
function maskNonCodeTokens(sql: string): string {
  let result = "";
  let i = 0;
  while (i < sql.length) {
    // Single-quoted string literal
    if (sql[i] === "'") {
      let j = i + 1;
      while (j < sql.length) {
        if (sql[j] === "'") {
          if (sql[j + 1] === "'") { j += 2; continue; }
          j++; break;
        }
        j++;
      }
      result += " ".repeat(j - i);
      i = j;
      continue;
    }
    // Double-quoted identifier
    if (sql[i] === '"') {
      let j = i + 1;
      while (j < sql.length) {
        if (sql[j] === '"') {
          if (sql[j + 1] === '"') { j += 2; continue; }
          j++; break;
        }
        j++;
      }
      result += " ".repeat(j - i);
      i = j;
      continue;
    }
    // Dollar-quoted body ($tag$...$tag$)
    if (sql[i] === "$") {
      let j = i + 1;
      while (j < sql.length && /[A-Za-z0-9_]/.test(sql[j])) j++;
      if (j > i + 1 || sql[j] === "$") {
        // Could be a dollar quote — check for closing $
        if (sql[j] === "$") {
          const tag = sql.slice(i, j + 1);
          let k = j + 1;
          while (k < sql.length) {
            if (sql.startsWith(tag, k)) {
              k += tag.length;
              break;
            }
            k++;
          }
          result += " ".repeat(k - i);
          i = k;
          continue;
        }
      }
      // Not a dollar quote — just a $ character
      result += sql[i];
      i++;
      continue;
    }
    // Line comment
    if (sql[i] === "-" && sql[i + 1] === "-") {
      let j = i;
      while (j < sql.length && sql[j] !== "\n") j++;
      result += " ".repeat(j - i);
      i = j;
      continue;
    }
    // Block comment
    if (sql[i] === "/" && sql[i + 1] === "*") {
      let j = i + 2;
      let depth = 1;
      while (j < sql.length && depth > 0) {
        if (sql[j] === "/" && sql[j + 1] === "*") { depth++; j += 2; }
        else if (sql[j] === "*" && sql[j + 1] === "/") { depth--; j += 2; }
        else j++;
      }
      result += " ".repeat(j - i);
      i = j;
      continue;
    }
    result += sql[i];
    i++;
  }
  return result;
}

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
 *   - String literals, dollar-quoted bodies, comments
 *     (no false positives on keywords inside non-code tokens)
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
  // Mask strings/comments/dollar-quotes BEFORE checking for mutation keywords.
  // This prevents false positives on:
  //   WITH x AS (SELECT 'DELETE FROM users') SELECT * FROM x
  //   WITH x AS (SELECT 1 /* DELETE FROM users */) SELECT * FROM x
  if (/^WITH\b/i.test(upper)) {
    const masked = maskNonCodeTokens(stripped);
    const maskedUpper = masked.toUpperCase();
    const body = maskedUpper.replace(/^WITH\b[\s\S]*?\bAS\b\s*\(/, "");
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
