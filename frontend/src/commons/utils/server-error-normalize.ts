import type { CommandErrorShape, ErrorCode, NormalizedError } from "./error-types";
import { isValidErrorCode } from "./error-types";

function isCommandErrorShape(err: unknown): err is CommandErrorShape {
  return (
    typeof err === "object" &&
    err !== null &&
    typeof (err as CommandErrorShape).error === "string" &&
    typeof (err as CommandErrorShape).message === "string" &&
    typeof (err as CommandErrorShape).message_id === "string"
  );
}

/** Map backend error codes (which may use DB_ prefix) to frontend ErrorCode. */
function resolveCode(raw: string): ErrorCode {
  if (isValidErrorCode(raw)) return raw;
  // Backend uses DB_ prefix for connection-level errors — strip it.
  if (raw.startsWith("DB_")) {
    const stripped = raw.slice(3);
    switch (stripped) {
      case "CONNECTION_REFUSED":
      case "CONNECTION_FAILED":
      case "CONNECTION_LOST":
      case "SSL_ERROR":
        return "CONNECTION_FAILED";
      case "CONNECTION_TIMEOUT":
        return "TIMEOUT";
      case "DATABASE_NOT_FOUND":
        return "NOT_FOUND";
      case "AUTH_FAILED":
        return "AUTH_FAILED";
      default:
        return "CONNECTION_FAILED";
    }
  }
  // Backend query/schema/data codes that need mapping.
  switch (raw) {
    case "SCHEMA_FAILED":
    case "OPERATION_UNSUPPORTED":
    case "DATA_FAILED":
      return "INTERNAL";
    case "VALIDATION_ERROR":
      return "VALIDATION";
    case "INTERNAL_ERROR":
      return "INTERNAL";
    default:
      return "UNKNOWN";
  }
}

export function normalizeServerError(error: unknown): NormalizedError {
  if (isCommandErrorShape(error)) {
    return {
      code: resolveCode(error.error),
      message: error.message,
      messageId: error.message_id,
      details: error.details,
      originalError: error,
    };
  }

  if (typeof error === "string") {
    return {
      code: "UNKNOWN",
      message: error,
      messageId: "error.unknown",
      originalError: error,
    };
  }

  if (error instanceof Error) {
    const message = error.message.toLowerCase();
    let code: ErrorCode = "UNKNOWN";
    let messageId = "error.unknown";

    if (message.includes("timeout") || message.includes("timed out")) {
      code = "TIMEOUT";
      messageId = "error.timeout";
    } else if (
      message.includes("network") ||
      message.includes("fetch") ||
      message.includes("connection")
    ) {
      code = "CONNECTION_FAILED";
      messageId = "error.connection.failed";
    }

    return {
      code,
      message: error.message,
      messageId,
      originalError: error,
    };
  }

  return {
    code: "UNKNOWN",
    message: String(error),
    messageId: "error.unknown",
    originalError: error,
  };
}
