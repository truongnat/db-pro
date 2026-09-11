import React from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import { bootstrapServices } from "./app/app.module";
import { RootErrorBoundary } from "./app/root-error-boundary";
import { SANITIZED_REACT_ROOT_OPTIONS } from "./app/root-error-reporting";
import "./styles/globals.css";
import "@/commons/locales/i18n";
import "@/commons/actions";

async function main() {
  try {
    // Dev-only: when the frontend is served by `vite dev` in a plain browser
    // (no Tauri webview, therefore no IPC bridge) install the in-memory fixture
    // backend so every screen is reviewable without the Rust core. The dynamic
    // import lives inside `import.meta.env.DEV`, which Rollup folds to `false`
    // in production builds and dead-code-eliminates along with the chunk.
    if (import.meta.env.DEV) {
      const { installBrowserBackend } = await import("./dev/browser-backend");
      installBrowserBackend();
    }

    await bootstrapServices();

    ReactDOM.createRoot(document.getElementById("root")!, SANITIZED_REACT_ROOT_OPTIONS).render(
      <React.StrictMode>
        <RootErrorBoundary>
          <App />
        </RootErrorBoundary>
      </React.StrictMode>,
    );
  } catch (error) {
    console.error("Bootstrap failed", error);
  } finally {
    // Hand off to the main window regardless of bootstrap success so the
    // splash screen can never block startup forever. A Rust-side timeout
    // also guarantees this if the frontend crashes entirely.
    try {
      await invoke("finish_startup");
    } catch (handoffError) {
      console.error("Startup handoff failed", handoffError);
    }
  }
}

main();
