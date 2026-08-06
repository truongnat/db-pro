import { createFileRoute } from "@tanstack/react-router";

import { QueryPage } from "@/modules/query/pages/query-page";

export const Route = createFileRoute("/query")({
  component: QueryPage,
});
