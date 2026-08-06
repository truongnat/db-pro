import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/")({
  component: IndexPage,
});

function IndexPage() {
  return (
    <div className="flex h-full items-center justify-center">
      <p className="text-muted-foreground">
        Select a connection to get started
      </p>
    </div>
  );
}
