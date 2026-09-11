import { describe, it, expect } from "vitest";
import { BUILT_IN_SNIPPETS } from "../types/snippet.types";

describe("Built-in diagnostic snippets", () => {
  const diagnosticTriggers = ["pgsize", "pgts", "pgsess", "pgidx", "pglock", "slsize", "slidx"];

  it("includes all diagnostic snippets", () => {
    for (const trigger of diagnosticTriggers) {
      const snippet = BUILT_IN_SNIPPETS.find((s) => s.trigger === trigger);
      expect(snippet, `Missing diagnostic snippet: ${trigger}`).toBeDefined();
      expect(snippet?.builtIn).toBe(true);
      expect(snippet?.body.length).toBeGreaterThan(10);
    }
  });

  it("PostgreSQL snippets reference pg_ system catalogs", () => {
    const pgSnippets = BUILT_IN_SNIPPETS.filter((s) => s.trigger.startsWith("pg"));
    expect(pgSnippets.length).toBeGreaterThanOrEqual(5);
    for (const s of pgSnippets) {
      expect(s.body).toMatch(/pg_|SELECT/i);
    }
  });

  it("SQLite snippets reference sqlite_master", () => {
    const slSnippets = BUILT_IN_SNIPPETS.filter((s) => s.trigger.startsWith("sl"));
    expect(slSnippets.length).toBeGreaterThanOrEqual(2);
    for (const s of slSnippets) {
      expect(s.body).toMatch(/sqlite_master|SELECT/i);
    }
  });
});
