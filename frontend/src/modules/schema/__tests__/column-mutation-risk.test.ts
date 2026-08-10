import { describe, it, expect } from "vitest";
import {
  classifyColumnMutation,
  hasChanges,
  validateDataType,
  type ColumnMutationDraft,
} from "../utils/column-mutation-risk";

function makeDraft(overrides: Partial<ColumnMutationDraft> = {}): ColumnMutationDraft {
  return {
    original: {
      name: "email",
      dataType: "varchar",
      nullable: true,
      defaultValue: null,
    },
    newName: "email",
    newDataType: "varchar",
    newNullable: true,
    newDefaultValue: null,
    ...overrides,
  };
}

describe("classifyColumnMutation", () => {
  // ── No-op ──────────────────────────────────────────────────────────

  it("returns no operations when nothing changed", () => {
    const result = classifyColumnMutation(makeDraft(), "public", "users");
    expect(result.operations).toHaveLength(0);
    expect(result.sql).toHaveLength(0);
    expect(result.risk.level).toBe("low");
    expect(result.risk.requiresConfirmation).toBe(false);
  });

  // ── Rename ─────────────────────────────────────────────────────────

  it("classifies rename as medium risk with dependency warning", () => {
    const draft = makeDraft({ newName: "email_address" });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.operations).toHaveLength(1);
    expect(result.operations[0]).toContain("Rename");
    expect(result.sql[0]).toContain("RENAME COLUMN");
    expect(result.sql[0]).toContain('"email_address"');
    expect(result.risk.level).toBe("medium");
    expect(result.risk.requiresConfirmation).toBe(true);
    expect(result.warnings.length).toBeGreaterThan(0);
    expect(result.warnings[0]).toContain("break views");
  });

  // ── Type changes ───────────────────────────────────────────────────

  it("classifies varchar→text as low risk (safe widening)", () => {
    const draft = makeDraft({
      original: { name: "email", dataType: "varchar", nullable: true, defaultValue: null },
      newDataType: "text",
    });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.operations).toHaveLength(1);
    expect(result.operations[0]).toContain("Change type");
    expect(result.risk.level).toBe("low");
    expect(result.risk.requiresConfirmation).toBe(false);
  });

  it("classifies text→integer as high risk", () => {
    const draft = makeDraft({
      original: { name: "age", dataType: "text", nullable: true, defaultValue: null },
      newDataType: "integer",
    });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.risk.level).toBe("high");
    expect(result.risk.requiresConfirmation).toBe(true);
    expect(result.warnings.some((w) => w.includes("may rewrite"))).toBe(true);
  });

  it("classifies integer→bigint as low risk (safe widening)", () => {
    const draft = makeDraft({
      original: { name: "email", dataType: "integer", nullable: false, defaultValue: null },
      newDataType: "bigint",
    });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.risk.level).toBe("low");
  });

  it("classifies integer→text as medium risk", () => {
    const draft = makeDraft({
      original: { name: "code", dataType: "integer", nullable: false, defaultValue: null },
      newDataType: "text",
    });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.risk.level).toBe("medium");
    expect(result.risk.requiresConfirmation).toBe(true);
  });

  it("classifies numeric→integer as high risk (precision loss)", () => {
    const draft = makeDraft({
      original: { name: "price", dataType: "numeric", nullable: false, defaultValue: null },
      newDataType: "integer",
    });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.risk.level).toBe("high");
  });

  it("classifies timestamp→date as medium risk (loses time)", () => {
    const draft = makeDraft({
      original: { name: "created_at", dataType: "timestamp", nullable: false, defaultValue: null },
      newDataType: "date",
    });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.risk.level).toBe("medium");
  });

  it("classifies date→timestamp as low risk (safe widening)", () => {
    const draft = makeDraft({
      original: { name: "email", dataType: "date", nullable: false, defaultValue: null },
      newDataType: "timestamp",
    });
    const result = classifyColumnMutation(draft, "public", "events");

    expect(result.risk.level).toBe("low");
  });

  // ── Nullable changes ───────────────────────────────────────────────

  it("classifies nullable→NOT NULL as medium risk (needs data validation)", () => {
    const draft = makeDraft({
      original: { name: "email", dataType: "varchar", nullable: true, defaultValue: null },
      newNullable: false,
    });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.operations[0]).toContain("NOT NULL");
    expect(result.risk.level).toBe("medium");
    expect(result.risk.requiresConfirmation).toBe(true);
    expect(result.warnings.some((w) => w.includes("NULL values"))).toBe(true);
  });

  it("classifies NOT NULL→nullable as low risk (relaxing constraint)", () => {
    const draft = makeDraft({
      original: { name: "email", dataType: "varchar", nullable: false, defaultValue: null },
      newNullable: true,
    });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.operations[0]).toContain("Allow NULL");
    expect(result.risk.level).toBe("low");
    expect(result.risk.requiresConfirmation).toBe(false);
  });

  // ── Default changes ────────────────────────────────────────────────

  it("classifies adding a default as low risk", () => {
    const draft = makeDraft({
      newDefaultValue: "'unknown@example.com'",
    });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.operations[0]).toContain("Set default");
    expect(result.risk.level).toBe("low");
    expect(result.risk.requiresConfirmation).toBe(false);
  });

  it("classifies removing a default as low risk", () => {
    const draft = makeDraft({
      original: { name: "email", dataType: "varchar", nullable: true, defaultValue: "'test'" },
      newDefaultValue: null,
    });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.operations[0]).toContain("Remove default");
    expect(result.risk.level).toBe("low");
  });

  // ── Combined mutations ─────────────────────────────────────────────

  it("takes worst risk when multiple operations combined", () => {
    const draft = makeDraft({
      newName: "email_addr", // medium (rename)
      newDataType: "text",   // low (varchar→text)
      newNullable: false,    // medium (nullable→NOT NULL)
    });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.operations).toHaveLength(3);
    expect(result.risk.level).toBe("medium");
    expect(result.risk.requiresConfirmation).toBe(true);
  });

  it("produces correct SQL for multiple operations", () => {
    const draft = makeDraft({
      newName: "email_addr",
      newNullable: false,
      newDefaultValue: "'none'",
    });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.sql).toHaveLength(3);
    expect(result.sql[0]).toContain("RENAME COLUMN");
    expect(result.sql[1]).toContain("SET NOT NULL");
    expect(result.sql[2]).toContain("SET DEFAULT");
  });

  // ── SQL quoting ────────────────────────────────────────────────────

  it("properly quotes identifiers with special characters", () => {
    const draft = makeDraft({
      original: { name: "my col", dataType: "varchar", nullable: true, defaultValue: null },
      newName: 'my "quoted" col',
    });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.sql[0]).toContain('"my ""quoted"" col"');
  });
});

