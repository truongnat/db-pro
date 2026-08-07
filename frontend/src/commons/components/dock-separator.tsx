import type { useResizableDock } from "@/hooks/use-resizable-dock";
import { useTranslation } from "@/commons/locales/useTranslation";
import { cn } from "@/lib/utils";

interface DockSeparatorProps {
  separatorProps: ReturnType<typeof useResizableDock>["separatorProps"];
  isCollapsed: boolean;
}

export function DockSeparator({
  separatorProps,
  isCollapsed,
}: DockSeparatorProps) {
  const { t } = useTranslation();

  return (
    <div
      {...separatorProps}
      className={cn(
        "h-1 w-full cursor-row-resize border-y border-transparent bg-border transition-colors hover:bg-primary focus-visible:z-10 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0",
        isCollapsed && "bg-primary/50",
      )}
      aria-label={isCollapsed ? t("dock.expandPanel") : t("dock.resizePanel")}
    />
  );
}
