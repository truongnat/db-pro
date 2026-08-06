import { cn } from "@/ui/lib/utils";
import { useTranslation } from "@/commons/locales/useTranslation";

import type { ConnectionStatus } from "../types/connection.types";

const STATUS_STYLES: Record<ConnectionStatus, string> = {
  connected: "bg-[var(--color-success,#22c55e)]",
  disconnected: "bg-[var(--color-text-secondary,#6b7280)]",
  connecting: "bg-[var(--color-warning,#f59e0b)]",
  error: "bg-[var(--color-error,#ef4444)]",
};

const STATUS_I18N_KEYS: Record<ConnectionStatus, string> = {
  connected: "common.states.connected",
  disconnected: "common.states.disconnected",
  connecting: "common.states.loading",
  error: "common.states.error",
};

interface ConnectionStatusBadgeProps {
  status: ConnectionStatus;
  className?: string;
}

export function ConnectionStatusBadge({ status, className }: ConnectionStatusBadgeProps) {
  const { t } = useTranslation();

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 text-xs",
        className,
      )}
      style={{ color: "var(--color-text-secondary)" }}
    >
      <span className={cn("h-2 w-2 rounded-full", STATUS_STYLES[status])} />
      {t(STATUS_I18N_KEYS[status])}
    </span>
  );
}
