import { describe, expect, it } from "vitest";
import { normalizeServerError } from "../utils/server-error-normalize";

describe("normalizeServerError", () => {
  describe("command error shape", () => {
    it("extracts fields from a CommandErrorShape", () => {
      const err = {
        error: "CONNECTION_FAILED",
        message: "could not connect",
        message_id: "error.connection.failed",
        details: { host: "localhost" },
      };
      const result = normalizeServerError(err);
      expect(result.code).toBe("CONNECTION_FAILED");
      expect(result.message).toBe("could not connect");
      expect(result.messageId).toBe("error.connection.failed");
      expect(result.details).toEqual({ host: "localhost" });
      expect(result.originalError).toBe(err);
    });

    it("falls back to UNKNOWN for invalid error code", () => {
      const err = {
        error: "NOT_A_REAL_CODE",
        message: "something",
        message_id: "error.something",
      };
      const result = normalizeServerError(err);
      expect(result.code).toBe("UNKNOWN");
    });

    it("preserves valid error codes", () => {
      const validCodes = [
        "CONNECTION_FAILED",
        "QUERY_FAILED",
        "NOT_FOUND",
        "AUTH_FAILED",
        "TIMEOUT",
        "IO_ERROR",
        "INTROSPECTION_FAILED",
        "ENCRYPTION_FAILED",
        "VALIDATION",
        "INTERNAL",
      ] as const;

      for (const code of validCodes) {
        const err = { error: code, message: "msg", message_id: "id" };
        const result = normalizeServerError(err);
        expect(result.code).toBe(code);
      }
    });
  });

  describe("string errors", () => {
    it("wraps a plain string", () => {
      const result = normalizeServerError("something broke");
      expect(result.code).toBe("UNKNOWN");
      expect(result.message).toBe("something broke");
      expect(result.messageId).toBe("error.unknown");
      expect(result.originalError).toBe("something broke");
    });
  });

  describe("Error instances", () => {
    it("detects timeout from error message", () => {
      const err = new Error("Query timed out after 30s");
      const result = normalizeServerError(err);
      expect(result.code).toBe("TIMEOUT");
      expect(result.messageId).toBe("error.timeout");
      expect(result.message).toBe("Query timed out after 30s");
    });

    it("detects 'timeout' keyword", () => {
      const err = new Error("timeout exceeded");
      const result = normalizeServerError(err);
      expect(result.code).toBe("TIMEOUT");
    });

    it("detects connection/network errors", () => {
      const err = new Error("network failure");
      const result = normalizeServerError(err);
      expect(result.code).toBe("CONNECTION_FAILED");
      expect(result.messageId).toBe("error.connection.failed");
    });

    it("detects 'fetch' keyword as connection error", () => {
      const err = new Error("fetch failed");
      const result = normalizeServerError(err);
      expect(result.code).toBe("CONNECTION_FAILED");
    });

    it("detects 'connection' keyword as connection error", () => {
      const err = new Error("connection refused");
      const result = normalizeServerError(err);
      expect(result.code).toBe("CONNECTION_FAILED");
    });

    it("defaults to UNKNOWN for unrecognized Error", () => {
      const err = new Error("something weird");
      const result = normalizeServerError(err);
      expect(result.code).toBe("UNKNOWN");
      expect(result.messageId).toBe("error.unknown");
    });
  });

  describe("fallback for other types", () => {
    it("handles null", () => {
      const result = normalizeServerError(null);
      expect(result.code).toBe("UNKNOWN");
      expect(result.message).toBe("null");
    });

    it("handles undefined", () => {
      const result = normalizeServerError(undefined);
      expect(result.code).toBe("UNKNOWN");
      expect(result.message).toBe("undefined");
    });

    it("handles number", () => {
      const result = normalizeServerError(42);
      expect(result.code).toBe("UNKNOWN");
      expect(result.message).toBe("42");
    });

    it("handles object without command error shape", () => {
      const result = normalizeServerError({ foo: "bar" });
      expect(result.code).toBe("UNKNOWN");
      expect(result.message).toBe("[object Object]");
    });
  });
});
