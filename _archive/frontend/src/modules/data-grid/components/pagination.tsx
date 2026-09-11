import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface PaginationProps {
  page: number;
  pageSize: number;
  totalCount: number;
  onPageChange: (page: number) => void;
  onPageSizeChange: (size: number) => void;
}

const PAGE_SIZES = [25, 50, 100, 200];

export function Pagination({
  page,
  pageSize,
  totalCount,
  onPageChange,
  onPageSizeChange,
}: PaginationProps) {
  const { t } = useTranslation();
  const totalPages = Math.max(1, Math.ceil(totalCount / pageSize));

  const navButton = "h-auto px-1.5 py-0.5 text-xs hover:bg-background disabled:opacity-40";

  return (
    <div className="flex items-center justify-between gap-3 border-t border-[var(--border-subtle)] bg-background px-3 py-1.5 text-xs text-[var(--text-secondary)]">
      <span>{t("dataGrid.rowsCount", { count: totalCount })}</span>

      <div className="flex items-center gap-3">
        <Select value={String(pageSize)} onValueChange={(v) => onPageSizeChange(Number(v))}>
          <SelectTrigger className="h-auto w-auto text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {PAGE_SIZES.map((size) => (
              <SelectItem key={size} value={String(size)}>
                {size} {t("dataGrid.pageSize")}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <div className="flex items-center gap-1">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className={navButton}
            disabled={page <= 1}
            onClick={() => onPageChange(page - 1)}
            aria-label={t("dataGrid.prevPage")}
          >
            &lsaquo;
          </Button>
          <span className="min-w-14 text-center">
            {t("dataGrid.page")} {page} {t("dataGrid.of")} {totalPages}
          </span>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className={navButton}
            disabled={page >= totalPages}
            onClick={() => onPageChange(page + 1)}
            aria-label={t("dataGrid.nextPage")}
          >
            &rsaquo;
          </Button>
        </div>
      </div>
    </div>
  );
}
