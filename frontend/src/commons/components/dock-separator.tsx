import type { useResizableDock } from "@/hooks/use-resizable-dock";
import { cn } from "@/lib/utils";

interface DockSeparatorProps {
  separatorProps: ReturnType<typeof useResizableDock>["separatorProps"];
  isCollapsed: boolean;
}

export function DockSeparator({
  separatorProps,
  isCollapsed,
}: DockSeparatorProps) {
  return (
    <div
      {...separatorProps}
      className={cn(
        "h-1 w-full cursor-row-resize border-y border-transparent bg-border transition-colors hover:bg-primary focus-visible:z-10 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0",
        isCollapsed && "bg-primary/50",
      )}
      aria-label={isCollapsed ? "Expand panel" : "Resize panel"}
    />
  );
}
