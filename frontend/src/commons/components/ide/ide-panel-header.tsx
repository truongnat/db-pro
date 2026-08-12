import type { LucideIcon } from "lucide-react";

import { cn } from "@/lib/utils";

interface IdePanelHeaderProps {
  icon?: LucideIcon;
  title: string;
  subtitle?: string;
  actions?: React.ReactNode;
  className?: string;
}

export function IdePanelHeader({
  icon: Icon,
  title,
  subtitle,
  actions,
  className,
}: IdePanelHeaderProps) {
  return (
    <div
      className={cn(
        "flex shrink-0 items-center gap-2 border-b border-[var(--border-subtle)] px-3 py-2",
        className,
      )}
    >
      {Icon && <Icon className="h-4 w-4 shrink-0 text-primary" />}
      <div className="min-w-0 flex-1">
        <div className="truncate text-[13px] font-medium text-foreground">{title}</div>
        {subtitle && (
          <div className="truncate text-[11px] text-[var(--text-tertiary)]">{subtitle}</div>
        )}
      </div>
      {actions && <div className="flex shrink-0 items-center gap-1">{actions}</div>}
    </div>
  );
}