describe("hasChanges", () => {
  it("returns false when nothing changed", () => {
    expect(hasChanges(makeDraft())).toBe(false);
  });

  it("returns true when name changed", () => {
    expect(hasChanges(makeDraft({ newName: "other" }))).toBe(true);
  });

  it("returns true when type changed", () => {
    expect(hasChanges(makeDraft({ newDataType: "text" }))).toBe(true);
  });

  it("returns true when nullable changed", () => {
    expect(hasChanges(makeDraft({ newNullable: false }))).toBe(true);
  });

  it("returns true when default changed", () => {
    expect(hasChanges(makeDraft({ newDefaultValue: "'x'" }))).toBe(true);
  });

  it("treats empty string default as null (no change)", () => {
    const draft = makeDraft({ newDefaultValue: "" });
    expect(hasChanges(draft)).toBe(false);
  });
});

describe("default value formatting", () => {
  function defaultSql(value: string) {
    const draft = makeDraft({ newDefaultValue: value });
    return classifyColumnMutation(draft, "public", "users").sql[0];
  }

  it("passes through already-quoted string literals", () => {
    const sql = defaultSql("'hello'");
    expect(sql).toContain("SET DEFAULT 'hello'");
  });

  it("passes through integer literals", () => {
    const sql = defaultSql("0");
    expect(sql).toContain("SET DEFAULT 0");
  });

  it("passes through decimal literals", () => {
    const sql = defaultSql("3.14");
    expect(sql).toContain("SET DEFAULT 3.14");
  });

  it("passes through negative numbers", () => {
    const sql = defaultSql("-1");
    expect(sql).toContain("SET DEFAULT -1");
  });

  it("passes through NULL keyword", () => {
    const sql = defaultSql("NULL");
    expect(sql).toContain("SET DEFAULT NULL");
  });

  it("passes through boolean TRUE", () => {
    const sql = defaultSql("true");
    expect(sql).toContain("SET DEFAULT true");
  });

  it("passes through boolean FALSE", () => {
    const sql = defaultSql("FALSE");
    expect(sql).toContain("SET DEFAULT FALSE");
  });

  it("passes through CURRENT_TIMESTAMP", () => {
    const sql = defaultSql("CURRENT_TIMESTAMP");
    expect(sql).toContain("SET DEFAULT CURRENT_TIMESTAMP");
  });

  it("passes through now() function call", () => {
    const sql = defaultSql("now()");
    expect(sql).toContain("SET DEFAULT now()");
  });

  it("passes through gen_random_uuid() function call", () => {
    const sql = defaultSql("gen_random_uuid()");
    expect(sql).toContain("SET DEFAULT gen_random_uuid()");
  });

  it("wraps bare strings as quoted literals", () => {
    const sql = defaultSql("hello");
    expect(sql).toContain("SET DEFAULT 'hello'");
  });

  it("escapes single quotes in bare strings", () => {
    const sql = defaultSql("it's");
    expect(sql).toContain("SET DEFAULT 'it''s'");
  });

  it("throws on expression with semicolon (injection guard)", () => {
    expect(() => defaultSql("now(); DROP TABLE users")).toThrow("statement separators");
  });

  it("throws on expression with SQL comment", () => {
    expect(() => defaultSql("now() -- comment")).toThrow("SQL comments");
  });

  it("throws on expression with DDL keyword", () => {
    expect(() => defaultSql("delete()")).toThrow("DDL/DML");
  });

  it("safely quotes bare DDL-like strings as literals", () => {
    const sql = defaultSql("DROP TABLE users");
    expect(sql).toContain("SET DEFAULT 'DROP TABLE users'");
  });

  it("validates concatenation expression through expression path, not literal bypass", () => {
    const sql = defaultSql("'a' || now() || 'b'");
    expect(sql).toContain("SET DEFAULT 'a' || now() || 'b'");
  });

  it("safely quotes input with semicolons when not a function expression", () => {
    const sql = defaultSql("'a'; DROP TABLE users -- '");
    const afterDefault = sql.split("SET DEFAULT ")[1];
    expect(afterDefault[0]).toBe("'");
  });
});

