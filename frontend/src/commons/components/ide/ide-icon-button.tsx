import type { LucideIcon } from "lucide-react";

import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

interface IdeIconButtonProps {
  icon: LucideIcon;
  label: string;
  onClick?: () => void;
  active?: boolean;
  disabled?: boolean;
  size?: "sm" | "md";
  className?: string;
}

export function IdeIconButton({
  icon: Icon,
  label,
  onClick,
  active,
  disabled,
  size = "md",
  className,
}: IdeIconButtonProps) {
  const sizeClass = size === "sm" ? "h-6 w-6" : "h-7 w-7";
  const iconSize = size === "sm" ? "h-3.5 w-3.5" : "h-4 w-4";

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          type="button"
          className={cn(
            "flex items-center justify-center rounded text-[var(--app-text-muted)] transition-colors",
            "hover:bg-[var(--app-hover)] hover:text-foreground",
            "disabled:pointer-events-none disabled:opacity-40",
            sizeClass,
            active && "bg-[var(--app-primary-soft)] text-primary",
            className,
          )}
          onClick={onClick}
          disabled={disabled}
          aria-label={label}
        >
          <Icon className={iconSize} />
        </button>
      </TooltipTrigger>
      <TooltipContent side="bottom" sideOffset={4}>
        {label}
      </TooltipContent>
    </Tooltip>
  );
}
