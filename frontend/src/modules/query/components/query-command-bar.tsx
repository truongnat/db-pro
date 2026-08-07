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
  Database,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useTranslation } from "@/commons/locales/useTranslation";
import type { QueryContext } from "@/commons/types/workspace.types";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";
import { useSchemaCatalogStore } from "../stores/schema-catalog.store";

import { setQueryTabConnection, setQueryTabSchema } from "../controllers/query-workspace.controller";

const DEFAULT_SCHEMA = "__default__";

interface QueryCommandBarProps {
  tabId: string;
  connectionId: string | null;
  context: QueryContext;
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

export function QueryCommandBar({
  tabId,
  connectionId,
  context,
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
}: QueryCommandBarProps) {
  const { t } = useTranslation();
  const { data: connections } = useConnectionList();
  const catalogs = useSchemaCatalogStore((s) => s.catalogs);

  const connection = connections?.find((c) => c.id === connectionId);
  const database = context.database ?? connection?.database ?? null;
  const schemas = catalogs.get(connectionId ?? "")?.schemas ?? [];

  const handleConnectionChange = (nextId: string) => {
    if (!nextId) return;
    const next = connections?.find((c) => c.id === nextId);
    if (!next || next.id === connectionId) return;
    setQueryTabConnection(tabId, next.id, { database: next.database, schema: null });
  };

  return (
    <div className="flex items-center gap-2 border-b border-[var(--app-border-subtle)] bg-background px-3 py-1.5">
      {/* Left — connection/schema context */}
      <div className="flex items-center gap-1.5">
        <Database className="h-3.5 w-3.5 shrink-0 text-[var(--app-text-dim)]" aria-hidden />
        <Select value={connectionId ?? ""} onValueChange={handleConnectionChange}>
          <SelectTrigger className="h-6 w-auto max-w-[160px] rounded border border-[var(--app-border)] bg-transparent px-2 py-0 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {connections?.map((conn) => (
              <SelectItem key={conn.id} value={conn.id}>
                {conn.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {database && (
          <span className="truncate text-[11px] text-[var(--app-text-dim)]">
            {database}
            {context.schema ? `.${context.schema}` : ""}
          </span>
        )}
        {schemas.length > 0 && (
          <Select
            value={context.schema ?? DEFAULT_SCHEMA}
            onValueChange={(value) =>
              setQueryTabSchema(tabId, value === DEFAULT_SCHEMA ? null : value)
            }
          >
            <SelectTrigger className="h-6 w-auto max-w-[140px] rounded border border-[var(--app-border)] bg-transparent px-2 py-0 text-xs">
              <SelectValue placeholder={t("query.contextDefaultSchema")} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={DEFAULT_SCHEMA}>{t("query.contextDefaultSchema")}</SelectItem>
              {schemas.map((schema) => (
                <SelectItem key={schema.name} value={schema.name}>
                  {schema.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </div>

      <div className="flex-1" />

      {/* Right — actions */}
      <div className="flex items-center gap-1">
        {/* Run dropdown — primary action */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              type="button"
              className="gap-1 rounded-md px-3 py-1 text-xs font-medium"
              disabled={!hasConnection || !hasSql || isExecuting}
            >
              <Play className="h-3.5 w-3.5" />
              {t("query.run")}
              <ChevronDown className="h-3 w-3 opacity-60" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
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

        {/* Stop — visible only while running */}
        {isExecuting && (
          <Button
            type="button"
            variant="destructive"
            className="gap-1 rounded-md px-2 py-1 text-xs"
            onClick={onCancel}
          >
            <Square className="h-3 w-3" />
          </Button>
        )}

        {/* Explain */}
        <Button
          type="button"
          variant="ghost"
          className="gap-1 rounded-md px-2 py-1 text-xs text-[var(--app-text-muted)]"
          onClick={onExplain}
          disabled={!hasConnection || !hasSql || isExplaining}
        >
          <HelpCircle className="h-3.5 w-3.5" />
          {t("query.explain")}
        </Button>

        {/* Format */}
        <Button
          type="button"
          variant="ghost"
          className="gap-1 rounded-md px-2 py-1 text-xs text-[var(--app-text-muted)]"
          onClick={onFormat}
          disabled={!hasSql}
        >
          <AlignLeft className="h-3.5 w-3.5" />
          {t("query.format")}
        </Button>

        {/* More menu */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-7 w-7 rounded-md text-[var(--app-text-muted)]"
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
    </div>
  );
}
