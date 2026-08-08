import { describe, expect, it } from "vitest";

/**
 * Regression tests for the PRODUCTION SQL safety classifier.
 *
 * These tests import the actual production implementation from
 * modules/query/sql/sql-risk-classifier.ts — NOT a copy.
 *
 * If the production classifier is broken, these tests WILL fail.
 */
import { classifySqlRisk, classifyScriptRisk } from "@/modules/query/sql/sql-risk-classifier";

describe("SQL Safety Classifier — classifySqlRisk (production)", () => {
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

    it("WITH string containing DELETE → read (no false positive)", () => {
      expect(classifySqlRisk("WITH x AS (SELECT 'DELETE FROM users') SELECT * FROM x")).toBe("read");
    });

    it("WITH block comment containing DELETE → read (no false positive)", () => {
      expect(classifySqlRisk("WITH x AS (\n  SELECT 1 /* DELETE FROM users */\n) SELECT * FROM x")).toBe("read");
    });

    it("WITH line comment containing DELETE → read (no false positive)", () => {
      expect(classifySqlRisk("WITH x AS (\n  SELECT 1 -- DELETE FROM users\n) SELECT * FROM x")).toBe("read");
    });

    it("WITH actual mutating CTE → destructive", () => {
      expect(classifySqlRisk("WITH x AS (DELETE FROM users RETURNING *) SELECT * FROM x")).toBe("destructive");
    });

    it("WITH dollar-quoted DELETE → read (no false positive)", () => {
      expect(classifySqlRisk("WITH x AS (SELECT $$DELETE FROM users$$) SELECT * FROM x")).toBe("read");
    });

    it("WITH double-quoted identifier containing DELETE → read", () => {
      expect(classifySqlRisk('WITH x AS (SELECT 1 AS "DELETE FROM users") SELECT * FROM x')).toBe("read");
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

    it("EXPLAIN (ANALYZE, VERBOSE) DELETE → destructive", () => {
      expect(classifySqlRisk("EXPLAIN (ANALYZE, VERBOSE) DELETE FROM users")).toBe("destructive");
    });

    it("EXPLAIN (ANALYZE) DROP → destructive", () => {
      expect(classifySqlRisk("EXPLAIN (ANALYZE) DROP TABLE users")).toBe("destructive");
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

describe("SQL Safety Classifier — classifyScriptRisk (production)", () => {
  it("SELECT 1; DROP TABLE users → destructive (max risk)", () => {
    expect(classifyScriptRisk("SELECT 1; DROP TABLE users;")).toBe("destructive");
  });

  it("SELECT 1; UPDATE users SET → write (max risk)", () => {
    expect(classifyScriptRisk("SELECT 1; UPDATE users SET name = 'x';")).toBe("write");
  });

  it("SELECT 1; SELECT 2 → read", () => {
    expect(classifyScriptRisk("SELECT 1; SELECT 2;")).toBe("read");
  });

  it("INSERT; DELETE → destructive", () => {
    expect(classifyScriptRisk("INSERT INTO t VALUES (1); DELETE FROM t;")).toBe("destructive");
  });

  it("single SELECT → read", () => {
    expect(classifyScriptRisk("SELECT * FROM users")).toBe("read");
  });

  it("single DROP → destructive", () => {
    expect(classifyScriptRisk("DROP TABLE users")).toBe("destructive");
  });

  it("empty script → read", () => {
    expect(classifyScriptRisk("")).toBe("read");
  });

  it("comments only → read", () => {
    expect(classifyScriptRisk("-- comment\n/* block */")).toBe("read");
  });

  it("mixed read and write → write", () => {
    expect(classifyScriptRisk("SELECT 1; INSERT INTO t VALUES (1);")).toBe("write");
  });

  it("all destructive → destructive", () => {
    expect(classifyScriptRisk("DROP TABLE a; DROP TABLE b;")).toBe("destructive");
  });

  it("WITH string literal DELETE in script → read", () => {
    expect(classifyScriptRisk("WITH x AS (SELECT 'DELETE FROM users') SELECT * FROM x")).toBe("read");
  });

  it("dollar-quoted DELETE in script → read", () => {
    expect(classifyScriptRisk("SELECT $$DELETE FROM users$$")).toBe("read");
  });
});
