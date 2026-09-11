import { createFileRoute } from "@tanstack/react-router";

import { WelcomeView } from "@/commons/components/welcome-view";

export const Route = createFileRoute("/")({
  component: () => <WelcomeView />,
});
