export type QueryAction =
  | "execute"
  | "executeCurrent"
  | "explain"
  | "format"
  | "clear"
  | "cancel"
  | "export"
  | "importSql"
  | "exportSql"
  | "saveQuery";

const listeners = new Map<QueryAction, Set<() => void>>();

export function onQueryAction(
  action: QueryAction,
  handler: () => void,
): () => void {
  if (!listeners.has(action)) {
    listeners.set(action, new Set());
  }
  listeners.get(action)!.add(handler);
  return () => {
    listeners.get(action)?.delete(handler);
  };
}

export function dispatchQueryAction(action: QueryAction): void {
  listeners.get(action)?.forEach((fn) => fn());
}
