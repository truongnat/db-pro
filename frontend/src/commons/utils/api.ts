import { invoke } from "@tauri-apps/api/core";

import { normalizeServerError } from "./server-error-normalize";
import { translateError } from "./server-error-translate";
import type { TranslatedError } from "./error-types";

export async function apiInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (rawError) {
    const normalized = normalizeServerError(rawError);
    const translated = translateError(normalized);
    console.error(`[apiInvoke] ${command}:`, translated);
    throw translated as TranslatedError;
  }
}
