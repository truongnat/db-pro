import { useTranslation as useI18next } from "react-i18next";

export function useTranslation() {
  const { t, i18n } = useI18next();
  return { t, i18n, currentLanguage: i18n.language };
}
