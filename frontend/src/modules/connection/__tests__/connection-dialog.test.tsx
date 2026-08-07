import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { I18nextProvider, initReactI18next } from "react-i18next";
import i18n from "i18next";

import { ConnectionDialog } from "../components/connection-dialog";
import * as queries from "../queries/connection.queries";

vi.mock("../queries/connection.queries", () => ({
  useCreateConnection: vi.fn(),
  useUpdateConnection: vi.fn(),
  useTestConnection: vi.fn(),
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
  });

  it("does not render when open=false", () => {
    renderWithProviders(
      <ConnectionDialog open={false} onClose={vi.fn()} />,
    );
    expect(screen.queryByText("New Connection")).not.toBeInTheDocument();
  });

  it("renders dialog title when open=true", () => {
    renderWithProviders(
      <ConnectionDialog open={true} onClose={vi.fn()} />,
    );
    expect(screen.getByText("New Connection")).toBeInTheDocument();
  });

  it("renders edit title when editConnectionId is provided", () => {
    renderWithProviders(
      <ConnectionDialog open={true} onClose={vi.fn()} editConnectionId="conn-1" />,
    );
    expect(screen.getByText("Edit Connection")).toBeInTheDocument();
  });
});
