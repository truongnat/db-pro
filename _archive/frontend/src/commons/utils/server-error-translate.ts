import i18n from "@/commons/locales/i18n";

import type { NormalizedError, TranslatedError } from "./error-types";

export function translateError(error: NormalizedError): TranslatedError {
  const userMessage = i18n.exists(error.messageId)
    ? i18n.t(error.messageId)
    : i18n.t("error.unknown");

  return {
    code: error.code,
    userMessage,
    technicalMessage: error.message,
    messageId: error.messageId,
    details: error.details,
  };
}
