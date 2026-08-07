import { Link, useNavigate } from "@tanstack/react-router";
import { Plus } from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";
import { useConnectionModuleStore } from "@/modules/connection/state/connection.store";

function statusOf(statuses: Record<string, string>, id: string) {
  return statuses[id] ?? "disconnected";
}

function StatusDot({ status }: { status: string }) {
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

export function ConnectionsView() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const connections = useConnectionList();
  const statuses = useConnectionModuleStore((s) => s.statuses);
  const activeConnectionId = useConnectionStore((s) => s.activeConnectionId);

  return (
    <div className="flex min-h-0 flex-col gap-2">
      <div className="flex items-center justify-between px-2">
        <span className="text-[10px] font-semibold uppercase tracking-widest text-[var(--app-text-dim)]">
          {t("connection.title")}
        </span>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-5 w-5 text-muted-foreground"
              onClick={() => navigate({ to: "/connection-editor" })}
            >
              <Plus className="h-3.5 w-3.5" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right" sideOffset={4}>{t("connection.new")}</TooltipContent>
        </Tooltip>
      </div>

      <div className="flex flex-col gap-0.5">
        {connections.isLoading && (
          <p className="px-2 py-1 text-xs text-[var(--app-text-dim)]">{t("common.states.loading")}</p>
        )}
        {!connections.isLoading && (connections.data?.length ?? 0) === 0 && (
          <p className="px-2 py-1 text-xs text-[var(--app-text-dim)]">{t("common.states.empty")}</p>
        )}
        {connections.data?.map((conn) => {
          const isActive = conn.id === activeConnectionId;
          return (
            <Link
              key={conn.id}
              to="/connection-editor"
              search={{ id: conn.id }}
              title={conn.name}
              className={cn(
                "flex items-center gap-2.5 rounded-md px-2 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-[var(--app-hover)] hover:text-foreground",
                isActive && "bg-[var(--app-active)] text-foreground",
              )}
            >
              <StatusDot status={statusOf(statuses, conn.id)} />
              <span className="flex-1 truncate">{conn.name}</span>
              {conn.group && <small className="text-[10px] text-[var(--app-text-dim)]">{conn.group}</small>}
            </Link>
          );
        })}
      </div>
    </div>
  );
}
