import * as React from "react";

import { cn } from "@/lib/utils";

function Kbd({ className, ...props }: React.ComponentProps<"kbd">) {
  return (
    <kbd
      data-slot="kbd"
      className={cn(
        "pointer-events-none inline-flex h-[1.4em] min-w-5 select-none items-center justify-center gap-0.5 rounded-[calc(var(--radius-xs,0.25rem))] border border-border/60 bg-muted/60 px-1 font-mono text-[0.7em] font-medium text-muted-foreground shadow-[inset_0_-1px_0_color-mix(in_oklch,var(--border-default)_80%,transparent),var(--shadow-xs)] [[data-slot=tooltip-content]_&]:bg-background/20 [[data-slot=tooltip-content]_&]:text-background dark:bg-muted/40 dark:border-border/50 dark:text-muted-foreground/90",
        className,
      )}
      {...props}
    />
  );
}

function KbdGroup({ className, ...props }: React.ComponentProps<"span">) {
  return (
    <span
      data-slot="kbd-group"
      className={cn("inline-flex items-center gap-0.5", className)}
      {...props}
    />
  );
}

export { Kbd, KbdGroup };
