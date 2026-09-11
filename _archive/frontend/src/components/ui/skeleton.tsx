import * as React from "react";

import { cn } from "@/lib/utils";

function Skeleton({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="skeleton"
      className={cn(
        "relative isolate overflow-hidden rounded-md bg-muted/60 motion-safe:after:absolute motion-safe:after:inset-0 motion-safe:after:-translate-x-full motion-safe:after:animate-[shimmer_1.6s_infinite] motion-safe:after:bg-gradient-to-r motion-safe:after:from-transparent motion-safe:after:via-foreground/5 motion-safe:after:to-transparent",
        className,
      )}
      {...props}
    />
  );
}

export { Skeleton };
