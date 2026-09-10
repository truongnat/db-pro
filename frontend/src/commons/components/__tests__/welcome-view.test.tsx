import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { I18nextProvider, initReactI18next } from "react-i18next";
import i18n from "i18next";

import { WelcomeView } from "../welcome-view";
import { useRecentStore } from "@/commons/stores/recent.store";
import * as connectionQueries from "@/modules/connection/queries/connection.queries";
import * as snackbarProvider from "@/app/providers/snackbar.provider";

vi.mock("@/modules/connection/queries/connection.queries", () => ({
  useConnectionList: vi.fn(),
  useConnect: vi.fn(),
  useDeleteConnection: vi.fn(),
  useCreateConnection: vi.fn(),
  useUpdateConnection: vi.fn(),
  useTestConnection: vi.fn(),
}));

vi.mock("@/app/providers/snackbar.provider", () => ({
  useSnackbar: vi.fn(() => ({
    success: vi.fn(),
    error: vi.fn(),
  })),
}));

vi.mock("@/app/app.module", () => ({
  container: {
    resolve: vi.fn(() => ({
      get: vi.fn().mockResolvedValue(null),
    })),
  },
}));

vi.mock("@/commons/di/registry", () => ({
  SERVICE_NAMES: { CONNECTION_SERVICE: "ConnectionService" },
}));

vi.mock("@/modules/connection/state/connection.store", () => ({
  useConnectionModuleStore: vi.fn((selector) => selector({ statuses: {}, connectionErrors: {} })),
}));

