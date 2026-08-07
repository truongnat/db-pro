import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { I18nextProvider, initReactI18next } from "react-i18next";
import i18n from "i18next";

import { QuickOpen } from "@/commons/components/quick-open";
import { useQuickOpenStore } from "@/commons/stores/quick-open.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useSchemaCatalogStore } from "@/modules/query/stores/schema-catalog.store";

vi.mock("@/commons/stores/shell.store", () => ({
  useShellStore: vi.fn(() => ({
    setSidebarView: vi.fn(),
  })),
}));

vi.mock("@/commons/hooks/use-sidebar-tab-ops", () => ({
  useSidebarTabOps: vi.fn(() => ({
    openSchemaPreview: vi.fn((connId, schema, name) => {
      useWorkspaceStore.getState().openDbObject({
        id: `tab-${name}`,
        kind: "db-object",
        title: name,
        connectionId: connId,
        resourceKey: `dbobj:${schema}.${name}:${connId}`,
        dirty: false,
        pinned: false,
        preview: true,
        order: 1,
        data: { schema, objectName: name, objectType: "table", activeSection: "columns" },
      } as never);
    }),
  })),
}));

vi.mock("@/modules/connection/queries/connection.queries", () => ({
  useConnectionList: vi.fn(() => ({
    data: [
      {
        id: "conn-1",
        name: "Local",
        host: "localhost",
        port: 5432,
        database: "app",
        username: "dev",
        driver: "postgres",
        sslMode: "disable",
        createdAt: "",
        updatedAt: "",
      },
    ],
  })),
  useConnect: vi.fn(() => ({ mutate: vi.fn() })),
}));

vi.mock("@/commons/stores/explorer.store", () => ({
  useExplorerStore: vi.fn(() => ({
    toggleNode: vi.fn(),
  })),
}));

vi.mock("@/commons/stores/command.store", () => {
  const mockFn = vi.fn(() => ({ close: vi.fn() }));
  mockFn.getState = vi.fn(() => ({ close: vi.fn() }));
  return { useCommandStore: mockFn };
});

vi.mock("@/lib/utils", () => ({
  cn: (...args: unknown[]) => args.filter(Boolean).join(" "),
}));

i18n.use(initReactI18next).init({
  resources: {
    en: {
      translation: {
        quickOpen: {
          placeholder: "Search tables, views, connections...",
          noResults: "No matching resources.",
          groups: { open: "Open", tables: "Tables", views: "Views", schemas: "Schemas", connections: "Connections" },
        },
      },
    },
  },
  lng: "en",
  fallbackLng: "en",
});

function createQueryClient() {
  return new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
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

function resetStores() {
  useQuickOpenStore.setState({ isOpen: false, query: "", selectedIndex: 0 });
  useWorkspaceStore.setState({ tabs: [], activeTabId: null, recentlyClosed: [] });
  useConnectionStore.setState({ connections: [], explorerConnectionId: null, isLoading: false, error: null });
  useSchemaCatalogStore.setState({ catalogs: new Map() });
}

function seedCatalog() {
  useSchemaCatalogStore.setState({
    catalogs: new Map([
      [
        "conn-1",
        {
          schemas: [{ name: "public" }],
          objects: [{ name: "client", schema: "public", rowCount: 1, kind: "table" as const }],
          columnsByTable: new Map(),
          columnsLoaded: new Set(),
          columnsLoading: new Map(),
        },
      ],
    ]),
  });
}

describe("QuickOpen", () => {
  beforeEach(() => {
    resetStores();
  });

  afterEach(() => {
    resetStores();
  });

  it("renders nothing when isOpen is false", () => {
    renderWithProviders(<QuickOpen />);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("renders catalog tables with context when open", () => {
    seedCatalog();
    useQuickOpenStore.getState().open();
    renderWithProviders(<QuickOpen />);
    expect(screen.getByPlaceholderText("Search tables, views, connections...")).toBeTruthy();
    expect(screen.getByText("client")).toBeTruthy();
    expect(screen.getByText("public · Local")).toBeTruthy();
  });

  it("filtering removes non-matching items", async () => {
    seedCatalog();
    useQuickOpenStore.getState().open();
    renderWithProviders(<QuickOpen />);
    const input = screen.getByPlaceholderText("Search tables, views, connections...");
    await userEvent.type(input, "zzz");
    expect(screen.getByText("No matching resources.")).toBeTruthy();
  });

  it("opening a table creates a preview tab without duplicate", async () => {
    seedCatalog();
    useQuickOpenStore.getState().open();
    renderWithProviders(<QuickOpen />);

    const item = screen.getByText("client");
    await userEvent.click(item);

    const tabs = useWorkspaceStore.getState().tabs;
    expect(tabs).toHaveLength(1);
    expect(tabs[0]).toMatchObject({ resourceKey: "dbobj:public.client:conn-1", preview: true });

    expect(useQuickOpenStore.getState().isOpen).toBe(false);
  });

  it("opening an existing resource activates the existing tab (no duplicate)", async () => {
    seedCatalog();
    useWorkspaceStore.setState({
      tabs: [
        {
          id: "existing-tab",
          kind: "db-object",
          title: "client",
          connectionId: "conn-1",
          resourceKey: "dbobj:public.client:conn-1",
          dirty: false,
          pinned: false,
          preview: false,
          order: 1,
          data: { schema: "public", objectName: "client", objectType: "table", activeSection: "columns" },
        },
      ],
      activeTabId: "existing-tab",
    });
    useQuickOpenStore.getState().open();
    renderWithProviders(<QuickOpen />);

    const item = screen.getByRole("option", { name: /client.*public · Local/ });
    await userEvent.click(item);

    const tabs = useWorkspaceStore.getState().tabs;
    expect(tabs).toHaveLength(1);
    expect(useWorkspaceStore.getState().activeTabId).toBe("existing-tab");
  });
});
