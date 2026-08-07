import React from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import { bootstrapServices } from "./app/app.module";
import "./styles/globals.css";
import "@/commons/locales/i18n";

async function main() {
  try {
    await bootstrapServices();

    ReactDOM.createRoot(document.getElementById("root")!).render(
      <React.StrictMode>
        <App />
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
