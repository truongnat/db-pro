import { useState } from "react";
import { Copy, FileCode2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useTranslation } from "@/commons/locales/useTranslation";

interface DdlViewerProps {
  ddl: string | null;
  isLoading: boolean;
  error: string | null;
  onOpenInQuery?: () => void;
}

export function DdlViewer({ ddl, isLoading, error, onOpenInQuery }: DdlViewerProps) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);

  if (isLoading) {
    return (
      <div className="p-4 text-[13px] text-[var(--app-text-muted)]">
        {t("common.states.loading")}
      </div>
    );
  }

  if (error) {
    return <div className="p-4 text-[13px] text-destructive">{error}</div>;
  }

  if (!ddl) {
    return <div className="p-4 text-[13px] text-[var(--app-text-muted)]">{t("schema.noDdl")}</div>;
  }

  const handleCopy = async () => {
    await navigator.clipboard.writeText(ddl);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="relative flex-1 overflow-auto">
      <div className="absolute right-2 top-2 z-10 flex items-center gap-1.5">
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="h-7 gap-1.5 rounded-[5px] text-[12px]"
          onClick={handleCopy}
        >
          <Copy className="h-3 w-3" />
          {copied ? t("schema.copied") : t("schema.copyDdl")}
        </Button>
        {onOpenInQuery && (
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="h-7 gap-1.5 rounded-[5px] text-[12px]"
            onClick={onOpenInQuery}
          >
            <FileCode2 className="h-3 w-3" />
            {t("dbObject.contextHeader.openDdl")}
          </Button>
        )}
      </div>
      <pre className="overflow-auto p-4 pt-10 font-mono text-[13px] leading-relaxed text-foreground">
        <code>{ddl}</code>
      </pre>
    </div>
  );
}
