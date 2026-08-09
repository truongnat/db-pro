import { describe, it, expect } from "vitest";
import {
  classifyColumnMutation,
  hasChanges,
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
