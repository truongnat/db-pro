import { createRouter, RouterProvider } from "@tanstack/react-router";
import { routeTree } from "./routeTree.gen";
import { ThemeProvider } from "./app/providers/theme-provider";
import { QueryProvider } from "./app/providers/query-provider";
import { SnackbarProvider } from "./app/providers/snackbar.provider";
import { ModalProvider } from "./app/providers/modal.provider";
import { TooltipProvider } from "./components/ui/tooltip";
import { registerAllCommands } from "./commons/commands/register-commands";

export const router = createRouter({ routeTree });

registerAllCommands(router);

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

function App() {
  return (
    <ThemeProvider>
      <TooltipProvider>
        <QueryProvider>
          <SnackbarProvider>
            <ModalProvider>
              <RouterProvider router={router} />
            </ModalProvider>
          </SnackbarProvider>
        </QueryProvider>
      </TooltipProvider>
    </ThemeProvider>
  );
}

export default App;
