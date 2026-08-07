import { AlertTriangle } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface IdeErrorStateProps {
  title?: string;
  message: string;
  onRetry?: () => void;
  onDetails?: () => void;
  className?: string;
}

export function IdeErrorState({
  title = "Something went wrong",
  message,
  onRetry,
  onDetails,
  className,
}: IdeErrorStateProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center px-6 py-12", className)}>
      <div className="mb-4 grid h-10 w-10 place-items-center rounded-full bg-destructive/10">
        <AlertTriangle className="h-5 w-5 text-destructive" />
      </div>
      <p className="mb-1 text-sm font-medium text-foreground">{title}</p>
      <p className="mb-4 max-w-md text-center text-xs text-[var(--app-text-dim)]">{message}</p>
      <div className="flex items-center gap-2">
        {onRetry && (
          <Button variant="outline" size="sm" onClick={onRetry}>
            Retry
          </Button>
        )}
        {onDetails && (
          <Button variant="ghost" size="sm" onClick={onDetails}>
            View details
          </Button>
        )}
      </div>
    </div>
  );
}
