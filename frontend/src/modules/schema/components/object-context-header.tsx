import {
  Braces,
  ChevronDown,
  Columns3,
  Copy,
  FileCode2,
  ListOrdered,
  MoreHorizontal,
  RefreshCw,
  SquareFunction,
  Table2,
  Wand2,
} from "lucide-react";
import { useMemo } from "react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";
import { getDialectForConnection } from "@/modules/query/sql/dialect";
import {
  generateSelectSQL,
  generateCountSQL,
  generateInsertSQL,
  generateUpdateSQL,
  generateDeleteSQL,
} from "@/modules/query/sql/generators";

const OBJECT_ICONS: Record<string, typeof Table2> = {
  table: Table2,
  view: Columns3,
  function: SquareFunction,
  sequence: ListOrdered,
  type: Braces,
};

interface ObjectContextHeaderProps {
  connectionId: string | null;
  schema: string;
  objectName: string;
  objectType: string;
  columns?: { name: string; isPrimaryKey?: boolean; nullable?: boolean; defaultValue?: string | null }[];
  onRefresh: () => void;
  onOpenSelect: () => void;
  onOpenDdl: () => void;
  onOpenDdlEditor: () => void;
  onGenerateSql: () => void;
  onOpenQuery?: (sql: string, title: string) => void;
}

export function ObjectContextHeader({
  connectionId,
  schema,
  objectName,
  objectType,
  columns,
  onRefresh,
  onOpenSelect,
  onOpenDdl,
  onOpenDdlEditor,
  onGenerateSql,
  onOpenQuery,
}: ObjectContextHeaderProps) {
  const { t } = useTranslation();
  const { data: connections } = useConnectionList();
  const Icon = OBJECT_ICONS[objectType] ?? Table2;

  const connectionName = useMemo(() => {
    const conn = connections?.find((c) => c.id === connectionId);
    return conn?.name ?? connectionId ?? "";
  }, [connections, connectionId]);

  const dialect = useMemo(() => getDialectForConnection(connectionId), [connectionId]);
  const isTableOrView = objectType === "table" || objectType === "view";

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text).catch(() => {});
  };

  const qualifiedName = `${schema}.${objectName}`;

  const handleOpenSqlTemplate = (kind: "select" | "count" | "insert" | "update" | "delete") => {
    if (!onOpenQuery || !columns) return;
    const sqlColumns = columns.map((c) => ({
      name: c.name,
      isPrimaryKey: c.isPrimaryKey,
      nullable: c.nullable,
      defaultValue: c.defaultValue,
    }));

    let sql: string;
    let title: string;
    switch (kind) {
      case "select":
        sql = generateSelectSQL(dialect, schema, objectName, sqlColumns);
        title = `SELECT ${objectName}`;
        break;
      case "count":
        sql = generateCountSQL(dialect, schema, objectName);
        title = `COUNT ${objectName}`;
        break;
      case "insert":
        sql = generateInsertSQL(dialect, schema, objectName, sqlColumns);
        title = `INSERT ${objectName}`;
        break;
      case "update":
        sql = generateUpdateSQL(dialect, schema, objectName, sqlColumns);
        title = `UPDATE ${objectName}`;
        break;
      case "delete":
        sql = generateDeleteSQL(dialect, schema, objectName, sqlColumns);
        title = `DELETE ${objectName}`;
        break;
    }
    onOpenQuery(sql, title);
  };

  return (
    <div className="flex h-[46px] shrink-0 items-center gap-3 border-b border-[var(--app-border-subtle)] bg-[var(--app-surface-2)] px-3">
      <Icon className="h-4 w-4 shrink-0 text-primary" />

      <div className="min-w-0 flex-1">
        <div className="flex items-baseline gap-2">
          <span className="truncate text-[13px] font-semibold text-foreground">{objectName}</span>
          <span className="shrink-0 text-[11px] text-[var(--app-text-dim)]">
            {schema} · {objectType} · {connectionName}
          </span>
        </div>
      </div>

      <div className="flex shrink-0 items-center gap-1">
        {/* Primary: Refresh (only instance) */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              onClick={onRefresh}
              aria-label={t("dbObject.contextHeader.refresh")}
            >
              <RefreshCw className="h-3.5 w-3.5" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>{t("dbObject.contextHeader.refresh")}</TooltipContent>
        </Tooltip>

        {/* Secondary: Open Query dropdown */}
        {isTableOrView && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="gap-1"
                data-icon="inline-end"
              >
                {t("dbObject.contextHeader.sqlMenu")}
                <ChevronDown className="h-3 w-3" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => handleOpenSqlTemplate("select")}>
                <Table2 className="size-3.5" />
                SELECT *
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleOpenSqlTemplate("count")}>
                <ListOrdered className="size-3.5" />
                COUNT(*)
              </DropdownMenuItem>
              {columns && (
                <>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem onClick={() => handleOpenSqlTemplate("insert")}>
                    <Wand2 className="size-3.5" />
                    INSERT template
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => handleOpenSqlTemplate("update")}>
                    <Wand2 className="size-3.5" />
                    UPDATE template
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => handleOpenSqlTemplate("delete")}>
                    <Wand2 className="size-3.5" />
                    DELETE template
                  </DropdownMenuItem>
                </>
              )}
            </DropdownMenuContent>
          </DropdownMenu>
        )}

        {/* Fallback for non-table/view objects: keep the simple SQL menu */}
        {!isTableOrView && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="gap-1"
                data-icon="inline-end"
              >
                {t("dbObject.contextHeader.sqlMenu")}
                <ChevronDown className="h-3 w-3" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={onOpenDdl}>
                <FileCode2 />
                {t("dbObject.contextHeader.openDdl")}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={onOpenSelect}>
                <Wand2 />
                {t("dbObject.contextHeader.openSelect")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        )}

        {/* Overflow menu */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              aria-label={t("dbObject.contextHeader.more")}
            >
              <MoreHorizontal className="h-3.5 w-3.5" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => copyToClipboard(objectName)}>
              <Copy className="size-3.5" />
              Copy table name
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => copyToClipboard(qualifiedName)}>
              <Copy className="size-3.5" />
              Copy qualified name
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem onClick={onOpenDdl}>
              <FileCode2 className="size-3.5" />
              {t("dbObject.contextHeader.openDdl")}
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onOpenDdlEditor}>
              <FileCode2 className="size-3.5" />
              {t("dbObject.contextHeader.ddlEditor")}
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onGenerateSql}>
              <Wand2 className="size-3.5" />
              {t("dbObject.contextHeader.generateSql")}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  );
}
