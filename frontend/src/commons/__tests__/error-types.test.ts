import { describe, expect, it } from "vitest";
import { AppError, isValidErrorCode, type ErrorCode } from "../utils/error-types";

describe("error-types", () => {
  describe("isValidErrorCode", () => {
    it("returns true for valid error codes", () => {
      const validCodes: string[] = [
        "CONNECTION_FAILED",
        "QUERY_FAILED",
        "QUERY_CANCELLED",
        "NOT_FOUND",
        "AUTH_FAILED",
        "TIMEOUT",
        "IO_ERROR",
        "INTROSPECTION_FAILED",
        "ENCRYPTION_FAILED",
        "VALIDATION",
        "INTERNAL",
      ];
      for (const code of validCodes) {
        expect(isValidErrorCode(code), `${code} should be valid`).toBe(true);
      }
    });

    it("returns false for invalid error codes", () => {
      expect(isValidErrorCode("UNKNOWN_CODE")).toBe(false);
      expect(isValidErrorCode("")).toBe(false);
      expect(isValidErrorCode("connection_failed")).toBe(false);
      expect(isValidErrorCode("CONNECTION")).toBe(false);
    });

    it("returns false for UNKNOWN code (not in valid set)", () => {
      expect(isValidErrorCode("UNKNOWN")).toBe(false);
    });
  });

  describe("AppError", () => {
    it("creates error with all properties", () => {
      const err = new AppError("CONNECTION_FAILED", "Could not connect", "err.conn.failed", { host: "localhost" });
      expect(err).toBeInstanceOf(Error);
      expect(err).toBeInstanceOf(AppError);
      expect(err.name).toBe("AppError");
      expect(err.code).toBe("CONNECTION_FAILED");
      expect(err.message).toBe("Could not connect");
      expect(err.messageId).toBe("err.conn.failed");
      expect(err.details).toEqual({ host: "localhost" });
    });

    it("works without details", () => {
      const err = new AppError("TIMEOUT", "Query timed out", "err.timeout");
      expect(err.code).toBe("TIMEOUT");
      expect(err.details).toBeUndefined();
    });

    it("has proper Error prototype", () => {
      const err = new AppError("INTERNAL", "Something broke", "err.internal");
      expect(err instanceof Error).toBe(true);
      expect(err.stack).toBeDefined();
    });
  });
});
