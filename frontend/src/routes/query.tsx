import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/query")({
  component: QueryPage,
});

function QueryPage() {
  return (
    <div>
      <h1 className="text-xl font-semibold" style={{ color: "var(--color-text)" }}>
        Query Editor
      </h1>
      <p className="mt-2" style={{ color: "var(--color-text-secondary)" }}>
        SQL editor and results — Phase 8
      </p>
    </div>
  );
}
