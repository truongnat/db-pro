import { describe, expect, it } from "vitest";

import { getSqlTemplates } from "../sql/templates";

describe("dialect-aware SQL templates (UX-R7.2c)", () => {
  it("keeps postgres CREATE TABLE with SERIAL and NOW()", () => {
    const templates = getSqlTemplates("postgres");
    const createTable = templates.find((t) => t.label === "CREATE TABLE");
    expect(createTable?.sql).toContain("SERIAL");
    expect(createTable?.sql).toContain("NOW()");
  });

  it("removes SERIAL and NOW() from sqlite CREATE TABLE", () => {
    const templates = getSqlTemplates("sqlite");
    const createTable = templates.find((t) => t.label === "CREATE TABLE");
    expect(createTable?.sql).not.toContain("SERIAL");
    expect(createTable?.sql).not.toContain("NOW()");
  });

  it("uses sqlite-native types in sqlite CREATE TABLE", () => {
    const templates = getSqlTemplates("sqlite");
    const createTable = templates.find((t) => t.label === "CREATE TABLE");
    expect(createTable?.sql).toContain("INTEGER PRIMARY KEY");
    expect(createTable?.sql).toContain("TEXT NOT NULL");
    expect(createTable?.sql).toContain("CURRENT_TIMESTAMP");
  });

  it("keeps the same template set for both dialects", () => {
    const postgres = getSqlTemplates("postgres");
    const sqlite = getSqlTemplates("sqlite");
    expect(postgres).toHaveLength(sqlite.length);
    expect(postgres.map((t) => t.label)).toEqual(sqlite.map((t) => t.label));
  });

  it("leaves shared templates untouched", () => {
    const select = getSqlTemplates("sqlite").find((t) => t.label === "SELECT");
    expect(select?.sql).toContain("LIMIT 100");
  });
});
