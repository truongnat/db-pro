import { createRootRoute } from "@tanstack/react-router";

import { AppShell } from "@/commons/components/shell";

export const Route = createRootRoute({
  component: () => <AppShell />,
});
