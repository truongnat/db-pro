import { describe, expect, it } from "vitest";
import { normalizeMutationError } from "../utils/mutation-error";

describe("normalizeMutationError", () => {
  it("extracts fields from TranslatedError", () => {
    const err = {
      code: "QUERY_FAILED",
      userMessage: "Localized message",
      technicalMessage: "ERROR 23505: duplicate key",
      messageId: "error.query.failed",
      details: { constraint_type: "unique", constraint: "uq_email", table: "users", column: null },
    };

    const result = normalizeMutationError(err);

    expect(result.technicalMessage).toBe("ERROR 23505: duplicate key");
    expect(result.userMessage).toBe("Localized message");
    expect(result.code).toBe("QUERY_FAILED");
    expect(result.details).toEqual({
      constraint_type: "unique",
      constraint: "uq_email",
      table: "users",
      column: null,
    });
  });

  it("extracts message from Error instance", () => {
    const err = new Error("something broke");
    const result = normalizeMutationError(err);

    expect(result.technicalMessage).toBe("something broke");
    expect(result.userMessage).toBe("something broke");
    expect(result.code).toBe("UNKNOWN");
    expect(result.details).toBeNull();
  });

  it("handles string errors", () => {
    const result = normalizeMutationError("plain string error");

    expect(result.technicalMessage).toBe("plain string error");
    expect(result.code).toBe("UNKNOWN");
    expect(result.details).toBeNull();
  });

  it("handles null", () => {
    const result = normalizeMutationError(null);

    expect(result.technicalMessage).toBe("Unknown error");
    expect(result.code).toBe("UNKNOWN");
    expect(result.details).toBeNull();
  });

  it("handles undefined", () => {
    const result = normalizeMutationError(undefined);

    expect(result.technicalMessage).toBe("Unknown error");
    expect(result.code).toBe("UNKNOWN");
  });

  it("handles TranslatedError without details", () => {
    const err = {
      code: "READ_ONLY_VIOLATION",
      userMessage: "Read only",
      technicalMessage: "Connection is read-only",
      messageId: "error.data.read_only",
    };

    const result = normalizeMutationError(err);

    expect(result.technicalMessage).toBe("Connection is read-only");
    expect(result.code).toBe("READ_ONLY_VIOLATION");
    expect(result.details).toBeNull();
  });
});
