import type { LucideIcon } from "lucide-react";
import { Database } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface IdeEmptyStateProps {
  icon?: LucideIcon;
  title: string;
  description?: string;
  action?: {
    label: string;
    onClick: () => void;
  };
  className?: string;
}

export function IdeEmptyState({
  icon: Icon = Database,
  title,
  description,
  action,
  className,
}: IdeEmptyStateProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center px-6 py-12", className)}>
      <div className="mb-4 grid h-10 w-10 place-items-center rounded-lg bg-muted">
        <Icon className="h-5 w-5 text-[var(--app-text-dim)]" />
      </div>
      <p className="mb-1 text-sm font-medium text-foreground">{title}</p>
      {description && (
        <p className="mb-4 max-w-sm text-center text-xs text-[var(--app-text-dim)]">{description}</p>
      )}
      {action && (
        <Button variant="outline" size="sm" onClick={action.onClick}>
          {action.label}
        </Button>
      )}
    </div>
  );
}
