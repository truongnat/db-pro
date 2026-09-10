import { describe, expect, it, vi, beforeEach } from "vitest";
import { act, renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";

import { useConnect, useToggleFavorite } from "../queries/connection.queries";
import { useRecentStore } from "@/commons/stores/recent.store";
import { useConnectionModuleStore } from "../state/connection.store";

const connectMock = vi.fn();
const getMock = vi.fn();
const updateMock = vi.fn();

vi.mock("@/app/app.module", () => ({
  container: {
    resolve: vi.fn(() => ({
      connect: connectMock,
      get: getMock,
      update: updateMock,
    })),
  },
}));

function wrapper({ children }: { children: ReactNode }) {
  const qc = new QueryClient({ defaultOptions: { mutations: { retry: false } } });
  return <QueryClientProvider client={qc}>{children}</QueryClientProvider>;
}

function resetStore() {
  useRecentStore.setState({
    recentConnections: [],
    connectionDialogOpen: false,
    connectionDialogEditId: null,
  });
}

describe("useConnect recent tracking", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetStore();
    connectMock.mockResolvedValue(undefined);
  });

  it("records a recent entry on successful connect", async () => {
    const { result } = renderHook(() => useConnect(), { wrapper });

    act(() => result.current.mutate("conn-1"));

    await waitFor(() => expect(connectMock).toHaveBeenCalledWith("conn-1"));

    await waitFor(() => {
      expect(useRecentStore.getState().recentConnections).toHaveLength(1);
      expect(useRecentStore.getState().recentConnections[0].connectionId).toBe("conn-1");
    });
  });

  it("reconnects do not duplicate; they reorder and increment count", async () => {
    const { result } = renderHook(() => useConnect(), { wrapper });

    act(() => result.current.mutate("conn-1"));
    await waitFor(() => expect(useRecentStore.getState().recentConnections).toHaveLength(1));

    act(() => result.current.mutate("conn-2"));
    await waitFor(() => expect(useRecentStore.getState().recentConnections).toHaveLength(2));

    act(() => result.current.mutate("conn-1"));
    await waitFor(() => {
      const recent = useRecentStore.getState().recentConnections;
      expect(recent).toHaveLength(2);
      expect(recent[0].connectionId).toBe("conn-1");
      expect(recent[0].connectCount).toBe(2);
    });
  });

  it("does not record recent when connect fails", async () => {
    connectMock.mockRejectedValue(new Error("auth"));
    const { result } = renderHook(() => useConnect(), { wrapper });

    act(() => result.current.mutate("conn-1"));

    await waitFor(() => expect(connectMock).toHaveBeenCalled());

    expect(useRecentStore.getState().recentConnections).toHaveLength(0);
  });
});

describe("useToggleFavorite optimistic rollback", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useConnectionModuleStore.setState({ favorites: {} });
  });

  it("reverts optimistic favorite toggle when update fails", async () => {
    getMock.mockResolvedValue({
      id: "conn-1",
      name: "Test DB",
      driver: "postgres",
      favorite: false,
    });
    updateMock.mockRejectedValue(new Error("Database write error"));

    const { result } = renderHook(() => useToggleFavorite(), { wrapper });

    // Initial state: not favorited
    expect(useConnectionModuleStore.getState().favorites["conn-1"]).toBeFalsy();

    // Trigger mutation
    act(() => result.current.mutate({ id: "conn-1", favorite: true }));

    // Optimistically favorited immediately
    expect(useConnectionModuleStore.getState().favorites["conn-1"]).toBe(true);

    // After failure, onError rolls back favorite state
    await waitFor(() => {
      expect(useConnectionModuleStore.getState().favorites["conn-1"]).toBe(false);
    });
  });
});
