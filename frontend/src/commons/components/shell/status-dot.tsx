import { cn } from "@/lib/utils";

interface StatusDotProps {
  status: string;
}

export function StatusDot({ status }: StatusDotProps) {
  const className =
    status === "connected"
      ? "bg-[var(--app-success)]"
      : status === "connecting"
        ? "bg-[var(--app-warning)] animate-pulse"
        : status === "error"
          ? "bg-[var(--app-danger)]"
          : "bg-[var(--app-text-dim)]";
  return <span className={cn("h-2 w-2 shrink-0 rounded-full", className)} />;
}
