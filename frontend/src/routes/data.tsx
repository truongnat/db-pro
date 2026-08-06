import { createFileRoute } from "@tanstack/react-router";

import { DataPage } from "@/modules/data-grid/pages/data-page";

export const Route = createFileRoute("/data")({
  component: DataPage,
});
