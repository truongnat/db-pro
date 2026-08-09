import {
  ChevronDown,
  MoreHorizontal,
  Play,
  Square,
  FileText,
  Download,
  Upload,
  Trash2,
  AlignLeft,
  HelpCircle,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useTranslation } from "@/commons/locales/useTranslation";
import { formatShortcut } from "@/commons/utils/platform";

interface QueryToolbarProps {
  onExecuteCurrent: () => void;
  onExecuteAll: () => void;
  onCancel: () => void;
  onExplain: () => void;
  onClear: () => void;
  onExport: () => void;
  onFormat: () => void;
  onExportSql: () => void;
  onImportSql: () => void;
  isExecuting: boolean;
  isExplaining: boolean;
  hasConnection: boolean;
  hasSql: boolean;
}

export function QueryToolbar({
  onExecuteCurrent,
  onExecuteAll,
  onCancel,
  onExplain,
  onClear,
  onExport,
  onFormat,
  onExportSql,
  onImportSql,
  isExecuting,
  isExplaining,
  hasConnection,
  hasSql,
}: QueryToolbarProps) {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-1.5 border-b border-[var(--app-border-subtle)] bg-background px-3 py-1.5">
      {/* ── Run split button ── */}
      <div className="inline-flex">
        <Button
          type="button"
          className="gap-1 rounded-r-none border-r border-r-white/20 px-3 py-1.5 text-sm font-medium"
          disabled={!hasConnection || !hasSql || isExecuting}
          onClick={onExecuteCurrent}
        >
          <Play className="h-3.5 w-3.5" />
          {t("query.run")}
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              type="button"
              variant="default"
              size="icon"
              className="h-auto w-[28px] rounded-l-none px-0 py-1.5"
              aria-label="Run options"
              disabled={!hasConnection || !hasSql || isExecuting}
            >
              <ChevronDown className="h-3 w-3" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start" className="min-w-[220px]">
            <DropdownMenuItem onClick={onExecuteCurrent} className="h-[32px]">
              <Play className="mr-2 h-3.5 w-3.5" />
              {t("query.runCurrent")}
              <span className="ml-auto text-[11px] text-[var(--app-text-muted)]">{formatShortcut({ primary: true, key: "Enter" })}</span>
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onExecuteAll} className="h-[32px]">
              <Play className="mr-2 h-3.5 w-3.5" />
              {t("query.runAll")}
              <span className="ml-auto text-[11px] text-[var(--app-text-muted)]">{formatShortcut({ primary: true, shiftKey: true, key: "Enter" })}</span>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {/* ── Stop (visible only while running) ── */}
      {isExecuting && (
        <Button
          type="button"
          variant="destructive"
          className="gap-1 rounded-sm px-2.5 py-1.5 text-sm"
          onClick={onCancel}
        >
          <Square className="h-3 w-3" />
          {t("common.actions.cancel")}
        </Button>
      )}

      {/* ── Explain (always visible) ── */}
      <Button
        type="button"
        variant="ghost"
        className="gap-1 rounded-sm px-2.5 py-1.5 text-sm"
        onClick={onExplain}
        disabled={!hasConnection || !hasSql || isExplaining}
      >
        <HelpCircle className="h-3.5 w-3.5" />
        {t("query.explain")}
      </Button>

      {/* ── Format ── */}
      <Button
        type="button"
        variant="ghost"
        className="gap-1 rounded-sm px-2.5 py-1.5 text-sm"
        onClick={onFormat}
        disabled={!hasSql}
      >
        <AlignLeft className="h-3.5 w-3.5" />
        {t("query.format")}
      </Button>

      <div className="flex-1" />

      {/* ── More menu ── */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="h-7 w-7 rounded-sm"
          >
            <MoreHorizontal className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem onClick={onFormat} disabled={!hasSql}>
            <AlignLeft className="mr-2 h-3.5 w-3.5" />
            {t("query.format")}
          </DropdownMenuItem>
          <DropdownMenuItem onClick={onExportSql} disabled={!hasSql}>
            <FileText className="mr-2 h-3.5 w-3.5" />
            {t("query.exportSql")}
          </DropdownMenuItem>
          <DropdownMenuItem onClick={onImportSql}>
            <Upload className="mr-2 h-3.5 w-3.5" />
            {t("query.importSql")}
          </DropdownMenuItem>
          <DropdownMenuItem onClick={onExport} disabled={!hasSql}>
            <Download className="mr-2 h-3.5 w-3.5" />
            {t("query.exportResults")}
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={onClear}>
            <Trash2 className="mr-2 h-3.5 w-3.5" />
            {t("query.clear")}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
