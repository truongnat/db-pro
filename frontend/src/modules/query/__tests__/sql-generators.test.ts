import { describe, expect, it } from "vitest";

import { getSqlDialect } from "../sql/dialect";
import {
  generateCountSQL,
  generateDeleteSQL,
  generateInsertSQL,
  generateSelectSQL,
  generateUpdateSQL,
} from "../sql/generators";

const COLUMNS = [
  { name: "id", isPrimaryKey: true, defaultValue: "nextval('users_id_seq'::regclass)" },
  { name: "name", nullable: false },
  { name: "nickname", nullable: true },
];

describe("dialect-aware CRUD generators (UX-R7.2b)", () => {
  const postgres = getSqlDialect("postgres");
  const sqlite = getSqlDialect("sqlite");

  describe("generateSelectSQL", () => {
    it("qualifies schema on postgres", () => {
      expect(generateSelectSQL(postgres, "public", "users", COLUMNS)).toBe(
        `SELECT\n  "id",\n  "name",\n  "nickname"\nFROM "public"."users";`,
      );
    });

    it("drops schema on sqlite when null", () => {
      expect(generateSelectSQL(sqlite, null, "users", COLUMNS)).toBe(
        `SELECT\n  "id",\n  "name",\n  "nickname"\nFROM "users";`,
      );
    });
  });

  describe("generateInsertSQL", () => {
    it("uses DEFAULT for PK with default on postgres", () => {
      expect(generateInsertSQL(postgres, "public", "users", COLUMNS)).toBe(
        'INSERT INTO "public"."users" ("id", "name", "nickname")\nVALUES (DEFAULT, \'<name>\', NULL);',
      );
    });

    it("applies same quote rules on sqlite", () => {
      expect(generateInsertSQL(sqlite, "main", "users", COLUMNS)).toBe(
        'INSERT INTO "main"."users" ("id", "name", "nickname")\nVALUES (DEFAULT, \'<name>\', NULL);',
      );
    });
  });

  describe("generateUpdateSQL", () => {
    it("quotes pk and non-pk columns on postgres", () => {
      expect(generateUpdateSQL(postgres, "public", "users", COLUMNS)).toBe(
        'UPDATE "public"."users"\nSET\n  "name" = <name>,\n  "nickname" = <nickname>\nWHERE "id" = <id>;',
      );
    });
  });

  describe("generateDeleteSQL", () => {
    it("quotes pk columns on sqlite without schema", () => {
      expect(generateDeleteSQL(sqlite, null, "users", COLUMNS)).toBe(
        'DELETE FROM "users"\nWHERE "id" = <id>;',
      );
    });
  });

  describe("generateCountSQL", () => {
    it("generates SELECT COUNT(*) with schema on postgres", () => {
      expect(generateCountSQL(postgres, "public", "users")).toBe(
        'SELECT COUNT(*) AS cnt\nFROM "public"."users";',
      );
    });

    it("generates SELECT COUNT(*) without schema on sqlite", () => {
      expect(generateCountSQL(sqlite, null, "users")).toBe(
        'SELECT COUNT(*) AS cnt\nFROM "users";',
      );
    });

    it("qualifies schema on sqlite when provided", () => {
      expect(generateCountSQL(sqlite, "main", "orders")).toBe(
        'SELECT COUNT(*) AS cnt\nFROM "main"."orders";',
      );
    });
  });

  describe("quote rules integration with db-object flow", () => {
    it("escapes double quotes in column names on both dialects", () => {
      const weird = [{ name: 'weird"col', isPrimaryKey: false }];
      expect(generateSelectSQL(postgres, "public", "t", weird)).toBe(
        `SELECT\n  "weird""col"\nFROM "public"."t";`,
      );
      expect(generateSelectSQL(sqlite, null, "t", weird)).toBe(
        `SELECT\n  "weird""col"\nFROM "t";`,
      );
    });
  });
});
