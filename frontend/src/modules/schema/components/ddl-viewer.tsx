import { useState } from "react";

import { Button } from "@/components/ui/button";
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
      <div className="p-4 text-sm text-muted-foreground">
        {t("common.states.loading")}
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 text-sm text-destructive">
        {error}
      </div>
    );
  }

  if (!ddl) {
    return (
      <div className="p-4 text-sm text-muted-foreground">
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
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="absolute right-2 top-2"
        onClick={handleCopy}
      >
        {copied ? t("schema.copied") : t("schema.copyDdl")}
      </Button>
      <pre className="overflow-auto p-4 font-mono text-xs leading-relaxed text-foreground">
        <code>{ddl}</code>
      </pre>
    </div>
  );
}
