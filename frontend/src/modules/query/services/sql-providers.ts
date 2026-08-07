import type * as monaco from "monaco-editor";

import { createSqlCompletionProvider } from "./sql-completion-provider";
import { createSqlHoverProvider } from "./sql-hover-provider";
import { registerSqlDiagnostics } from "./sql-diagnostics";

let providersRegistered = false;

export function registerSqlProviders(monacoInstance: typeof monaco): void {
  if (providersRegistered) return;
  providersRegistered = true;

  monacoInstance.languages.registerCompletionItemProvider(
    "sql",
    createSqlCompletionProvider(monacoInstance),
  );

  monacoInstance.languages.registerHoverProvider(
    "sql",
    createSqlHoverProvider(monacoInstance),
  );

  registerSqlDiagnostics(monacoInstance);
}
