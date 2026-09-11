export type ErrorCode =
  | "CONNECTION_FAILED"
  | "QUERY_FAILED"
  | "QUERY_CANCELLED"
  | "QUERY_SYNTAX_ERROR"
  | "QUERY_TIMEOUT"
  | "QUERY_PERMISSION_DENIED"
  | "NOT_FOUND"
  | "AUTH_FAILED"
  | "TIMEOUT"
  | "IO_ERROR"
  | "INTROSPECTION_FAILED"
  | "ENCRYPTION_FAILED"
  | "VALIDATION"
  | "READ_ONLY_VIOLATION"
  | "INTERNAL"
  | "UNKNOWN";

const VALID_ERROR_CODES: ReadonlySet<string> = new Set<string>([
  "CONNECTION_FAILED",
  "QUERY_FAILED",
  "QUERY_CANCELLED",
  "QUERY_SYNTAX_ERROR",
  "QUERY_TIMEOUT",
  "QUERY_PERMISSION_DENIED",
  "NOT_FOUND",
  "AUTH_FAILED",
  "TIMEOUT",
  "IO_ERROR",
  "INTROSPECTION_FAILED",
  "ENCRYPTION_FAILED",
  "VALIDATION",
  "READ_ONLY_VIOLATION",
  "INTERNAL",
]);

export function isValidErrorCode(code: string): code is ErrorCode {
  return VALID_ERROR_CODES.has(code);
}

export interface CommandErrorShape {
  error: string;
  message: string;
  message_id: string;
  details?: unknown;
}

export interface NormalizedError {
  code: ErrorCode;
  message: string;
  messageId: string;
  details?: unknown;
  originalError?: unknown;
}

export interface TranslatedError {
  code: ErrorCode;
  userMessage: string;
  technicalMessage: string;
  messageId: string;
  details?: unknown;
}

export class AppError extends Error {
  public readonly code: ErrorCode;
  public readonly messageId: string;
  public readonly details?: unknown;

  constructor(code: ErrorCode, message: string, messageId: string, details?: unknown) {
    super(message);
    this.name = "AppError";
    this.code = code;
    this.messageId = messageId;
    this.details = details;
  }
}
