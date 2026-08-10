import { useEffect, useState } from "react";
import { ArrowUpDown, Columns3, Filter, Trash2 } from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
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
  /** Applied filters (drive the query). */
  filters: GridFilter[];
  /** Applied sorts (drive the query). */
  sorts: GridSort[];
  /** Draft filters (UI-only, not yet applied). */
  draftFilters: GridFilter[];
  /** Draft sorts (UI-only, not yet applied). */
  draftSorts: GridSort[];
  hiddenColumns: string[];
  onAddDraftFilter: (filter: GridFilter) => void;
  onRemoveDraftFilter: (index: number) => void;
  onApplyFilters: () => void;
  onClearFilters: () => void;
  onAddDraftSort: (sort: GridSort) => void;
  onRemoveDraftSort: (index: number) => void;
  onApplySorts: () => void;
  onClearSorts: () => void;
  onToggleHiddenColumn: (column: string) => void;
  onShowAllColumns: () => void;
}

export function DataToolbar({
  columns,
  rowCount,
  filters,
  sorts,
  draftFilters,
  draftSorts,
  hiddenColumns,
  onAddDraftFilter,
  onRemoveDraftFilter,
  onApplyFilters,
  onClearFilters,
  onAddDraftSort,
  onRemoveDraftSort,
  onApplySorts,
  onClearSorts,
  onToggleHiddenColumn,
  onShowAllColumns,
}: DataToolbarProps) {
  const { t } = useTranslation();
  const [sortColumn, setSortColumn] = useState(columns[0]?.name ?? "");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");

  useEffect(() => {
    if (columns.length > 0 && !columns.some((c) => c.name === sortColumn)) {
      setSortColumn(columns[0].name);
    }
  }, [columns, sortColumn]);

  const countBadge = (n: number) =>
    n > 0 ? (
      <span className="ml-1 inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-primary px-1 text-[11px] font-medium leading-none text-primary-foreground">
        {n}
      </span>
    ) : null;

  const handleAddSort = () => {
    if (!sortColumn) return;
    onAddDraftSort({ column: sortColumn, direction: sortDir });
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
              draftFilters={draftFilters}
              onAddDraftFilter={onAddDraftFilter}
              onRemoveDraftFilter={onRemoveDraftFilter}
              onApply={onApplyFilters}
              onClear={onClearFilters}
              appliedCount={filters.length}
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
            <div className="flex flex-col gap-2.5 p-1">
              {/* Draft sort rows */}
              {draftSorts.length > 0 && (
                <div className="flex flex-col gap-1.5">
                  {draftSorts.map((s, i) => (
                    <div
                      key={`${s.column}-${i}`}
                      className="flex items-center gap-2 rounded-md bg-muted/50 px-2 py-1.5 text-xs"
                    >
                      <span className="font-medium text-foreground">{s.column}</span>
                      <span className="text-[var(--app-text-muted)]">
                        {dirLabel(s.direction)}
                      </span>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            className="ml-auto h-5 w-5 p-0 text-[var(--app-text-muted)] hover:bg-destructive/10 hover:text-destructive"
                            onClick={() => onRemoveDraftSort(i)}
                          >
                            <Trash2 className="h-3 w-3" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>Remove sort</TooltipContent>
                      </Tooltip>
                    </div>
                  ))}
                </div>
              )}

              {/* Add sort row */}
              <div className="flex items-center gap-2">
                <Select value={sortColumn} onValueChange={setSortColumn}>
                  <SelectTrigger className="h-7 w-auto text-xs">
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
                  <SelectTrigger className="h-7 w-auto text-xs">
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
                  className="h-7 shrink-0 px-2 text-xs text-primary hover:bg-background"
                  onClick={handleAddSort}
                >
                  + {t("dataGrid.addSort")}
                </Button>
              </div>

              {/* Footer: Apply / Clear */}
              <div className="flex items-center justify-between border-t border-[var(--app-border-subtle)] pt-2">
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-7 px-2 text-xs text-[var(--app-text-muted)] hover:bg-background"
                  onClick={onClearSorts}
                  disabled={sorts.length === 0 && draftSorts.length === 0}
                >
                  {t("common.actions.clear")}
                </Button>
                <Button
                  type="button"
                  size="sm"
                  className="h-7 px-3 text-xs"
                  onClick={onApplySorts}
                  disabled={draftSorts.length === 0}
                >
                  {t("common.actions.apply")}
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
              <span className="text-xs font-medium text-[var(--app-text-muted)]">
                {t("dataGrid.toolbarColumns")}
              </span>
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
      </div>

      <span className="px-1 text-[11px] text-[var(--app-text-muted)]">
        {t("dataGrid.rowsCount", { count: rowCount })}
      </span>
    </div>
  );
}
