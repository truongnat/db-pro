import type { TranslatedError } from "./error-types";

export interface NormalizedAppError {
  userMessage: string;
  technicalMessage: string;
  code: string;
}

export function normalizeAppError(err: unknown): NormalizedAppError {
  if (err && typeof err === "object" && "technicalMessage" in err) {
    const te = err as TranslatedError;
    return {
      userMessage: te.userMessage,
      technicalMessage: te.technicalMessage,
      code: te.code,
    };
  }
  if (
    err &&
    typeof err === "object" &&
    "message" in err &&
    typeof (err as Record<string, unknown>).message === "string"
  ) {
    const msg = (err as { message: string }).message;
    return { userMessage: msg, technicalMessage: msg, code: "UNKNOWN" };
  }
  if (err instanceof Error) {
    return { userMessage: err.message, technicalMessage: err.message, code: "UNKNOWN" };
  }
  const msg = err != null ? String(err) : "Unknown error";
  return { userMessage: msg, technicalMessage: msg, code: "UNKNOWN" };
}
