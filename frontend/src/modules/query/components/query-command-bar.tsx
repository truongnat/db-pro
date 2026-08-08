import {
  ChevronDown,
  MoreHorizontal,
  Play,
  Square,
  FileText,
  Download,
  Upload,
  Trash2,
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
    <div className="flex h-[38px] items-center gap-2 border-b border-[var(--app-border-subtle)] bg-[var(--app-surface-2)] px-3">
      {/* Left — compact context breadcrumb */}
      <div className="flex items-center gap-1 text-[13px]">
        <Database className="h-3.5 w-3.5 shrink-0 text-[var(--app-text-muted)]" aria-hidden />
        <Select value={connectionId ?? ""} onValueChange={handleConnectionChange}>
          <SelectTrigger className="h-6 w-auto max-w-[160px] rounded border-none bg-transparent px-1.5 py-0 text-[13px] font-medium shadow-none hover:bg-[var(--app-hover)]">
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
          <>
            <span className="text-[var(--app-text-dim)]">/</span>
            <span className="truncate text-[13px] text-[var(--app-text-muted)]">
              {database}
            </span>
          </>
        )}
        {schemas.length > 0 ? (
          <>
            {database && <span className="text-[var(--app-text-dim)]">/</span>}
            <Select
              value={context.schema ?? DEFAULT_SCHEMA}
              onValueChange={(value) =>
                setQueryTabSchema(tabId, value === DEFAULT_SCHEMA ? null : value)
              }
            >
              <SelectTrigger className="h-6 w-auto max-w-[140px] rounded border-none bg-transparent px-1.5 py-0 text-[13px] text-[var(--app-text-muted)] shadow-none hover:bg-[var(--app-hover)]">
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
          </>
        ) : context.schema ? (
          <>
            <span className="text-[var(--app-text-dim)]">/</span>
            <span className="truncate text-[13px] text-[var(--app-text-muted)]">
              {context.schema}
            </span>
          </>
        ) : null}
      </div>

      <div className="flex-1" />

      {/* Right — actions */}
      <div className="flex items-center gap-1.5">
        {/* Run — primary action */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              type="button"
              className="h-[30px] gap-1.5 rounded-[5px] px-3 text-[13px] font-medium"
              disabled={!hasConnection || !hasSql || isExecuting}
            >
              <Play className="h-3.5 w-3.5" />
              {t("query.run")}
              <ChevronDown className="h-3 w-3 opacity-60" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="min-w-[220px]">
            <DropdownMenuItem onClick={onExecuteCurrent} className="h-[32px]">
              <Play className="mr-2 h-3.5 w-3.5" />
              {t("query.runCurrent")}
              <span className="ml-auto text-[11px] text-[var(--app-text-muted)]">Ctrl+Enter</span>
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onExecuteAll} className="h-[32px]">
              <Play className="mr-2 h-3.5 w-3.5" />
              {t("query.runAll")}
              <span className="ml-auto text-[11px] text-[var(--app-text-muted)]">Ctrl+Shift+Enter</span>
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem onClick={onExplain} disabled={!hasSql || isExplaining} className="h-[32px]">
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
            className="h-[30px] gap-1 rounded-[5px] px-2 text-[13px]"
            onClick={onCancel}
          >
            <Square className="h-3 w-3" />
          </Button>
        )}

        {/* Explain */}
        <Button
          type="button"
          variant="ghost"
          className="h-[28px] gap-1 rounded-[5px] px-2 text-[13px] text-[var(--app-text-muted)]"
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
          className="h-[28px] gap-1 rounded-[5px] px-2 text-[13px] text-[var(--app-text-muted)]"
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
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  );
}
