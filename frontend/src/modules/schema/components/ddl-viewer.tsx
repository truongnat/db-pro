import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

interface DdlViewerProps {
  ddl: string | null;
  isLoading: boolean;
  error: string | null;
}

export function DdlViewer({ ddl, isLoading, error }: DdlViewerProps) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);

  if (isLoading) {
    return (
      <div
        className="p-4 text-sm"
        style={{ color: "var(--color-text-secondary)" }}
      >
        {t("common.states.loading")}
      </div>
    );
  }

  if (error) {
    return (
      <div
        className="p-4 text-sm"
        style={{ color: "var(--color-error, #ef4444)" }}
      >
        {error}
      </div>
    );
  }

  if (!ddl) {
    return (
      <div
        className="p-4 text-sm"
        style={{ color: "var(--color-text-secondary)" }}
      >
        {t("schema.noDdl")}
      </div>
    );
  }

  const handleCopy = async () => {
    await navigator.clipboard.writeText(ddl);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="relative flex-1 overflow-auto">
      <button
        className="absolute right-2 top-2 rounded-[var(--radius-sm)] border px-2 py-1 text-xs transition-colors hover:bg-[var(--color-bg)]"
        style={{
          borderColor: "var(--color-border)",
          color: "var(--color-text-secondary)",
        }}
        onClick={handleCopy}
        type="button"
      >
        {copied ? t("schema.copied") : t("schema.copyDdl")}
      </button>
      <pre
        className="overflow-auto p-4 font-mono text-xs leading-relaxed"
        style={{ color: "var(--color-text)" }}
      >
        <code>{ddl}</code>
      </pre>
    </div>
  );
}
