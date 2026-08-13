import { describe, expect, it, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";

import { useConnectionList, __resetSessionRestored } from "../queries/connection.queries";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useConnectionModuleStore } from "../state/connection.store";

const listMock = vi.fn();
const connectMock = vi.fn();

vi.mock("@/app/app.module", () => ({
  container: {
    resolve: vi.fn(() => ({
      list: listMock,
      connect: connectMock,
    })),
  },
}));

vi.mock("@/commons/di/registry", () => ({
  SERVICE_NAMES: { CONNECTION_SERVICE: "ConnectionService" },
}));

vi.mock("@/commons/stores/workspace.store", () => ({
  reconcileWorkspaceTabs: vi.fn(),
}));

function wrapper({ children }: { children: ReactNode }) {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return <QueryClientProvider client={qc}>{children}</QueryClientProvider>;
}

describe("QA-P1-09 session restoration", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    __resetSessionRestored();
    useConnectionStore.setState({
      activeConnectionIds: [],
      explorerConnectionId: null,
    });
    useConnectionModuleStore.setState({
      statuses: {},
      errors: {},
    });
    listMock.mockResolvedValue([]);
    connectMock.mockResolvedValue(undefined);
  });

  it("restoreSession runs only once on first fetch", async () => {
    useConnectionStore.setState({ activeConnectionIds: ["conn-1"] });
    listMock.mockResolvedValue([
      { id: "conn-1", name: "Test", driver: "postgres" },
    ]);

    const { result } = renderHook(() => useConnectionList(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    await waitFor(() => expect(connectMock).toHaveBeenCalledTimes(1));

    // Simulate query invalidation and refetch
    result.current.refetch();
    await waitFor(() => expect(result.current.isFetching).toBe(false));

    // connect should still only have been called once
    expect(connectMock).toHaveBeenCalledTimes(1);
  });

  it("does not attempt reconnect for stale connection IDs", async () => {
    useConnectionStore.setState({ activeConnectionIds: ["stale-id"] });
    listMock.mockResolvedValue([
      { id: "conn-1", name: "Test", driver: "postgres" },
    ]);

    const { result } = renderHook(() => useConnectionList(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    // stale-id should be cleaned up and not attempted
    expect(connectMock).not.toHaveBeenCalled();
    expect(useConnectionStore.getState().activeConnectionIds).not.toContain("stale-id");
  });

  it("failed reconnect does not block other reconnects", async () => {
    useConnectionStore.setState({ activeConnectionIds: ["conn-1", "conn-2"] });
    listMock.mockResolvedValue([
      { id: "conn-1", name: "Test 1", driver: "postgres" },
      { id: "conn-2", name: "Test 2", driver: "postgres" },
    ]);
    connectMock
      .mockRejectedValueOnce(new Error("auth failed"))
      .mockResolvedValueOnce(undefined);

    const { result } = renderHook(() => useConnectionList(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    await waitFor(() => expect(connectMock).toHaveBeenCalledTimes(2));

    // Both should have been attempted despite first failure
    expect(useConnectionModuleStore.getState().statuses["conn-1"]).toBe("error");
    expect(useConnectionModuleStore.getState().statuses["conn-2"]).toBe("connected");
  });
});
