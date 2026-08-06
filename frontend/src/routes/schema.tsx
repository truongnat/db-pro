import { createFileRoute } from "@tanstack/react-router";

import { SchemaPage } from "@/modules/schema/pages/schema-page";

export const Route = createFileRoute("/schema")({
  component: SchemaPage,
});
