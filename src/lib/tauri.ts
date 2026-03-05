import { invoke } from "@tauri-apps/api/core";

import type {
  ConnectionInput,
  ConnectionListItem,
  NavigatorTree,
  QueryRequest,
  QueryResult,
} from "../types";

function hasTauriRuntime(): boolean {
  const runtime = (window as typeof window & {
    __TAURI_INTERNALS__?: {
      invoke?: unknown;
    };
  }).__TAURI_INTERNALS__;

  return typeof runtime?.invoke === "function";
}

async function invokeCommand<T>(
  command: string,
  payload?: Record<string, unknown>,
): Promise<T> {
  if (!hasTauriRuntime()) {
    throw new Error(
      "Tauri runtime not detected. Run this app with `npm run tauri dev`.",
    );
  }

  return invoke<T>(command, payload);
}

export async function listConnections(): Promise<ConnectionListItem[]> {
  return invokeCommand<ConnectionListItem[]>("list_connections");
}

export async function upsertConnection(
  input: ConnectionInput,
): Promise<ConnectionListItem> {
  return invokeCommand<ConnectionListItem>("upsert_connection", { input });
}

export async function getConnection(connectionId: string): Promise<ConnectionInput> {
  return invokeCommand<ConnectionInput>("get_connection", { connectionId });
}

export async function deleteConnection(connectionId: string): Promise<void> {
  return invokeCommand("delete_connection", { connectionId });
}

export async function resetConnections(): Promise<ConnectionListItem[]> {
  return invokeCommand<ConnectionListItem[]>("reset_connections");
}

export async function testConnection(connectionId: string): Promise<string> {
  return invokeCommand<string>("test_connection", { connectionId });
}

export async function executeQuery(
  request: QueryRequest,
): Promise<QueryResult> {
  return invokeCommand<QueryResult>("execute_query", { request });
}

export async function cancelQuery(connectionId: string): Promise<string> {
  return invokeCommand<string>("cancel_query", { connectionId });
}

export async function loadNavigator(
  connectionId: string,
): Promise<NavigatorTree> {
  return invokeCommand<NavigatorTree>("load_navigator", { connectionId });
}
