import { describe, expect, it, vi } from "vitest";

vi.mock("@/commons/locales/i18n", () => ({
  default: {
    exists: (key: string) => key !== "error.unknown",
    t: (key: string) => `translated:${key}`,
  },
}));

import { translateError } from "../utils/server-error-translate";
import type { NormalizedError } from "../utils/error-types";

describe("translateError", () => {
  it("translates a known message id", () => {
    const error: NormalizedError = {
      code: "TIMEOUT",
      message: "Query timed out",
      messageId: "error.timeout",
      originalError: new Error("Query timed out"),
    };
    const result = translateError(error);
    expect(result.code).toBe("TIMEOUT");
    expect(result.userMessage).toBe("translated:error.timeout");
    expect(result.technicalMessage).toBe("Query timed out");
    expect(result.messageId).toBe("error.timeout");
  });

  it("falls back to error.unknown for unknown message id", () => {
    const error: NormalizedError = {
      code: "UNKNOWN",
      message: "Something broke",
      messageId: "error.unknown",
      originalError: "Something broke",
    };
    const result = translateError(error);
    // i18n.exists returns false for "error.unknown", so it falls back to t("error.unknown")
    expect(result.userMessage).toBe("translated:error.unknown");
    expect(result.technicalMessage).toBe("Something broke");
  });

  it("preserves details when present", () => {
    const error: NormalizedError = {
      code: "QUERY_ERROR",
      message: "syntax error",
      messageId: "error.query",
      details: { line: 1, column: 5 },
      originalError: new Error("syntax error"),
    };
    const result = translateError(error);
    expect(result.details).toEqual({ line: 1, column: 5 });
  });

  it("passes through code and messageId unchanged", () => {
    const error: NormalizedError = {
      code: "AUTH_FAILED",
      message: "bad password",
      messageId: "error.auth.failed",
      originalError: new Error("bad password"),
    };
    const result = translateError(error);
    expect(result.code).toBe("AUTH_FAILED");
    expect(result.messageId).toBe("error.auth.failed");
  });
});
