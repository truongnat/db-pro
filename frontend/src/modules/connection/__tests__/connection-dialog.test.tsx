import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { I18nextProvider, initReactI18next } from "react-i18next";
import i18n from "i18next";

import { ConnectionDialog } from "../components/connection-dialog";
import { useRecentStore } from "@/commons/stores/recent.store";
import * as queries from "../queries/connection.queries";

vi.mock("../queries/connection.queries", () => ({
  useCreateConnection: vi.fn(),
  useUpdateConnection: vi.fn(),
  useTestConnection: vi.fn(),
  useConnect: vi.fn(),
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
        },
        connection: {
          new: "New Connection",
          edit: "Edit Connection",
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

describe("ConnectionDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useRecentStore.setState({
      recentConnections: [],
      connectionDialogOpen: false,
      connectionDialogEditId: null,
    });
    vi.mocked(queries.useCreateConnection).mockReturnValue({
      mutate: vi.fn(),
      isPending: false,
    } as unknown as ReturnType<typeof queries.useCreateConnection>);
    vi.mocked(queries.useUpdateConnection).mockReturnValue({
      mutate: vi.fn(),
      isPending: false,
    } as unknown as ReturnType<typeof queries.useUpdateConnection>);
    vi.mocked(queries.useTestConnection).mockReturnValue({
      mutate: vi.fn(),
      isPending: false,
      isSuccess: false,
      isError: false,
    } as unknown as ReturnType<typeof queries.useTestConnection>);
    vi.mocked(queries.useConnect).mockReturnValue({
      mutate: vi.fn(),
      isPending: false,
    } as unknown as ReturnType<typeof queries.useConnect>);
  });

  it("does not render when closed", () => {
    renderWithProviders(<ConnectionDialog />);
    expect(screen.queryByText("New Connection")).not.toBeInTheDocument();
  });

  it("renders new-connection title when opened without edit id", () => {
    useRecentStore.getState().openConnectionDialog();
    renderWithProviders(<ConnectionDialog />);
    expect(screen.getByText("New Connection")).toBeInTheDocument();
  });

  it("renders edit title when opened with an edit id", () => {
    useRecentStore.getState().openConnectionDialog("conn-1");
    renderWithProviders(<ConnectionDialog />);
    expect(screen.getByText("Edit Connection")).toBeInTheDocument();
  });
});
