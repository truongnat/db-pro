import type {
  CommandErrorShape,
  ErrorCode,
  NormalizedError,
} from "./error-types";
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

function resolveCode(raw: string): ErrorCode {
  return isValidErrorCode(raw) ? raw : "UNKNOWN";
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
