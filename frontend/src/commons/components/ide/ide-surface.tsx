import { cn } from "@/lib/utils";

/**
 * IDE surface — applies semantic background layers.
 * L0 = deepest, L3 = elevated/popover
 */
export function IdeSurface({
  layer = 0,
  className,
  children,
  ...props
}: {
  layer?: 0 | 1 | 2 | 3;
  className?: string;
  children?: React.ReactNode;
} & React.HTMLAttributes<HTMLDivElement>) {
  const bgClass = [
    "bg-background",       // L0
    "bg-sidebar",          // L1
    "bg-card",             // L2
    "bg-popover",          // L3
  ][layer];

  return (
    <div className={cn(bgClass, className)} {...props}>
      {children}
    </div>
  );
}