describe("validateDataType", () => {
  it("accepts simple types", () => {
    expect(validateDataType("integer")).toBe("integer");
    expect(validateDataType("text")).toBe("text");
    expect(validateDataType("uuid")).toBe("uuid");
    expect(validateDataType("boolean")).toBe("boolean");
    expect(validateDataType("date")).toBe("date");
    expect(validateDataType("jsonb")).toBe("jsonb");
  });

  it("accepts types with numeric parameters", () => {
    expect(validateDataType("varchar(255)")).toBe("varchar(255)");
    expect(validateDataType("numeric(10,2)")).toBe("numeric(10,2)");
    expect(validateDataType("char(1)")).toBe("char(1)");
    expect(validateDataType("decimal(5, 3)")).toBe("decimal(5, 3)");
  });

  it("accepts multi-word types", () => {
    expect(validateDataType("timestamp with time zone")).toBe("timestamp with time zone");
    expect(validateDataType("character varying")).toBe("character varying");
    expect(validateDataType("double precision")).toBe("double precision");
    expect(validateDataType("time without time zone")).toBe("time without time zone");
  });

  it("accepts array types", () => {
    expect(validateDataType("integer[]")).toBe("integer[]");
    expect(validateDataType("text[][]")).toBe("text[][]");
  });

  it("rejects empty input", () => {
    expect(() => validateDataType("")).toThrow("must not be empty");
    expect(() => validateDataType("  ")).toThrow("must not be empty");
  });

  it("rejects semicolons", () => {
    expect(() => validateDataType("integer; DROP TABLE users")).toThrow("statement separators");
  });

  it("rejects SQL comments", () => {
    expect(() => validateDataType("integer -- comment")).toThrow("SQL comments");
    expect(() => validateDataType("integer /* comment */")).toThrow("SQL comments");
  });

  it("rejects USING keyword", () => {
    expect(() => validateDataType("integer USING old_col::int")).toThrow("USING");
  });

  it("rejects ALTER keyword", () => {
    expect(() => validateDataType("integer ALTER TABLE x")).toThrow("ALTER");
  });

  it("rejects DROP keyword", () => {
    expect(() => validateDataType("integer; DROP TABLE x")).toThrow("statement separators");
  });

  it("rejects non-numeric parameters", () => {
    expect(() => validateDataType("varchar(foo)")).toThrow("parameters must be numeric");
    expect(() => validateDataType("varchar('255')")).toThrow("parameters must be numeric");
  });

  it("rejects special characters in type name", () => {
    expect(() => validateDataType("integer@evil")).toThrow("Invalid data type");
    expect(() => validateDataType("int=1")).toThrow("Invalid data type");
  });

  it("is used by classifyColumnMutation for type change SQL", () => {
    const draft = makeDraft({ newDataType: "integer; DROP TABLE users" });
    expect(() => classifyColumnMutation(draft, "public", "users")).toThrow("statement separators");
  });
});

