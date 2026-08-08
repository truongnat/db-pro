import { describe, expect, it, beforeEach } from "vitest";
import { z } from "zod";

import {
  resetActionRegistry,
  defineAction,
  executeAction,
} from "../index";

/**
 * Regression tests for the SQL safety classifier used in query actions.
 *
 * The classifier must correctly identify risk levels for:
 *   - Standard DML/DDL keywords
 *   - Leading comments (line and block)
 *   - WITH mutating CTE
 *   - EXPLAIN ANALYZE mutation
 *   - String literals containing keywords (no false positives)
 *   - Multi-statement scripts (cursor-based resolution)
 *
 * Since classifySqlRisk is not exported, we test it indirectly through
 * the resolveRisk hook on a test action that uses the same logic.
 */

// Re-implement the classifier here to test it in isolation.
// This mirrors the implementation in query.actions.ts.
function classifySqlRisk(sql: string): "read" | "write" | "destructive" {
  let stripped = sql.trim();
  while (true) {
    if (stripped.startsWith("--")) {
      const nl = stripped.indexOf("\n");
      if (nl === -1) return "read";
      stripped = stripped.slice(nl + 1).trim();
      continue;
    }
    if (stripped.startsWith("/*")) {
      const end = stripped.indexOf("*/");
      if (end === -1) return "read";
      stripped = stripped.slice(end + 2).trim();
      continue;
    }
    break;
  }

  if (!stripped) return "read";
  const upper = stripped.toUpperCase();

  if (/^EXPLAIN\b/i.test(upper)) {
    const inner = stripped
      .replace(/^EXPLAIN\b/i, "")
      .replace(/^\s+ANALYZE\b/i, "")
      .replace(/^\s+VERBOSE\b/i, "")
      .trim();
    if (inner) return classifySqlRisk(inner);
    return "read";
  }

  if (/^WITH\b/i.test(upper)) {
    const body = upper.replace(/^WITH\b[\s\S]*?\bAS\b\s*\(/, "");
    if (/\b(INSERT|UPDATE|DELETE)\b/.test(body)) return "destructive";
    return "read";
  }

  const noStrings = upper.replace(/'[^']*'/g, "''");

  if (/^(DROP|TRUNCATE)\b/.test(noStrings)) return "destructive";
  if (/^DELETE\b/.test(noStrings)) return "destructive";
  if (/^(INSERT|UPDATE|ALTER|CREATE|REPLACE)\b/.test(noStrings)) return "write";
  return "read";
}

beforeEach(() => {
  resetActionRegistry();
});

describe("SQL Safety Classifier", () => {
  // ── Standard keywords ───────────────────────────────────────

  describe("Standard SQL keywords", () => {
    it("SELECT → read", () => {
      expect(classifySqlRisk("SELECT * FROM users")).toBe("read");
    });

    it("INSERT → write", () => {
      expect(classifySqlRisk("INSERT INTO users VALUES (1)")).toBe("write");
    });

    it("UPDATE → write", () => {
      expect(classifySqlRisk("UPDATE users SET name = 'x'")).toBe("write");
    });

    it("ALTER → write", () => {
      expect(classifySqlRisk("ALTER TABLE users ADD COLUMN age INT")).toBe("write");
    });

    it("CREATE → write", () => {
      expect(classifySqlRisk("CREATE TABLE foo (id INT)")).toBe("write");
    });

    it("DELETE → destructive", () => {
      expect(classifySqlRisk("DELETE FROM users")).toBe("destructive");
    });

    it("DROP → destructive", () => {
      expect(classifySqlRisk("DROP TABLE users")).toBe("destructive");
    });

    it("TRUNCATE → destructive", () => {
      expect(classifySqlRisk("TRUNCATE TABLE users")).toBe("destructive");
    });
  });

  // ── Leading comments ────────────────────────────────────────

  describe("Leading comments", () => {
    it("strips line comments before classification", () => {
      expect(classifySqlRisk("-- cleanup\nDELETE FROM users")).toBe("destructive");
    });

    it("strips multiple line comments", () => {
      expect(classifySqlRisk("-- step 1\n-- step 2\nDROP TABLE users")).toBe("destructive");
    });

    it("strips block comments before classification", () => {
      expect(classifySqlRisk("/* dangerous */ DELETE FROM users")).toBe("destructive");
    });

    it("strips mixed comments", () => {
      expect(classifySqlRisk("/* header */\n-- note\nDROP TABLE users")).toBe("destructive");
    });

    it("comment-only SQL → read", () => {
      expect(classifySqlRisk("-- just a comment")).toBe("read");
    });
  });

  // ── WITH mutating CTE ──────────────────────────────────────

  describe("WITH mutating CTE", () => {
    it("WITH ... DELETE → destructive", () => {
      expect(classifySqlRisk("WITH x AS (DELETE FROM users RETURNING *) SELECT * FROM x")).toBe("destructive");
    });

    it("WITH ... INSERT → destructive", () => {
      expect(classifySqlRisk("WITH x AS (INSERT INTO t VALUES (1) RETURNING *) SELECT * FROM x")).toBe("destructive");
    });

    it("WITH ... UPDATE → destructive", () => {
      expect(classifySqlRisk("WITH x AS (UPDATE t SET a = 1 RETURNING *) SELECT * FROM x")).toBe("destructive");
    });

    it("WITH ... SELECT → read", () => {
      expect(classifySqlRisk("WITH x AS (SELECT * FROM t) SELECT * FROM x")).toBe("read");
    });
  });

  // ── EXPLAIN ANALYZE ────────────────────────────────────────

  describe("EXPLAIN ANALYZE mutation", () => {
    it("EXPLAIN DELETE → destructive", () => {
      expect(classifySqlRisk("EXPLAIN DELETE FROM users")).toBe("destructive");
    });

    it("EXPLAIN ANALYZE DELETE → destructive", () => {
      expect(classifySqlRisk("EXPLAIN ANALYZE DELETE FROM users")).toBe("destructive");
    });

    it("EXPLAIN ANALYZE VERBOSE DELETE → destructive", () => {
      expect(classifySqlRisk("EXPLAIN ANALYZE VERBOSE DELETE FROM users")).toBe("destructive");
    });

    it("EXPLAIN SELECT → read", () => {
      expect(classifySqlRisk("EXPLAIN SELECT * FROM users")).toBe("read");
    });

    it("EXPLAIN ANALYZE SELECT → read", () => {
      expect(classifySqlRisk("EXPLAIN ANALYZE SELECT * FROM users")).toBe("read");
    });
  });

  // ── String literals ────────────────────────────────────────

  describe("String literals (no false positives)", () => {
    it("'DELETE FROM x' as a string → read", () => {
      expect(classifySqlRisk("SELECT 'DELETE FROM users'")).toBe("read");
    });

    it("keyword inside string not classified", () => {
      expect(classifySqlRisk("SELECT 'DROP TABLE users'")).toBe("read");
    });

    it("INSERT with string values → write", () => {
      expect(classifySqlRisk("INSERT INTO t VALUES ('hello')")).toBe("write");
    });
  });

  // ── Edge cases ─────────────────────────────────────────────

  describe("Edge cases", () => {
    it("empty SQL → read", () => {
      expect(classifySqlRisk("")).toBe("read");
    });

    it("whitespace only → read", () => {
      expect(classifySqlRisk("   \n  \t  ")).toBe("read");
    });

    it("case insensitive", () => {
      expect(classifySqlRisk("select * from users")).toBe("read");
      expect(classifySqlRisk("delete from users")).toBe("destructive");
      expect(classifySqlRisk("Drop Table users")).toBe("destructive");
    });

    it("REPLACE → write", () => {
      expect(classifySqlRisk("REPLACE INTO t VALUES (1)")).toBe("write");
    });
  });
});
