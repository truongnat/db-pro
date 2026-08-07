import { cn } from "@/lib/utils";

interface StatusDotProps {
  status: string;
}

export function StatusDot({ status }: StatusDotProps) {
  const className =
    status === "connected"
      ? "bg-success shadow-[0_0_0_3px_rgba(34,197,94,0.15)]"
      : status === "connecting"
        ? "bg-warning shadow-[0_0_0_3px_rgba(229,195,106,0.15)]"
        : status === "error"
          ? "bg-destructive"
          : "bg-muted-foreground";
  return <span className={cn("h-[7px] w-[7px] shrink-0 rounded-full", className)} />;
}
