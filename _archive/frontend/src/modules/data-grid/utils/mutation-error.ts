import type { TranslatedError } from "@/commons/utils/error-types";
import type { ConstraintDetails } from "./constraint-errors";

export interface NormalizedMutationError {
  technicalMessage: string;
  userMessage: string;
  code: string;
  details: ConstraintDetails | null;
}

export function normalizeMutationError(err: unknown): NormalizedMutationError {
  if (err && typeof err === "object" && "technicalMessage" in err) {
    const te = err as TranslatedError;
    return {
      technicalMessage: te.technicalMessage,
      userMessage: te.userMessage,
      code: te.code,
      details: (te.details as ConstraintDetails) ?? null,
    };
  }
  if (err instanceof Error) {
    return {
      technicalMessage: err.message,
      userMessage: err.message,
      code: "UNKNOWN",
      details: null,
    };
  }
  const msg = err != null ? String(err) : "Unknown error";
  return { technicalMessage: msg, userMessage: msg, code: "UNKNOWN", details: null };
}
