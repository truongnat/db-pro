import { describe, expect, it, vi, beforeEach } from "vitest";
import { act, renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";

import {
  useConnectionList,
  useCreateConnection,
  useDeleteConnection,
  useTestConnection,
  useDuplicateConnection,
  useRenameConnection,
  useToggleFavorite,
  useToggleReadonly,
} from "../queries/connection.queries";

const {
  mockList,
  mockCreate,
  mockGet,
  mockUpdate,
  mockDelete,
  mockTest,
} = vi.hoisted(() => ({
  mockList: vi.fn(),
  mockCreate: vi.fn(),
  mockGet: vi.fn(),
  mockUpdate: vi.fn(),
  mockDelete: vi.fn(),
  mockTest: vi.fn(),
}));

vi.mock("@/app/app.module", () => ({
  container: {
    resolve: vi.fn(() => ({
      list: mockList,
      create: mockCreate,
      get: mockGet,
      update: mockUpdate,
      delete: mockDelete,
      test: mockTest,
    })),
  },
}));

function wrapper({ children }: { children: ReactNode }) {
  const qc = new QueryClient({ defaultOptions: { mutations: { retry: false } } });
  return <QueryClientProvider client={qc}>{children}</QueryClientProvider>;
}

const sampleConnection = {
  id: "conn-1",
  name: "Test DB",
  host: "localhost",
  port: 5432,
  database: "testdb",
  username: "user",
  driver: "postgres" as const,
  sslMode: "disable" as const,
  queryTimeoutMs: 30000,
  maxRows: 500,
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
};

describe("useConnectionList", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetches connections via service.list", async () => {
    mockList.mockResolvedValue([sampleConnection]);
    const { result } = renderHook(() => useConnectionList(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual([sampleConnection]);
  });
});

describe("useCreateConnection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("calls service.create with config and password", async () => {
    mockCreate.mockResolvedValue(sampleConnection);
    const { result } = renderHook(() => useCreateConnection(), { wrapper });

    const config = { ...sampleConnection };
    act(() => result.current.mutate({ config, password: "secret" }));

    await waitFor(() => expect(mockCreate).toHaveBeenCalledWith(config, "secret"));
  });
});

describe("useDeleteConnection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("calls service.delete with id", async () => {
    mockDelete.mockResolvedValue(undefined);
    const { result } = renderHook(() => useDeleteConnection(), { wrapper });

    act(() => result.current.mutate("conn-1"));

    await waitFor(() => expect(mockDelete).toHaveBeenCalledWith("conn-1"));
  });
});

describe("useTestConnection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("calls service.test with config, password, and optional connectionId", async () => {
    mockTest.mockResolvedValue(undefined);
    const { result } = renderHook(() => useTestConnection(), { wrapper });

    const config = { ...sampleConnection };
    act(() => result.current.mutate({ config, password: "pw", connectionId: "conn-1" }));

    await waitFor(() =>
      expect(mockTest).toHaveBeenCalledWith(config, "pw", "conn-1"),
    );
  });
});

describe("useDuplicateConnection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetches source, creates copy with '(copy)' suffix", async () => {
    mockGet.mockResolvedValue(sampleConnection);
    mockCreate.mockResolvedValue({ ...sampleConnection, id: "conn-2", name: "Test DB (copy)" });

    const { result } = renderHook(() => useDuplicateConnection(), { wrapper });
    act(() => result.current.mutate("conn-1"));

    await waitFor(() => {
      expect(mockGet).toHaveBeenCalledWith("conn-1");
      expect(mockCreate).toHaveBeenCalled();
      const createConfig = mockCreate.mock.calls[0][0];
      expect(createConfig.name).toBe("Test DB (copy)");
      expect(createConfig.host).toBe("localhost");
    });
  });

  it("throws when source connection not found", async () => {
    mockGet.mockResolvedValue(null);
    const { result } = renderHook(() => useDuplicateConnection(), { wrapper });

    act(() => result.current.mutate("nonexistent"));

    await waitFor(() => expect(result.current.isError).toBe(true));
  });
});

describe("useRenameConnection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetches source and updates with new name", async () => {
    mockGet.mockResolvedValue(sampleConnection);
    mockUpdate.mockResolvedValue(undefined);

    const { result } = renderHook(() => useRenameConnection(), { wrapper });
    act(() => result.current.mutate({ id: "conn-1", name: "Renamed DB" }));

    await waitFor(() => {
      expect(mockUpdate).toHaveBeenCalled();
      const updateConfig = mockUpdate.mock.calls[0][1];
      expect(updateConfig.name).toBe("Renamed DB");
    });
  });
});

describe("useToggleFavorite", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("updates connection with favorite flag", async () => {
    mockGet.mockResolvedValue(sampleConnection);
    mockUpdate.mockResolvedValue(undefined);

    const { result } = renderHook(() => useToggleFavorite(), { wrapper });
    act(() => result.current.mutate({ id: "conn-1", favorite: true }));

    await waitFor(() => {
      expect(mockUpdate).toHaveBeenCalled();
      const updateConfig = mockUpdate.mock.calls[0][1];
      expect(updateConfig.favorite).toBe(true);
    });
  });
});

describe("useToggleReadonly", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("updates connection with readonly flag", async () => {
    mockGet.mockResolvedValue(sampleConnection);
    mockUpdate.mockResolvedValue(undefined);

    const { result } = renderHook(() => useToggleReadonly(), { wrapper });
    act(() => result.current.mutate({ id: "conn-1", readonly: true }));

    await waitFor(() => {
      expect(mockUpdate).toHaveBeenCalled();
      const updateConfig = mockUpdate.mock.calls[0][1];
      expect(updateConfig.readonly).toBe(true);
    });
  });
});
