import { cn } from "@/lib/utils";

interface StatusDotProps {
  status: string;
}

export function StatusDot({ status }: StatusDotProps) {
  const className =
    status === "connected"
      ? "bg-[var(--state-success)]"
      : status === "connecting"
        ? "bg-[var(--state-warning)] animate-pulse"
        : status === "error"
          ? "bg-[var(--state-danger)]"
          : "bg-[var(--text-tertiary)]";
  return <span className={cn("h-2 w-2 shrink-0 rounded-full", className)} />;
}
