import {
  Braces,
  ChevronDown,
  Columns3,
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
  onRefresh: () => void;
  onOpenSelect: () => void;
  onOpenDdl: () => void;
  onOpenDdlEditor: () => void;
  onGenerateSql: () => void;
}

export function ObjectContextHeader({
  connectionId,
  schema,
  objectName,
  objectType,
  onRefresh,
  onOpenSelect,
  onOpenDdl,
  onOpenDdlEditor,
  onGenerateSql,
}: ObjectContextHeaderProps) {
  const { t } = useTranslation();
  const { data: connections } = useConnectionList();
  const Icon = OBJECT_ICONS[objectType] ?? Table2;

  const connectionName = useMemo(() => {
    const conn = connections?.find((c) => c.id === connectionId);
    return conn?.name ?? connectionId ?? "";
  }, [connections, connectionId]);

  return (
    <div className="flex h-[46px] shrink-0 items-center gap-3 border-b border-[var(--app-border-subtle)] bg-[var(--app-surface-2)] px-3">
      <Icon className="h-4 w-4 shrink-0 text-primary" />

      <div className="min-w-0 flex-1">
        <div className="flex items-baseline gap-2">
          <span className="truncate text-[13px] font-semibold text-foreground">
            {objectName}
          </span>
          <span className="shrink-0 text-[11px] text-[var(--app-text-dim)]">
            {schema} · {objectType} · {connectionName}
          </span>
        </div>
      </div>

      <div className="flex shrink-0 items-center gap-1">
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
            <DropdownMenuItem onClick={onOpenDdlEditor}>
              <FileCode2 />
              {t("dbObject.contextHeader.ddlEditor")}
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onGenerateSql}>
              <Wand2 />
              {t("dbObject.contextHeader.generateSql")}
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem onClick={onRefresh}>
              <RefreshCw />
              {t("dbObject.contextHeader.refresh")}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  );
}
