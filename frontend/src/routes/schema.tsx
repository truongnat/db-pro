import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/schema")({
  component: SchemaPage,
});

function SchemaPage() {
  return (
    <div>
      <h1 className="text-xl font-semibold" style={{ color: "var(--color-text)" }}>
        Schema Browser
      </h1>
      <p className="mt-2" style={{ color: "var(--color-text-secondary)" }}>
        Schema explorer — Phase 9
      </p>
    </div>
  );
}
