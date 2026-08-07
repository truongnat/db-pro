import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { I18nextProvider, initReactI18next } from "react-i18next";
import i18n from "i18next";

import { ConnectionDialog } from "../components/connection-dialog";
import { useRecentStore } from "@/commons/stores/recent.store";
import * as queries from "../queries/connection.queries";
import type { Connection } from "../types/connection.types";

vi.mock("../queries/connection.queries", () => ({
  useCreateConnection: vi.fn(),
  useUpdateConnection: vi.fn(),
  useTestConnection: vi.fn(),
  useConnect: vi.fn(),
}));

const getConnectionMock = vi.fn();

vi.mock("@/app/app.module", () => ({
  container: {
    resolve: vi.fn(() => ({
      get: getConnectionMock,
    })),
  },
}));

vi.mock("@/commons/di/registry", () => ({
  SERVICE_NAMES: { CONNECTION_SERVICE: "ConnectionService" },
}));

vi.mock("@/app/providers/snackbar.provider", () => ({
  useSnackbar: vi.fn(() => ({
    success: vi.fn(),
    error: vi.fn(),
  })),
}));

vi.mock("@/lib/utils", () => ({
  cn: (...args: unknown[]) => args.filter(Boolean).join(" "),
}));

i18n.use(initReactI18next).init({
  resources: {
    en: {
      translation: {
        common: {
          states: { loading: "Loading..." },
          actions: {
            save: "Save",
            cancel: "Cancel",
            retry: "Retry",
            close: "Close",
          },
        },
        connection: {
          new: "New Connection",
          edit: "Edit Connection",
          saveAndConnect: "Save & Connect",
          connectFailed: "Failed to connect",
          loadFailed: "Unable to load connection",
          notFound: "Connection not found",
        },
      },
    },
  },
  lng: "en",
  fallbackLng: "en",
});

function createQueryClient() {
  return new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
}

function renderDialog() {
  const qc = createQueryClient();
  return render(
    <I18nextProvider i18n={i18n}>
      <QueryClientProvider client={qc}>
        <ConnectionDialog />
      </QueryClientProvider>
    </I18nextProvider>,
  );
}

const fakeConnection: Connection = {
  id: "new-1",
  name: "New Conn",
  host: "localhost",
  port: 5432,
  database: "demo",
  username: "dbpro",
  driver: "postgres",
  sslMode: "disable",
  color: null,
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
};

function mockDefaultQueries() {
  const createMutate = vi.fn();
  const updateMutate = vi.fn();
  const testMutate = vi.fn();
  const connectMutate = vi.fn();

  vi.mocked(queries.useCreateConnection).mockReturnValue({
    mutate: createMutate,
    isPending: false,
  } as unknown as ReturnType<typeof queries.useCreateConnection>);
  vi.mocked(queries.useUpdateConnection).mockReturnValue({
    mutate: updateMutate,
    isPending: false,
  } as unknown as ReturnType<typeof queries.useUpdateConnection>);
  vi.mocked(queries.useTestConnection).mockReturnValue({
    mutate: testMutate,
    isPending: false,
    isSuccess: false,
    isError: false,
  } as unknown as ReturnType<typeof queries.useTestConnection>);
  vi.mocked(queries.useConnect).mockReturnValue({
    mutate: connectMutate,
    isPending: false,
  } as unknown as ReturnType<typeof queries.useConnect>);

  return { createMutate, updateMutate, testMutate, connectMutate };
}

