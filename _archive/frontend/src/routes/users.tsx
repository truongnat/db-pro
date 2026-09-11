import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import { useShellStore } from "@/commons/stores/shell.store";

export const Route = createFileRoute("/users")({
  component: UsersRedirect,
});

function UsersRedirect() {
  const navigate = useNavigate();
  useEffect(() => {
    useShellStore.getState().setSidebarView("users");
    navigate({ to: "/" });
  }, [navigate]);
  return null;
}
