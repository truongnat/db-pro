import { cn } from "@/lib/utils";

/**
 * Resize handle — transparent by default, shows primary on hover/drag.
 * Use `orientation` to set the visual axis.
 */
export function IdeResizeHandle({
  orientation = "vertical",
  className,
}: {
  orientation?: "vertical" | "horizontal";
  className?: string;
}) {
  const isVertical = orientation === "vertical";

  return (
    <div
      className={cn(
        "group/resize relative shrink-0 transition-colors",
        isVertical ? "w-[3px] cursor-col-resize" : "h-[3px] cursor-row-resize",
        className,
      )}
    >
      {/* Visible handle line — appears on hover */}
      <div
        className={cn(
          "absolute inset-0 rounded-full bg-transparent transition-colors",
          "group-hover/resize:bg-primary/30",
          "group-active/resize:bg-primary",
        )}
      />
      {/* Wider hit area */}
      <div
        className={cn("absolute", isVertical ? "-inset-x-1 inset-y-0" : "inset-x-0 -inset-y-1")}
      />
    </div>
  );
}
