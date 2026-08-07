import { describe, expect, it } from "vitest";

import {
  findStatementAt,
  resolveRunTarget,
  splitStatementsWithRanges,
} from "../services/statement-splitter";

describe("statement splitter (UX-R7.3a)", () => {
  describe("splitStatementsWithRanges", () => {
    it("splits on semicolons like the backend", () => {
      const stmts = splitStatementsWithRanges("SELECT 1; SELECT 2");
      expect(stmts.map((s) => s.sql)).toEqual(["SELECT 1", "SELECT 2"]);
    });

    it("drops trailing semicolon statement", () => {
      const stmts = splitStatementsWithRanges("SELECT 1; SELECT 2;");
      expect(stmts.map((s) => s.sql)).toEqual(["SELECT 1", "SELECT 2"]);
    });

    it("respects single-quoted strings", () => {
      const stmts = splitStatementsWithRanges("SELECT ';'; SELECT 2");
      expect(stmts.map((s) => s.sql)).toEqual(["SELECT ';'", "SELECT 2"]);
    });

    it("handles escaped single quotes", () => {
      const stmts = splitStatementsWithRanges("SELECT 'it''s;'; SELECT 2");
      expect(stmts.map((s) => s.sql)).toEqual(["SELECT 'it''s;'", "SELECT 2"]);
    });

    it("returns empty for blank input", () => {
      expect(splitStatementsWithRanges("  ")).toEqual([]);
    });

    it("respects double-quoted identifiers", () => {
      const stmts = splitStatementsWithRanges('SELECT "a;b"; SELECT 2');
      expect(stmts.map((s) => s.sql)).toEqual(['SELECT "a;b"', "SELECT 2"]);
    });

    it("ignores semicolons inside line comments", () => {
      const stmts = splitStatementsWithRanges("SELECT 1; -- note; comment\nSELECT 2");
      expect(stmts.map((s) => s.sql)).toEqual([
        "SELECT 1",
        "-- note; comment\nSELECT 2",
      ]);
    });

    it("ignores semicolons inside block comments", () => {
      const stmts = splitStatementsWithRanges("SELECT 1; /* a; b */ SELECT 2");
      expect(stmts.map((s) => s.sql)).toEqual(["SELECT 1", "/* a; b */ SELECT 2"]);
    });

    it("ignores semicolons inside dollar-quoted strings", () => {
      const stmts = splitStatementsWithRanges("SELECT $$a;b$$; SELECT 2");
      expect(stmts.map((s) => s.sql)).toEqual(["SELECT $$a;b$$", "SELECT 2"]);
    });

    it("reports raw offsets of trimmed statements", () => {
      const stmts = splitStatementsWithRanges("  SELECT 1; SELECT 2");
      expect(stmts[0]).toEqual({ start: 2, end: 10, sql: "SELECT 1" });
      expect(stmts[1]).toEqual({ start: 12, end: 20, sql: "SELECT 2" });
      const raw = "  SELECT 1; SELECT 2";
      expect(raw.slice(stmts[1].start, stmts[1].end)).toBe("SELECT 2");
    });

    it("keeps a final statement without semicolon", () => {
      const stmts = splitStatementsWithRanges("SELECT 1\nWHERE id = 2");
      expect(stmts).toEqual([{ start: 0, end: 21, sql: "SELECT 1\nWHERE id = 2" }]);
    });
  });

  describe("findStatementAt", () => {
    const sql = "SELECT 1;\nSELECT 2;";

    it("finds the statement containing the cursor", () => {
      expect(findStatementAt(sql, 12)?.sql).toBe("SELECT 2");
      expect(findStatementAt(sql, 3)?.sql).toBe("SELECT 1");
    });

    it("finds the statement at its boundaries", () => {
      expect(findStatementAt(sql, 8)?.sql).toBe("SELECT 1");
      expect(findStatementAt(sql, 10)?.sql).toBe("SELECT 2");
    });

    it("returns the previous statement when the cursor is in a gap", () => {
      expect(findStatementAt("SELECT 1;\n\nSELECT 2", 10)?.sql).toBe("SELECT 1");
    });

    it("returns the last statement when the cursor is after the end", () => {
      expect(findStatementAt(sql, 99)?.sql).toBe("SELECT 2");
    });

    it("returns the first statement when the cursor is before everything", () => {
      expect(findStatementAt("  SELECT 1;", 0)?.sql).toBe("SELECT 1");
    });

    it("returns undefined for empty sql", () => {
      expect(findStatementAt("", 0)).toBeUndefined();
    });
  });

  describe("resolveRunTarget", () => {
    const sql = "SELECT 1;\nSELECT 2;";

    it("prefers a non-empty selection over the statement under cursor", () => {
      expect(
        resolveRunTarget({ value: sql, selection: { start: 0, end: 8 }, cursorOffset: 12 }),
      ).toBe("SELECT 1");
    });

    it("runs the statement under cursor without a selection", () => {
      expect(resolveRunTarget({ value: sql, selection: null, cursorOffset: 12 })).toBe(
        "SELECT 2",
      );
    });

    it("trims whitespace-only selections and falls back to the statement", () => {
      expect(
        resolveRunTarget({ value: sql, selection: { start: 9, end: 10 }, cursorOffset: 12 }),
      ).toBe("SELECT 2");
    });

    it("returns undefined when the editor is blank", () => {
      expect(
        resolveRunTarget({ value: "  ", selection: null, cursorOffset: 1 }),
      ).toBeUndefined();
    });
  });
});
