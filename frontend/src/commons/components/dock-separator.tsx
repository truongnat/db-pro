import type { useResizableDock } from "@/hooks/use-resizable-dock";
import { useTranslation } from "@/commons/locales/useTranslation";
import { cn } from "@/lib/utils";

interface DockSeparatorProps {
  separatorProps: ReturnType<typeof useResizableDock>["separatorProps"];
  isCollapsed: boolean;
}

export function DockSeparator({ separatorProps, isCollapsed }: DockSeparatorProps) {
  const { t } = useTranslation();

  return (
    <div
      {...separatorProps}
      className={cn(
        "relative h-[3px] w-full cursor-row-resize bg-transparent transition-colors",
        "hover:bg-primary/25 active:bg-primary/70",
        "focus-visible:z-10 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0",
        isCollapsed && "bg-primary/50 hover:bg-primary/70",
      )}
      aria-label={isCollapsed ? t("dock.expandPanel") : t("dock.resizePanel")}
    >
      {/* Larger hit area */}
      <div className="absolute -inset-y-1 inset-x-0" />
    </div>
  );
}
