import { useState } from "react";

import { SchemaDetailPanel } from "./schema-detail-panel";
import type { DetailTab } from "../types/schema.types";

interface SchemaObjectTabContentProps {
  connectionId: string | null;
  schema: string;
  objectName: string;
  objectType: "table" | "view" | "function" | "sequence" | "type";
}

export function SchemaObjectTabContent({
  connectionId,
  schema,
  objectName,
  objectType,
}: SchemaObjectTabContentProps) {
  const [activeTab, setActiveTab] = useState<DetailTab>("columns");

  if (!connectionId) return null;

  const nodeType = objectType === "view" ? "view" as const : "table" as const;

  return (
    <SchemaDetailPanel
      connectionId={connectionId}
      schema={schema}
      table={objectName}
      nodeType={nodeType}
      activeTab={activeTab}
      onTabChange={setActiveTab}
    />
  );
}
