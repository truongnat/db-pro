import { describe, expect, it, vi, beforeEach } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { I18nextProvider, initReactI18next } from "react-i18next";
import i18n from "i18next";

import { ConnectionList } from "../components/connection-list";
import * as queries from "../queries/connection.queries";
import { ConfirmDialogProvider } from "@/app/providers/confirm-dialog.provider";

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

const snackbar = { success: vi.fn(), error: vi.fn(), warning: vi.fn(), info: vi.fn() };

vi.mock("@/app/providers/snackbar.provider", () => ({
  useSnackbar: () => snackbar,
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
            confirm: "Confirm",
            cancel: "Cancel",
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
          duplicate: "Duplicate",
          duplicateCredentialsNotice: "Credentials were not copied",
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
      <QueryClientProvider client={qc}>
        <ConfirmDialogProvider>{ui}</ConfirmDialogProvider>
      </QueryClientProvider>
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

const deleteConnection = vi.fn();

describe("ConnectionList", () => {
  const onEdit = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    onEdit.mockReset();
    vi.mocked(queries.useDeleteConnection).mockReturnValue({
      mutate: deleteConnection,
      isPending: false,
    } as ReturnType<typeof queries.useDeleteConnection>);
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

  it("uses the app confirmation dialog before deleting", async () => {
    vi.mocked(queries.useConnectionList).mockReturnValue({
      data: mockConnections,
      isLoading: false,
      error: null,
    } as ReturnType<typeof queries.useConnectionList>);

    const user = userEvent.setup();
    renderWithProviders(<ConnectionList onEdit={onEdit} />);

    await user.click(screen.getByRole("button", { name: "Delete" }));
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    expect(screen.getByText("Delete?")).toBeInTheDocument();
    expect(deleteConnection).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "Confirm" }));
    expect(deleteConnection).toHaveBeenCalledWith("1");
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

  it("explains that duplicated credentials must be added manually", async () => {
    const duplicate = vi.fn((_id: string, options?: { onSuccess?: () => void }) => {
      options?.onSuccess?.();
    });
    vi.mocked(queries.useDuplicateConnection).mockReturnValue({
      mutate: duplicate,
      isPending: false,
    } as ReturnType<typeof queries.useDuplicateConnection>);
    vi.mocked(queries.useConnectionList).mockReturnValue({
      data: mockConnections,
      isLoading: false,
      error: null,
    } as ReturnType<typeof queries.useConnectionList>);

    const user = userEvent.setup();
    renderWithProviders(<ConnectionList onEdit={onEdit} />);
    fireEvent.contextMenu(screen.getByText("Local PG"));
    await user.click(screen.getByText("Duplicate"));

    expect(duplicate).toHaveBeenCalledWith("1", expect.any(Object));
    expect(snackbar.info).toHaveBeenCalledWith("Credentials were not copied");
  });
});
