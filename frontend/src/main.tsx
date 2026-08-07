import React from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import { bootstrapServices } from "./app/app.module";
import "./styles/globals.css";
import "@/commons/locales/i18n";

async function main() {
  await bootstrapServices();

  ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );

  invoke("close_splashscreen");
}

main();
