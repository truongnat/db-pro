import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useTranslation } from "@/commons/locales/useTranslation";

interface SchemaToolbarProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  onRefresh: () => void;
  isRefreshing: boolean;
  tableCount: number;
}

export function SchemaToolbar({
  searchQuery,
  onSearchChange,
  onRefresh,
  isRefreshing,
  tableCount,
}: SchemaToolbarProps) {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-2 border-b border-border bg-card px-3 py-2">
      <Input
        type="text"
        className="flex-1 rounded-sm border border-border bg-background px-2 py-1 text-sm text-foreground outline-none"
        placeholder={t("schema.searchPlaceholder")}
        value={searchQuery}
        onChange={(e) => onSearchChange(e.target.value)}
      />

      <span className="text-xs text-muted-foreground">
        {tableCount}
      </span>

      <Button
        type="button"
        variant="outline"
        size="sm"
        onClick={onRefresh}
        disabled={isRefreshing}
      >
        {isRefreshing ? t("common.states.loading") : t("common.actions.refresh")}
      </Button>
    </div>
  );
}
