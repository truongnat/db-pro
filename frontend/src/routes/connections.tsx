import { createFileRoute } from "@tanstack/react-router";

import { ConnectionsPage } from "@/modules/connection/pages/connections-page";

export const Route = createFileRoute("/connections")({
  component: ConnectionsPage,
});
