import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/")({
  component: IndexPage,
});

function IndexPage() {
  return (
    <div className="flex h-full items-center justify-center">
      <p style={{ color: "var(--color-text-secondary)" }}>
        Select a connection to get started
      </p>
    </div>
  );
}