describe("SQLite capability gating", () => {
  it("allows rename on SQLite", () => {
    const draft = makeDraft({ newName: "email_addr" });
    const result = classifyColumnMutation(draft, "public", "users", "sqlite");

    expect(result.operations).toHaveLength(1);
    expect(result.operations[0]).toContain("Rename");
    expect(result.sql[0]).toContain("RENAME COLUMN");
    expect(result.unsupported).toHaveLength(0);
  });

  it("blocks type change on SQLite", () => {
    const draft = makeDraft({
      original: { name: "email", dataType: "varchar", nullable: true, defaultValue: null },
      newDataType: "text",
    });
    const result = classifyColumnMutation(draft, "public", "users", "sqlite");

    expect(result.operations).toHaveLength(0);
    expect(result.sql).toHaveLength(0);
    expect(result.unsupported).toHaveLength(1);
    expect(result.unsupported[0]).toContain("not supported by SQLite");
    expect(result.unsupported[0]).toContain("Change type");
  });

  it("blocks nullable change on SQLite", () => {
    const draft = makeDraft({ newNullable: false });
    const result = classifyColumnMutation(draft, "public", "users", "sqlite");

    expect(result.operations).toHaveLength(0);
    expect(result.sql).toHaveLength(0);
    expect(result.unsupported).toHaveLength(1);
    expect(result.unsupported[0]).toContain("NOT NULL");
  });

  it("blocks default value change on SQLite", () => {
    const draft = makeDraft({ newDefaultValue: "'test'" });
    const result = classifyColumnMutation(draft, "public", "users", "sqlite");

    expect(result.operations).toHaveLength(0);
    expect(result.sql).toHaveLength(0);
    expect(result.unsupported).toHaveLength(1);
    expect(result.unsupported[0]).toContain("Set default");
  });

  it("blocks all unsupported ops while allowing rename in combined draft", () => {
    const draft = makeDraft({
      newName: "email_addr",
      newDataType: "text",
      newNullable: false,
      newDefaultValue: "'x'",
    });
    const result = classifyColumnMutation(draft, "public", "users", "sqlite");

    expect(result.operations).toHaveLength(1);
    expect(result.operations[0]).toContain("Rename");
    expect(result.unsupported).toHaveLength(3);
  });

  it("returns empty unsupported for postgres (default)", () => {
    const draft = makeDraft({ newDataType: "text" });
    const result = classifyColumnMutation(draft, "public", "users");

    expect(result.unsupported).toHaveLength(0);
    expect(result.operations).toHaveLength(1);
  });
});
