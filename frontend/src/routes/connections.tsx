import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/connections")({
  component: ConnectionsPage,
});

function ConnectionsPage() {
  return (
    <div>
      <h1 className="text-xl font-semibold" style={{ color: "var(--color-text)" }}>
        Connections
      </h1>
      <p className="mt-2" style={{ color: "var(--color-text-secondary)" }}>
        Connection management — Phase 7
      </p>
    </div>
  );
}
