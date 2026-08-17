import type { RootOptions } from "react-dom/client";

export const SANITIZED_REACT_ERROR_MESSAGE =
  "Unhandled React error captured at the application root";

/**
 * React root/error-boundary reporter that intentionally discards the raw error.
 * Render failures can carry query text, connection strings, provider payloads,
 * or other sensitive values, so only a fixed message may reach the console.
 */
export function reportSanitizedReactError(_error?: unknown): void {
  console.error(SANITIZED_REACT_ERROR_MESSAGE);
}

/**
 * Replaces React's default root error reporting with the sanitized reporter.
 * Keep this object shared by production bootstrap and runtime regression tests.
 */
export const SANITIZED_REACT_ROOT_OPTIONS = {
  onCaughtError: reportSanitizedReactError,
  onUncaughtError: reportSanitizedReactError,
  onRecoverableError: reportSanitizedReactError,
} satisfies RootOptions;
