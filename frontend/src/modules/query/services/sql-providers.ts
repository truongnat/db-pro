import type * as monaco from "monaco-editor";

import { createSqlCompletionProvider } from "./sql-completion-provider";

let providersRegistered = false;

export function registerSqlProviders(monacoInstance: typeof monaco): void {
  if (providersRegistered) return;
  providersRegistered = true;

  monacoInstance.languages.registerCompletionItemProvider(
    "sql",
    createSqlCompletionProvider(monacoInstance),
  );
}
