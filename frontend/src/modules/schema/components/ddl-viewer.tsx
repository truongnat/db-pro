import { useState, useCallback } from "react";
import Editor from "@monaco-editor/react";
import { Copy, Check, FileCode2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useResolvedTheme } from "@/commons/stores/theme.store";

interface DdlViewerProps {
  ddl: string | null;
  isLoading: boolean;
  error: string | null;
  onOpenInQuery?: () => void;
}

export function DdlViewer({ ddl, isLoading, error, onOpenInQuery }: DdlViewerProps) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);

  const resolvedTheme = useResolvedTheme();
  const monacoTheme = resolvedTheme === "dark" ? "vs-dark" : "vs";

  const handleCopy = useCallback(async () => {
    if (!ddl) return;
    await navigator.clipboard.writeText(ddl);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [ddl]);

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

  return (
    <div className="relative flex min-h-0 flex-1 flex-col overflow-hidden">
      {/* Toolbar */}
      <div className="flex items-center gap-1.5 px-2 py-1.5">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              type="button"
              variant="outline"
              size="sm"
              className="h-7 gap-1.5 rounded-[5px] text-[12px]"
              onClick={handleCopy}
            >
              {copied ? (
                <Check className="h-3 w-3 text-emerald-500" />
              ) : (
                <Copy className="h-3 w-3" />
              )}
              {copied ? t("schema.copied") : t("schema.copyDdl")}
            </Button>
          </TooltipTrigger>
          <TooltipContent>Copy DDL to clipboard</TooltipContent>
        </Tooltip>
        {onOpenInQuery && (
          <Tooltip>
            <TooltipTrigger asChild>
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
            </TooltipTrigger>
            <TooltipContent>Open DDL in query editor</TooltipContent>
          </Tooltip>
        )}
      </div>

      {/* Monaco read-only editor */}
      <div className="min-h-0 flex-1">
        <Editor
          height="100%"
          language="sql"
          theme={monacoTheme}
          value={ddl}
          options={{
            readOnly: true,
            domReadOnly: true,
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            fontSize: 13,
            lineNumbers: "on",
            renderLineHighlight: "none",
            wordWrap: "on",
            automaticLayout: true,
            padding: { top: 8 },
            scrollbar: {
              verticalScrollbarSize: 8,
              horizontalScrollbarSize: 8,
            },
          }}
        />
      </div>
    </div>
  );
}
