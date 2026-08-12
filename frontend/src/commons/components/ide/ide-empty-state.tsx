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
    icon?: LucideIcon;
  };
  secondaryAction?: {
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
  secondaryAction,
  className,
}: IdeEmptyStateProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center px-6 py-12", className)}>
      <div className="mb-4 grid h-9 w-9 place-items-center rounded-lg bg-[var(--surface-panel)]">
        <Icon className="h-4 w-4 text-[var(--text-secondary)]" />
      </div>
      <p className="mb-1 text-[13px] font-medium text-foreground">{title}</p>
      {description && (
        <p className="mb-4 max-w-xs text-center text-[12px] leading-relaxed text-[var(--text-secondary)]">
          {description}
        </p>
      )}
      {action && (
        <Button
          variant="outline"
          size="sm"
          className="h-7 rounded-[5px] text-[13px]"
          onClick={action.onClick}
        >
          {action.icon && <action.icon className="mr-1.5 h-3.5 w-3.5" />}
          {action.label}
        </Button>
      )}
      {secondaryAction && (
        <button
          type="button"
          className="mt-2 text-[12px] text-[var(--text-secondary)] underline-offset-2 hover:text-foreground hover:underline"
          onClick={secondaryAction.onClick}
        >
          {secondaryAction.label}
        </button>
      )}
    </div>
  );
}
