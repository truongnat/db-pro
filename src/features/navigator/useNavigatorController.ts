import {
  type Dispatch,
  type SetStateAction,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import { loadNavigator } from "@/lib/tauri";
import { copyTextToClipboard } from "@/features/query/clipboard";
import type { QueryRunPageOptions } from "@/features/query/useQueryController";
import type { DbEngine, NavigatorTree } from "@/types";

import type { NavigatorActionRequest } from "./actions";
import {
  buildQualifiedObjectName,
  buildDdlTemplateSql,
  buildInsertTemplateSql,
  buildSelectFromObjectSql,
  buildUpdateTemplateSql,
} from "./sql";

const NAVIGATOR_FRONTEND_TIMEOUT_MS = 15_000;
const DEFAULT_QUERY_GRID_MODIFIERS = {
  quickFilter: "",
  sortColumn: "",
  sortDirection: "asc",
} as const;

type UseNavigatorControllerOptions = {
  selectedConnectionId: string | null;
  selectedConnectionEngine: DbEngine | undefined;
  queryPageSize: number;
  onSetSqlText: Dispatch<SetStateAction<string>>;
  onRunQueryPage: (
    sql: string,
    offset: number,
    runSource: string,
    options?: QueryRunPageOptions,
  ) => Promise<void>;
  onResetQueryGridModifiers: () => void;
  onStatus: (message: string) => void;
  onClearError: () => void;
  onError: (error: unknown) => void;
};

export function useNavigatorController({
  selectedConnectionId,
  selectedConnectionEngine,
  queryPageSize,
  onSetSqlText,
  onRunQueryPage,
  onResetQueryGridModifiers,
  onStatus,
  onClearError,
  onError,
}: UseNavigatorControllerOptions) {
  const [navigatorTree, setNavigatorTree] = useState<NavigatorTree | null>(null);
  const [navigatorLoading, setNavigatorLoading] = useState(false);
  const [navigatorError, setNavigatorError] = useState<string | null>(null);
  const navigatorRequestIdRef = useRef(0);

  const refreshNavigator = useCallback(
    async (connectionId?: string) => {
      const targetConnectionId = connectionId ?? selectedConnectionId;
      if (!targetConnectionId) {
        navigatorRequestIdRef.current += 1;
        setNavigatorTree(null);
        setNavigatorError(null);
        setNavigatorLoading(false);
        return;
      }

      const requestId = navigatorRequestIdRef.current + 1;
      navigatorRequestIdRef.current = requestId;

      try {
        setNavigatorLoading(true);
        setNavigatorError(null);
        const tree = await withPromiseTimeout(
          loadNavigator(targetConnectionId),
          NAVIGATOR_FRONTEND_TIMEOUT_MS,
          `Schema navigator load timed out after ${NAVIGATOR_FRONTEND_TIMEOUT_MS} ms`,
        );

        if (navigatorRequestIdRef.current !== requestId) {
          return;
        }

        setNavigatorTree(tree);
        onClearError();
      } catch (error) {
        if (navigatorRequestIdRef.current !== requestId) {
          return;
        }

        const message = formatError(error);
        setNavigatorTree(null);
        setNavigatorError(message);
        onError(error);
      } finally {
        if (navigatorRequestIdRef.current === requestId) {
          setNavigatorLoading(false);
        }
      }
    },
    [onClearError, onError, selectedConnectionId],
  );

  useEffect(() => {
    if (!selectedConnectionId) {
      setNavigatorTree(null);
      setNavigatorError(null);
      return;
    }
    void refreshNavigator(selectedConnectionId);
  }, [refreshNavigator, selectedConnectionId]);

  const navigatorObjectCount = useMemo(() => {
    if (!navigatorTree) {
      return 0;
    }
    return navigatorTree.schemas.reduce(
      (acc, schema) => acc + schema.tables.length + schema.views.length,
      0,
    );
  }, [navigatorTree]);

  const insertNavigatorSql = useCallback(
    (sql: string, message: string) => {
      onSetSqlText((previous) => (previous.trim() ? `${previous.trimEnd()}\n\n${sql}` : sql));
      onStatus(message);
      onClearError();
    },
    [onClearError, onSetSqlText, onStatus],
  );

  const handleNavigatorAction = useCallback(
    async (request: NavigatorActionRequest) => {
      const { schemaName, object, action } = request;
      const engine = selectedConnectionEngine;
      const qualifiedName = buildQualifiedObjectName(engine, schemaName, object.name);

      try {
        if (action === "copy_name") {
          await copyTextToClipboard(qualifiedName);
          onStatus(`Copied object name ${qualifiedName}.`);
          onClearError();
          return;
        }

        if (action === "generate_select" || action === "open_data") {
          const selectSql = buildSelectFromObjectSql(
            engine,
            schemaName,
            object.name,
            queryPageSize,
          );

          if (action === "generate_select") {
            insertNavigatorSql(selectSql, `Generated SELECT for ${qualifiedName}.`);
            return;
          }

          onSetSqlText(selectSql);
          onStatus(`Running SELECT for ${qualifiedName}...`);
          onClearError();
          onResetQueryGridModifiers();
          await onRunQueryPage(selectSql, 0, "navigator", {
            gridModifiersOverride: DEFAULT_QUERY_GRID_MODIFIERS,
          });
          return;
        }

        if (action === "generate_insert") {
          const insertSql = buildInsertTemplateSql(engine, schemaName, object);
          insertNavigatorSql(insertSql, `Generated INSERT for ${qualifiedName}.`);
          return;
        }

        if (action === "generate_update") {
          const updateSql = buildUpdateTemplateSql(engine, schemaName, object);
          insertNavigatorSql(updateSql, `Generated UPDATE for ${qualifiedName}.`);
          return;
        }

        if (action === "generate_ddl") {
          const ddlSql = buildDdlTemplateSql(engine, schemaName, object);
          insertNavigatorSql(ddlSql, `Generated DDL template for ${qualifiedName}.`);
        }
      } catch (error) {
        onError(error);
      }
    },
    [
      insertNavigatorSql,
      onClearError,
      onError,
      onResetQueryGridModifiers,
      onRunQueryPage,
      onSetSqlText,
      onStatus,
      queryPageSize,
      selectedConnectionEngine,
    ],
  );

  return {
    navigatorTree,
    navigatorLoading,
    navigatorError,
    navigatorObjectCount,
    refreshNavigator,
    handleNavigatorAction,
  };
}

async function withPromiseTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  message: string,
): Promise<T> {
  let timeoutHandle: ReturnType<typeof window.setTimeout> | undefined;

  try {
    const timeoutPromise = new Promise<T>((_, reject) => {
      timeoutHandle = window.setTimeout(() => {
        reject(new Error(message));
      }, timeoutMs);
    });

    return await Promise.race([promise, timeoutPromise]);
  } finally {
    if (timeoutHandle) {
      window.clearTimeout(timeoutHandle);
    }
  }
}

function formatError(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "Unexpected error";
}
