import { useState } from "react";

import { ArrowUpDown, Columns3, Filter, RefreshCw } from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

import type { ColumnMeta, GridFilter, GridSort } from "../types/data-grid.types";
import { VisualFilterBuilder } from "./visual-filter-builder";

interface DataToolbarProps {
  columns: ColumnMeta[];
  rowCount: number;
  filters: GridFilter[];
  sorts: GridSort[];
  hiddenColumns: string[];
  onAddFilter: (filter: GridFilter) => void;
  onRemoveFilter: (index: number) => void;
  onSetSorts: (sorts: GridSort[]) => void;
  onToggleHiddenColumn: (column: string) => void;
  onShowAllColumns: () => void;
  onRefresh: () => void;
  isRefreshing: boolean;
}

export function DataToolbar({
  columns,
  rowCount,
  filters,
  sorts,
  hiddenColumns,
  onAddFilter,
  onRemoveFilter,
  onSetSorts,
  onToggleHiddenColumn,
  onShowAllColumns,
  onRefresh,
  isRefreshing,
}: DataToolbarProps) {
  const { t } = useTranslation();
  const [sortColumn, setSortColumn] = useState(columns[0]?.name ?? "");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");

  const countBadge = (n: number) =>
    n > 0 ? (
      <span className="ml-1 inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-primary px-1 text-[11px] font-medium leading-none text-primary-foreground">
        {n}
      </span>
    ) : null;

  const handleAddSort = () => {
    if (!sortColumn) return;
    onSetSorts([...sorts.filter((s) => s.column !== sortColumn), { column: sortColumn, direction: sortDir }]);
  };

  const dirLabel = (d: "asc" | "desc") => t(d === "asc" ? "dataGrid.sortAsc" : "dataGrid.sortDesc");

  const toolButton =
    "h-auto gap-1 px-2 py-1 text-xs font-normal text-[var(--app-text-muted)] hover:bg-background hover:text-foreground";

  return (
    <div className="flex items-center justify-between gap-2 border-b border-[var(--app-border-subtle)] bg-background px-2 py-1.5">
      <div className="flex items-center gap-0.5">
        <Popover>
          <PopoverTrigger asChild>
            <Button type="button" variant="ghost" className={toolButton}>
              <Filter />
              {t("dataGrid.toolbarFilter")}
              {countBadge(filters.length)}
            </Button>
          </PopoverTrigger>
          <PopoverContent align="start" sideOffset={6} className="w-80">
            <VisualFilterBuilder
              columns={columns}
              filters={filters}
              onAddFilter={onAddFilter}
              onRemoveFilter={onRemoveFilter}
            />
          </PopoverContent>
        </Popover>

        <Popover>
          <PopoverTrigger asChild>
            <Button type="button" variant="ghost" className={toolButton}>
              <ArrowUpDown />
              {t("dataGrid.toolbarSort")}
              {countBadge(sorts.length)}
            </Button>
          </PopoverTrigger>
          <PopoverContent align="start" sideOffset={6} className="w-80">
            <div className="flex flex-col gap-2.5">
              {sorts.length > 0 && (
                <div className="flex flex-col gap-1">
                  {sorts.map((s, i) => (
                    <div key={s.column} className="flex items-center justify-between gap-2 rounded-sm bg-muted px-2 py-1 text-xs">
                      <span className="font-medium">{s.column}</span>
                      <div className="flex items-center gap-1">
                        <button
                          type="button"
                          className="px-1 text-[var(--app-text-muted)] hover:text-foreground"
                          onClick={() => {
                            onSetSorts(
                              sorts.map((x, idx) =>
                                idx === i
                                  ? { ...x, direction: x.direction === "asc" ? "desc" : "asc" }
                                  : x,
                              ),
                            );
                          }}
                          title={dirLabel(s.direction)}
                        >
                          {s.direction === "asc" ? "\u25B2" : "\u25BC"}
                        </button>
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          className="h-auto px-1 py-0 text-xs opacity-60 hover:bg-transparent hover:opacity-100"
                          onClick={() => onSetSorts(sorts.filter((_, idx) => idx !== i))}
                        >
                          ×
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              )}

              <div className="flex items-center gap-2">
                <Select value={sortColumn} onValueChange={setSortColumn}>
                  <SelectTrigger className="w-auto text-xs">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {columns.map((c) => (
                      <SelectItem key={c.name} value={c.name}>
                        {c.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>

                <Select value={sortDir} onValueChange={(v) => setSortDir(v as "asc" | "desc")}>
                  <SelectTrigger className="w-auto text-xs">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="asc">{dirLabel("asc")}</SelectItem>
                    <SelectItem value="desc">{dirLabel("desc")}</SelectItem>
                  </SelectContent>
                </Select>

                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-auto px-2 py-1 text-xs text-primary hover:bg-background"
                  onClick={handleAddSort}
                >
                  {t("dataGrid.addSort")}
                </Button>
              </div>
            </div>
          </PopoverContent>
        </Popover>

        <Popover>
          <PopoverTrigger asChild>
            <Button type="button" variant="ghost" className={toolButton}>
              <Columns3 />
              {t("dataGrid.toolbarColumns")}
              {countBadge(hiddenColumns.length)}
            </Button>
          </PopoverTrigger>
          <PopoverContent align="start" sideOffset={6} className="w-64 p-2">
            <div className="mb-2 flex items-center justify-between">
              <span className="text-xs font-medium text-[var(--app-text-muted)]">{t("dataGrid.toolbarColumns")}</span>
              {hiddenColumns.length > 0 && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-auto px-1 py-0 text-xs text-primary hover:bg-background"
                  onClick={onShowAllColumns}
                >
                  {t("dataGrid.showAllColumns")}
                </Button>
              )}
            </div>
            <ScrollArea className="max-h-64">
              <div className="flex flex-col gap-1">
                {columns.map((c) => (
                  <label
                    key={c.name}
                    className="flex cursor-pointer items-center gap-2 rounded-sm px-1 py-1 text-xs hover:bg-muted"
                  >
                    <Checkbox
                      checked={!hiddenColumns.includes(c.name)}
                      onCheckedChange={() => onToggleHiddenColumn(c.name)}
                    />
                    <span className="truncate">{c.name}</span>
                  </label>
                ))}
              </div>
            </ScrollArea>
          </PopoverContent>
        </Popover>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              type="button"
              variant="ghost"
              className={toolButton}
              onClick={onRefresh}
              disabled={isRefreshing}
              aria-label={t("dataGrid.refresh")}
            >
              <RefreshCw className={isRefreshing ? "animate-spin" : undefined} />
              {t("dataGrid.refresh")}
            </Button>
          </TooltipTrigger>
          <TooltipContent>{t("dataGrid.refresh")}</TooltipContent>
        </Tooltip>
      </div>

      <span className="px-1 text-[11px] text-[var(--app-text-muted)]">
        {t("dataGrid.rowsCount", { count: rowCount })}
      </span>
    </div>
  );
}
