import {
  ChevronDown,
  MoreHorizontal,
  Play,
  Square,
  FileText,
  Download,
  Upload,
  Trash2,
  Settings2,
  Save,
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

interface QueryToolbarProps {
  onExecuteCurrent: () => void;
  onExecuteAll: () => void;
  onCancel: () => void;
  onExplain: () => void;
  onClear: () => void;
  onExport: () => void;
  onFormat: () => void;
  onSaveQuery: () => void;
  onExportSql: () => void;
  onImportSql: () => void;
  onOpenRunConfig: () => void;
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
  onSaveQuery,
  onExportSql,
  onImportSql,
  onOpenRunConfig,
  isExecuting,
  isExplaining,
  hasConnection,
  hasSql,
}: QueryToolbarProps) {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-1.5 border-b border-[var(--app-border-subtle)] bg-background px-3 py-1.5">
      {/* ── Run dropdown ── */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            type="button"
            className="gap-1 rounded-sm px-3 py-1.5 text-sm font-medium"
            disabled={!hasConnection || !hasSql || isExecuting}
          >
            <Play className="h-3.5 w-3.5" />
            {t("query.run")}
            <ChevronDown className="h-3 w-3 opacity-60" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start">
          <DropdownMenuItem onClick={onExecuteCurrent}>
            <Play className="mr-2 h-3.5 w-3.5" />
            {t("query.runCurrent")}
            <span className="ml-auto text-[10px] text-[var(--app-text-muted)]">Ctrl+Enter</span>
          </DropdownMenuItem>
          <DropdownMenuItem onClick={onExecuteAll}>
            <Play className="mr-2 h-3.5 w-3.5" />
            {t("query.runAll")}
            <span className="ml-auto text-[10px] text-[var(--app-text-muted)]">Ctrl+Shift+Enter</span>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={onExplain} disabled={!hasSql || isExplaining}>
            <HelpCircle className="mr-2 h-3.5 w-3.5" />
            {t("query.explain")}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

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
          <DropdownMenuItem onClick={onSaveQuery} disabled={!hasSql}>
            <Save className="mr-2 h-3.5 w-3.5" />
            {t("query.save")}
          </DropdownMenuItem>
          <DropdownMenuSeparator />
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
          <DropdownMenuItem onClick={onOpenRunConfig}>
            <Settings2 className="mr-2 h-3.5 w-3.5" />
            {t("query.runConfiguration")}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
