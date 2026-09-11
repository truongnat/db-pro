import { describe, expect, it } from "vitest";

import { parseSqlContext } from "../services/sql-context-parser";

describe("parseSqlContext", () => {
  describe("keyword context", () => {
    it("returns keyword for empty input", () => {
      const ctx = parseSqlContext("", 0);
      expect(ctx.kind).toBe("keyword");
      expect(ctx.prefix).toBe("");
    });

    it("returns keyword at start of statement", () => {
      const ctx = parseSqlContext("SEL", 3);
      expect(ctx.kind).toBe("keyword");
      expect(ctx.prefix).toBe("SEL");
    });
  });

  describe("table context", () => {
    it("returns table after FROM", () => {
      const ctx = parseSqlContext("SELECT * FROM ", 15);
      expect(ctx.kind).toBe("table");
    });

    it("returns table after FROM with partial name", () => {
      const ctx = parseSqlContext("SELECT * FROM us", 17);
      expect(ctx.kind).toBe("table");
      expect(ctx.prefix).toBe("us");
    });

    it("returns table after JOIN", () => {
      const ctx = parseSqlContext("SELECT * FROM a JOIN ", 22);
      expect(ctx.kind).toBe("table");
    });

    it("returns table after LEFT JOIN", () => {
      const ctx = parseSqlContext("SELECT * FROM a LEFT JOIN ", 27);
      expect(ctx.kind).toBe("table");
    });

    it("returns table after INSERT INTO", () => {
      const ctx = parseSqlContext("INSERT INTO ", 12);
      expect(ctx.kind).toBe("table");
    });

    it("returns table after UPDATE", () => {
      const ctx = parseSqlContext("UPDATE ", 7);
      expect(ctx.kind).toBe("table");
    });
  });

  describe("column context", () => {
    it("returns column after SELECT", () => {
      const ctx = parseSqlContext("SELECT ", 7);
      expect(ctx.kind).toBe("column");
    });

    it("returns column after WHERE", () => {
      const ctx = parseSqlContext("SELECT * FROM users WHERE ", 25);
      expect(ctx.kind).toBe("column");
    });

    it("returns column after ORDER BY", () => {
      const ctx = parseSqlContext("SELECT * FROM users ORDER BY ", 30);
      expect(ctx.kind).toBe("column");
    });

    it("returns column after GROUP BY", () => {
      const ctx = parseSqlContext("SELECT * FROM users GROUP BY ", 30);
      expect(ctx.kind).toBe("column");
    });

    it("returns column after ON", () => {
      const ctx = parseSqlContext("SELECT * FROM a JOIN b ON ", 26);
      expect(ctx.kind).toBe("column");
    });
  });

  describe("qualified column context", () => {
    it("returns qualifiedColumn after alias dot", () => {
      const ctx = parseSqlContext("SELECT u.", 9);
      expect(ctx.kind).toBe("qualifiedColumn");
      expect(ctx.qualifier).toBe("u");
      expect(ctx.prefix).toBe("");
    });

    it("returns qualifiedColumn with partial name", () => {
      const ctx = parseSqlContext("SELECT u.na", 11);
      expect(ctx.kind).toBe("qualifiedColumn");
      expect(ctx.qualifier).toBe("u");
      expect(ctx.prefix).toBe("na");
    });

    it("returns qualifiedColumn after table name dot", () => {
      const ctx = parseSqlContext("SELECT users.", 13);
      expect(ctx.kind).toBe("qualifiedColumn");
      expect(ctx.qualifier).toBe("users");
    });
  });

  describe("table reference extraction", () => {
    it("extracts single table ref", () => {
      const ctx = parseSqlContext("SELECT * FROM users WHERE ", 25);
      expect(ctx.tableRefs).toHaveLength(1);
      expect(ctx.tableRefs[0]).toEqual({ alias: "users", schema: "public", table: "users" });
    });

    it("extracts table with alias", () => {
      const ctx = parseSqlContext("SELECT * FROM users u WHERE u.", 30);
      expect(ctx.tableRefs).toHaveLength(1);
      expect(ctx.tableRefs[0]).toEqual({ alias: "u", schema: "public", table: "users" });
    });

    it("extracts table with AS alias", () => {
      const ctx = parseSqlContext("SELECT * FROM users AS u WHERE ", 30);
      expect(ctx.tableRefs).toHaveLength(1);
      expect(ctx.tableRefs[0]).toEqual({ alias: "u", schema: "public", table: "users" });
    });

    it("extracts schema-qualified table", () => {
      const ctx = parseSqlContext("SELECT * FROM myschema.orders ", 30);
      expect(ctx.tableRefs).toHaveLength(1);
      expect(ctx.tableRefs[0]).toEqual({ alias: "orders", schema: "myschema", table: "orders" });
    });

    it("extracts multiple table refs from JOIN", () => {
      const ctx = parseSqlContext("SELECT * FROM users u JOIN orders o ON ", 40);
      expect(ctx.tableRefs).toHaveLength(2);
      expect(ctx.tableRefs[0]).toEqual({ alias: "u", schema: "public", table: "users" });
      expect(ctx.tableRefs[1]).toEqual({ alias: "o", schema: "public", table: "orders" });
    });
  });

  describe("string and comment handling", () => {
    it("ignores content inside single-quoted strings", () => {
      const ctx = parseSqlContext("SELECT * FROM users WHERE name = 'FROM fake' AND ", 50);
      expect(ctx.kind).toBe("column");
    });

    it("ignores line comments", () => {
      const ctx = parseSqlContext("SELECT * FROM users -- FROM fake\nWHERE ", 40);
      expect(ctx.kind).toBe("column");
    });

    it("ignores block comments", () => {
      const ctx = parseSqlContext("SELECT * FROM users /* FROM fake */ WHERE ", 43);
      expect(ctx.kind).toBe("column");
    });
  });

  describe("multi-statement handling", () => {
    it("analyzes only the current statement", () => {
      const ctx = parseSqlContext("SELECT 1; SELECT * FROM ", 23);
      expect(ctx.kind).toBe("table");
    });
  });
});