vi.mock("@/commons/stores/connection.store", () => ({
  useConnectionStore: vi.fn((selector) => selector({ explorerConnectionId: null })),
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
          actions: { delete: "Delete", cancel: "Cancel" },
        },
        connection: {
          confirmDelete: "Are you sure?",
          confirmDeleteDescription: "This cannot be undone.",
          connectFailed: "Failed to connect",
        },
        welcome: {
          title: "DB Pro",
          subtitle: "Database Management Made Simple",
          newConnection: "New Connection",
          openCommandPalette: "Command Palette",
          recentConnections: "Recent Connections",
          noConnections: "No connections yet",
          createFirstConnection: "Create your first connection to get started",
          noRecentConnections: "No recent connections",
          connectHint: "Click a connection to connect",
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

function renderWithProviders(ui: React.ReactElement) {
  const qc = createQueryClient();
  return render(
    <I18nextProvider i18n={i18n}>
      <QueryClientProvider client={qc}>{ui}</QueryClientProvider>
    </I18nextProvider>,
  );
}

const mockConnections = [
  {
    id: "conn-1",
    name: "Local PG",
    host: "localhost",
    port: 5432,
    database: "mydb",
    username: "user",
    driver: "postgres" as const,
    sslMode: "disable" as const,
    color: "#3b82f6",
    createdAt: "2026-01-01T00:00:00Z",
    updatedAt: "2026-01-01T00:00:00Z",
  },
];

describe("WelcomeView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useRecentStore.setState({
      recentConnections: [],
      connectionDialogOpen: false,
      connectionDialogEditId: null,
    });
    vi.mocked(connectionQueries.useConnect).mockReturnValue({
      mutate: vi.fn(),
    } as unknown as ReturnType<typeof connectionQueries.useConnect>);
    vi.mocked(connectionQueries.useDeleteConnection).mockReturnValue({
      mutate: vi.fn(),
    } as unknown as ReturnType<typeof connectionQueries.useDeleteConnection>);
    vi.mocked(connectionQueries.useCreateConnection).mockReturnValue({
      mutate: vi.fn(),
      isPending: false,
    } as unknown as ReturnType<typeof connectionQueries.useCreateConnection>);
    vi.mocked(connectionQueries.useUpdateConnection).mockReturnValue({
      mutate: vi.fn(),
      isPending: false,
    } as unknown as ReturnType<typeof connectionQueries.useUpdateConnection>);
    vi.mocked(connectionQueries.useTestConnection).mockReturnValue({
      mutate: vi.fn(),
      isPending: false,
      isSuccess: false,
      isError: false,
    } as unknown as ReturnType<typeof connectionQueries.useTestConnection>);
  });

  it("renders quick action buttons", () => {
    vi.mocked(connectionQueries.useConnectionList).mockReturnValue({
      data: [],
      isLoading: false,
    } as ReturnType<typeof connectionQueries.useConnectionList>);

    renderWithProviders(<WelcomeView />);
    expect(screen.getByText("New Connection")).toBeInTheDocument();
    expect(screen.getByText("Command Palette")).toBeInTheDocument();
  });

  it("shows empty state when no connections exist", () => {
    vi.mocked(connectionQueries.useConnectionList).mockReturnValue({
      data: [],
      isLoading: false,
    } as ReturnType<typeof connectionQueries.useConnectionList>);

    renderWithProviders(<WelcomeView />);
    expect(screen.getByText("No connections yet")).toBeInTheDocument();
    expect(screen.getByText("Create your first connection to get started")).toBeInTheDocument();
  });

  it("shows recent connections list when data available", () => {
    vi.mocked(connectionQueries.useConnectionList).mockReturnValue({
      data: mockConnections,
      isLoading: false,
    } as ReturnType<typeof connectionQueries.useConnectionList>);

    useRecentStore.setState({
      recentConnections: [
        { connectionId: "conn-1", lastConnectedAt: "2026-01-01T00:00:00Z", connectCount: 1 },
      ],
    });

    renderWithProviders(<WelcomeView />);
    expect(screen.getByText("Local PG")).toBeInTheDocument();
    expect(screen.getByText("localhost:5432 / mydb")).toBeInTheDocument();

    useRecentStore.setState({ recentConnections: [] });
  });

  it("clicking New Connection opens the connection dialog", async () => {
    vi.mocked(connectionQueries.useConnectionList).mockReturnValue({
      data: [],
      isLoading: false,
    } as ReturnType<typeof connectionQueries.useConnectionList>);

    const user = userEvent.setup();
    renderWithProviders(<WelcomeView />);

    await user.click(screen.getByText("New Connection"));

    expect(useRecentStore.getState().connectionDialogOpen).toBe(true);

    useRecentStore.getState().closeConnectionDialog();
  });

  it("shows loading state", () => {
    vi.mocked(connectionQueries.useConnectionList).mockReturnValue({
      data: undefined,
      isLoading: true,
    } as ReturnType<typeof connectionQueries.useConnectionList>);

    renderWithProviders(<WelcomeView />);
    expect(screen.getByText("Loading...")).toBeInTheDocument();
  });

  it("prunes stale recent entries when connections data loads", () => {
    useRecentStore.setState({
      recentConnections: [
        { connectionId: "conn-1", lastConnectedAt: "2026-01-01T00:00:00Z", connectCount: 1 },
        { connectionId: "deleted-conn", lastConnectedAt: "2026-01-01T00:00:00Z", connectCount: 3 },
      ],
    });

    vi.mocked(connectionQueries.useConnectionList).mockReturnValue({
      data: mockConnections,
      isLoading: false,
    } as ReturnType<typeof connectionQueries.useConnectionList>);

    renderWithProviders(<WelcomeView />);

    const remaining = useRecentStore.getState().recentConnections;
    expect(remaining).toHaveLength(1);
    expect(remaining[0].connectionId).toBe("conn-1");
  });

  it("shows error snackbar when connection fails", () => {
    const mockError = vi.fn();
    vi.mocked(snackbarProvider.useSnackbar).mockReturnValueOnce({
      success: vi.fn(),
      error: mockError,
      warning: vi.fn(),
      info: vi.fn(),
    });

    vi.mocked(connectionQueries.useConnect).mockReturnValue({
      mutate: vi.fn((_id: string, opts?: { onError?: (err: unknown) => void }) => {
        opts?.onError?.({ userMessage: "Connection refused" });
      }),
    } as unknown as ReturnType<typeof connectionQueries.useConnect>);

    vi.mocked(connectionQueries.useConnectionList).mockReturnValue({
      data: mockConnections,
      isLoading: false,
    } as ReturnType<typeof connectionQueries.useConnectionList>);

    useRecentStore.setState({
      recentConnections: [
        { connectionId: "conn-1", lastConnectedAt: "2026-01-01T00:00:00Z", connectCount: 1 },
      ],
    });

    renderWithProviders(<WelcomeView />);
    const connectButton = screen.getByText("Local PG").closest("button");
    expect(connectButton).toBeTruthy();
    connectButton!.click();

    expect(mockError).toHaveBeenCalledWith("Connection refused");
  });
});
