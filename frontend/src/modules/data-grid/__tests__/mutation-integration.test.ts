import { describe, expect, it } from "vitest";
import { normalizeMutationError } from "../utils/mutation-error";
import { classifyConstraintError } from "../utils/constraint-errors";
import { isCellTypeEditable, normalizeColumnType } from "../utils/column-value-codec";

describe("mutation error pipeline", () => {
  it("TranslatedError from apiInvoke → normalizeMutationError → classifier", () => {
    const apiError = {
      code: "QUERY_FAILED",
      userMessage: "Query execution failed",
      technicalMessage: "ERROR 23505: duplicate key value violates unique constraint \"uq_email\"",
      messageId: "error.query.failed",
      details: {
        constraint_type: "unique" as const,
        constraint: "uq_email",
        table: "users",
        column: null,
      },
    };

    const normalized = normalizeMutationError(apiError);
    const classified = classifyConstraintError(normalized.technicalMessage, normalized.details);

    expect(classified.kind).toBe("unique-violation");
    expect(classified.userMessage).toContain("unique");
  });

  it("non-constraint TranslatedError preserves technicalMessage", () => {
    const readOnlyError = {
      code: "READ_ONLY_VIOLATION" as const,
      userMessage: "Connection is read-only",
      technicalMessage: "ERROR: cannot execute UPDATE in a read-only connection",
      messageId: "error.data.read_only",
    };

    const normalized = normalizeMutationError(readOnlyError);

    expect(normalized.technicalMessage).toContain("read-only");
    expect(normalized.code).toBe("READ_ONLY_VIOLATION");
    expect(normalized.details).toBeNull();

    const classified = classifyConstraintError(normalized.technicalMessage, normalized.details);
    expect(classified.userMessage).toContain("read-only");
  });

  it("makeConflictError shape is handled by normalizeMutationError", () => {
    const conflict = {
      code: "ROW_NOT_FOUND",
      userMessage: "Row was modified or deleted by another process",
      technicalMessage: "Row was modified or deleted by another process",
      messageId: "",
      details: undefined,
    };

    const normalized = normalizeMutationError(conflict);

    expect(normalized.technicalMessage).toBe("Row was modified or deleted by another process");
    expect(normalized.code).toBe("ROW_NOT_FOUND");
    expect(normalized.details).toBeNull();
  });

  it("FK constraint with structured details produces correct classification", () => {
    const fkError = {
      code: "QUERY_FAILED",
      userMessage: "Query execution failed",
      technicalMessage:
        "ERROR 23503: insert or update on table \"orders\" violates foreign key constraint \"fk_user\"",
      messageId: "error.query.failed",
      details: {
        constraint_type: "foreign_key" as const,
        constraint: "fk_user",
        table: "orders",
        column: null,
      },
    };

    const normalized = normalizeMutationError(fkError);
    const classified = classifyConstraintError(normalized.technicalMessage, normalized.details);

    expect(classified.kind).toBe("foreign-key-violation");
  });

  it("deadlock error is classified from technicalMessage", () => {
    const deadlockError = {
      code: "QUERY_FAILED",
      userMessage: "Query execution failed",
      technicalMessage: "ERROR 40P01: deadlock detected",
      messageId: "error.query.failed",
    };

    const normalized = normalizeMutationError(deadlockError);
    const classified = classifyConstraintError(normalized.technicalMessage, normalized.details);

    expect(classified.userMessage).toContain("deadlock");
  });
});

describe("non-editable column types", () => {
  const cases: Array<{ dataType: string; expectedEditable: boolean }> = [
    { dataType: "bytea", expectedEditable: false },
    { dataType: "blob", expectedEditable: false },
    { dataType: "numeric", expectedEditable: false },
    { dataType: "numeric(30,10)", expectedEditable: false },
    { dataType: "decimal", expectedEditable: false },
    { dataType: "decimal(38,18)", expectedEditable: false },
    { dataType: "integer", expectedEditable: true },
    { dataType: "text", expectedEditable: true },
    { dataType: "boolean", expectedEditable: true },
    { dataType: "uuid", expectedEditable: true },
    { dataType: "jsonb", expectedEditable: true },
    { dataType: "timestamp", expectedEditable: true },
    { dataType: "float8", expectedEditable: true },
  ];

  for (const { dataType, expectedEditable } of cases) {
    it(`${dataType}: editable=${expectedEditable}`, () => {
      const cellType = normalizeColumnType(dataType);
      expect(isCellTypeEditable(cellType)).toBe(expectedEditable);
    });
  }
});
