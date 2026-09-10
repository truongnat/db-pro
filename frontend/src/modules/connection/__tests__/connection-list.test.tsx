import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { I18nextProvider, initReactI18next } from "react-i18next";
import i18n from "i18next";

import { ConnectionList } from "../components/connection-list";
import * as queries from "../queries/connection.queries";

vi.mock("../queries/connection.queries", () => ({
  useConnectionList: vi.fn(),
  useConnect: vi.fn(() => ({ mutate: vi.fn(), isPending: false })),
  useDisconnect: vi.fn(() => ({ mutate: vi.fn(), isPending: false })),
  useDeleteConnection: vi.fn(() => ({ mutate: vi.fn(), isPending: false })),
  useDuplicateConnection: vi.fn(() => ({ mutate: vi.fn(), isPending: false })),
  useRenameConnection: vi.fn(() => ({ mutate: vi.fn(), isPending: false })),
  useToggleFavorite: vi.fn(() => ({ mutate: vi.fn(), isPending: false })),
  useToggleReadonly: vi.fn(() => ({ mutate: vi.fn(), isPending: false })),
}));

vi.mock("@/commons/stores/connection.store", () => ({
  useConnectionStore: vi.fn((selector) => selector({ explorerConnectionId: null })),
}));

vi.mock("../state/connection.store", () => ({
  useConnectionModuleStore: vi.fn((selector) =>
    selector({
      statuses: {},
      connectionErrors: {},
      favorites: {},
      sortField: "name",
      sortDirection: "asc",
      filterTag: null,
      filterGroup: null,
      setSortField: vi.fn(),
      setSortDirection: vi.fn(),
      setFilterTag: vi.fn(),
      setFilterGroup: vi.fn(),
      clearFilters: vi.fn(),
    }),
  ),
}));

vi.mock("@/lib/utils", () => ({
  cn: (...args: unknown[]) => args.filter(Boolean).join(" "),
}));

i18n.use(initReactI18next).init({
  resources: {
    en: {
      translation: {
        common: {
          states: {
            loading: "Loading...",
            empty: "No data",
            error: "Error",
            connected: "Connected",
            disconnected: "Disconnected",
          },
          labels: { name: "Name", host: "Host", database: "Database", driver: "Driver" },
          actions: {
            delete: "Delete",
            connect: "Connect",
            disconnect: "Disconnect",
            edit: "Edit",
            clear: "Clear All",
            sort: "Sort",
          },
        },
        connection: {
          edit: "Edit Connection",
          group: "Group",
          tags: "Tags",
          confirmDelete: "Delete?",
          toggleFavorite: "Toggle favorite",
          readonly: "Read-only",
          sort: { name: "Name", driver: "Driver", group: "Group" },
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
    id: "1",
    name: "Local PG",
    host: "localhost",
    port: 5432,
    database: "mydb",
    username: "user",
    driver: "postgres" as const,
    sslMode: "disable" as const,
    createdAt: "2026-01-01T00:00:00Z",
    updatedAt: "2026-01-01T00:00:00Z",
  },
];

describe("ConnectionList", () => {
  const onEdit = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    onEdit.mockReset();
  });

  it("shows loading state", () => {
    vi.mocked(queries.useConnectionList).mockReturnValue({
      data: undefined,
      isLoading: true,
      error: null,
    } as ReturnType<typeof queries.useConnectionList>);

    renderWithProviders(<ConnectionList onEdit={onEdit} />);
    expect(screen.getByText("Loading...")).toBeInTheDocument();
  });

  it("shows empty state when no connections", () => {
    vi.mocked(queries.useConnectionList).mockReturnValue({
      data: [],
      isLoading: false,
      error: null,
    } as ReturnType<typeof queries.useConnectionList>);

    renderWithProviders(<ConnectionList onEdit={onEdit} />);
    expect(screen.getByText("No data")).toBeInTheDocument();
  });

  it("renders connection rows", () => {
    vi.mocked(queries.useConnectionList).mockReturnValue({
      data: mockConnections,
      isLoading: false,
      error: null,
    } as ReturnType<typeof queries.useConnectionList>);

    renderWithProviders(<ConnectionList onEdit={onEdit} />);
    expect(screen.getByText("Local PG")).toBeInTheDocument();
    expect(screen.getByText("localhost:5432")).toBeInTheDocument();
    expect(screen.getByText("mydb")).toBeInTheDocument();
  });

  it("calls onEdit when clicking a row", async () => {
    vi.mocked(queries.useConnectionList).mockReturnValue({
      data: mockConnections,
      isLoading: false,
      error: null,
    } as ReturnType<typeof queries.useConnectionList>);

    const user = userEvent.setup();
    renderWithProviders(<ConnectionList onEdit={onEdit} />);

    await user.click(screen.getByText("Local PG"));
    expect(onEdit).toHaveBeenCalledWith("1");
  });

  it("shows error state", () => {
    vi.mocked(queries.useConnectionList).mockReturnValue({
      data: undefined,
      isLoading: false,
      error: { userMessage: "Server error" },
    } as unknown as ReturnType<typeof queries.useConnectionList>);

    renderWithProviders(<ConnectionList onEdit={onEdit} />);
    expect(screen.getByText("Error")).toBeInTheDocument();
    expect(screen.getByText("Server error")).toBeInTheDocument();
  });
});
