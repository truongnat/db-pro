import { createFileRoute } from "@tanstack/react-router";

import { ConnectionEditPage } from "@/modules/connection/pages/connection-edit-page";

export const Route = createFileRoute("/connection-editor")({
  component: ConnectionEditPage,
  validateSearch: (search: Record<string, unknown>) => {
    const result: { id?: string } = {};
    if (typeof search.id === "string") result.id = search.id;
    return result;
  },
});