describe("ConnectionDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    getConnectionMock.mockResolvedValue(null);
    useRecentStore.setState({
      recentConnections: [],
      connectionDialogOpen: false,
      connectionDialogEditId: null,
    });
    mockDefaultQueries();
  });

  describe("rendering", () => {
    it("does not render when closed", () => {
      renderDialog();
      expect(screen.queryByText("New Connection")).not.toBeInTheDocument();
    });

    it("renders new-connection title when opened without edit id", async () => {
      useRecentStore.getState().openConnectionDialog();
      renderDialog();
      expect(await screen.findByText("New Connection")).toBeInTheDocument();
    });

    it("renders edit title when opened with an edit id", async () => {
      getConnectionMock.mockResolvedValue(fakeConnection);
      useRecentStore.getState().openConnectionDialog("new-1");
      renderDialog();
      expect(await screen.findByText("Edit Connection")).toBeInTheDocument();
    });
  });

  describe("load recovery", () => {
    it("shows error UI instead of a blank editor when load fails", async () => {
      getConnectionMock.mockRejectedValue(new Error("boom"));
      useRecentStore.getState().openConnectionDialog("new-1");
      renderDialog();

      expect(await screen.findByText("Unable to load connection")).toBeInTheDocument();
      expect(screen.getByText("Retry")).toBeInTheDocument();
      expect(screen.getAllByText("Close").length).toBeGreaterThan(0);
      expect(screen.queryByText("Save & Connect")).not.toBeInTheDocument();
    });

    it("shows not-found message when connection is missing", async () => {
      getConnectionMock.mockResolvedValue(null);
      useRecentStore.getState().openConnectionDialog("missing-1");
      renderDialog();

      expect(await screen.findByText("Connection not found")).toBeInTheDocument();
      expect(screen.queryByText("Save & Connect")).not.toBeInTheDocument();
    });
  });

  describe("save & connect lifecycle", () => {
    it("Save only persists and closes without connecting", async () => {
      const { createMutate, connectMutate } = mockDefaultQueries();
      const user = userEvent.setup();
      useRecentStore.getState().openConnectionDialog();
      renderDialog();

      await user.click(await screen.findByText("Save"));

      expect(createMutate).toHaveBeenCalledTimes(1);
      const createOptions = createMutate.mock.calls[0][1];
      createOptions.onSuccess(fakeConnection);

      expect(connectMutate).not.toHaveBeenCalled();
      expect(useRecentStore.getState().connectionDialogOpen).toBe(false);
    });

    it("Save & Connect connects with the created id then closes on success", async () => {
      const { createMutate, connectMutate } = mockDefaultQueries();
      const user = userEvent.setup();
      useRecentStore.getState().openConnectionDialog();
      renderDialog();

      await user.click(await screen.findByText("Save & Connect"));

      expect(createMutate).toHaveBeenCalledTimes(1);
      const createOptions = createMutate.mock.calls[0][1];
      createOptions.onSuccess(fakeConnection);

      expect(connectMutate).toHaveBeenCalledTimes(1);
      expect(connectMutate.mock.calls[0][0]).toBe("new-1");

      const connectOptions = connectMutate.mock.calls[0][1];
      connectOptions.onSuccess();

      expect(useRecentStore.getState().connectionDialogOpen).toBe(false);
    });

    it("keeps dialog open and shows connect error when connect fails", async () => {
      const { createMutate, connectMutate } = mockDefaultQueries();
      const user = userEvent.setup();
      useRecentStore.getState().openConnectionDialog();
      renderDialog();

      await user.click(await screen.findByText("Save & Connect"));
      createMutate.mock.calls[0][1].onSuccess(fakeConnection);

      connectMutate.mock.calls[0][1].onError({ userMessage: "bad password" });

      expect(useRecentStore.getState().connectionDialogOpen).toBe(true);
      expect(await screen.findByText("bad password")).toBeInTheDocument();
    });

    it("edit mode updates then connects with the existing id", async () => {
      const { updateMutate, connectMutate } = mockDefaultQueries();
      const user = userEvent.setup();
      getConnectionMock.mockResolvedValue(fakeConnection);
      useRecentStore.getState().openConnectionDialog("new-1");
      renderDialog();

      await user.click(await screen.findByText("Save & Connect"));

      expect(updateMutate).toHaveBeenCalledTimes(1);
      expect(updateMutate.mock.calls[0][0].id).toBe("new-1");
      updateMutate.mock.calls[0][1].onSuccess();

      expect(connectMutate.mock.calls[0][0]).toBe("new-1");
    });
  });
});
