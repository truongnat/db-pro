import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { I18nextProvider, initReactI18next } from "react-i18next";
import i18n from "i18next";

import { CommandPalette } from "@/commons/components/command-palette";
import { useCommandStore } from "@/commons/stores/command.store";

vi.mock("@/commons/stores/recent.store", () => ({
  useRecentStore: vi.fn((selector) =>
    selector({ recentConnections: [], addRecentConnection: vi.fn() }),
  ),
}));

vi.mock("@/modules/connection/queries/connection.queries", () => ({
  useConnectionList: vi.fn(() => ({ data: [] })),
  useConnect: vi.fn(() => ({ mutate: vi.fn() })),
}));

vi.mock("@/lib/utils", () => ({
  cn: (...args: unknown[]) => args.filter(Boolean).join(" "),
}));

i18n.use(initReactI18next).init({
  resources: {
    en: {
      translation: {
        commandPalette: { placeholder: "Search...", noResults: "No results" },
        "commands.groups.recent": "Recent",
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

function resetStore() {
  useCommandStore.setState({
    commands: [],
    isOpen: false,
  });
}

describe("CommandPalette", () => {
  beforeEach(() => {
    resetStore();
  });

  afterEach(() => {
    resetStore();
  });

  it("renders nothing when isOpen is false", () => {
    renderWithProviders(<CommandPalette />);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("renders dialog when isOpen is true", () => {
    useCommandStore.getState().open();
    renderWithProviders(<CommandPalette />);
    expect(screen.getByRole("dialog")).toBeTruthy();
  });

  it("closes dialog when close() is called", () => {
    useCommandStore.getState().open();
    const { rerender } = renderWithProviders(<CommandPalette />);
    expect(screen.getByRole("dialog")).toBeTruthy();

    useCommandStore.getState().close();
    rerender(
      <I18nextProvider i18n={i18n}>
        <QueryClientProvider client={createQueryClient()}>
          <CommandPalette />
        </QueryClientProvider>
      </I18nextProvider>,
    );
    expect(screen.queryByRole("dialog")).toBeNull();
  });
});
