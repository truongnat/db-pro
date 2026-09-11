import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import { useShellStore } from "@/commons/stores/shell.store";

export const Route = createFileRoute("/connections")({
  component: ConnectionsRedirect,
});

function ConnectionsRedirect() {
  const navigate = useNavigate();
  useEffect(() => {
    useShellStore.getState().setSidebarView("explorer");
    navigate({ to: "/" });
  }, [navigate]);
  return null;
}
