import { z } from "zod";

import { AppError } from "./error-types";

export const connectionConfigSchema = z.object({
  name: z.string().min(1).max(100),
  host: z.string().min(1).max(255),
  port: z.number().int().min(1).max(65535),
  database: z.string().min(1).max(255),
  username: z.string().min(1).max(255),
  driver: z.enum(["postgres", "sqlite"]),
  sslMode: z.enum(["disable", "require", "verify-ca", "verify-full"]),
  sshTunnel: z
    .object({
      host: z.string(),
      port: z.number().int().min(1).max(65535),
      user: z.string(),
      privateKeyPath: z.string(),
    })
    .optional(),
  queryTimeoutMs: z.number().int().min(1000).max(300000).default(30000),
  maxRows: z.number().int().min(1).max(100000).default(500),
});

export const sqlQuerySchema = z.object({
  connectionId: z.string().uuid(),
  sql: z.string().min(1).max(1_000_000),
});

export function validateInput<T>(
  schema: z.ZodType<T>,
  data: unknown,
): T {
  const result = schema.safeParse(data);
  if (!result.success) {
    throw new AppError(
      "VALIDATION",
      "Validation failed",
      "error.validation",
      result.error.issues,
    );
  }
  return result.data;
}
