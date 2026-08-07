import { useTranslation } from "@/commons/locales/useTranslation";
import { Badge } from "@/components/ui/badge";

import type { ConnectionStatus } from "../types/connection.types";

const STATUS_VARIANT: Record<ConnectionStatus, "success" | "warning" | "error" | "secondary"> = {
  connected: "success",
  disconnected: "secondary",
  connecting: "warning",
  error: "error",
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
    <Badge variant={STATUS_VARIANT[status]} dot className={className}>
      {t(STATUS_I18N_KEYS[status])}
    </Badge>
  );
}
