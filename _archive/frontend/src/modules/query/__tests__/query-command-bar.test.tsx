import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

const connectionHarness = vi.hoisted(() => ({
  connections: [] as Array<{ id: string; name: string; database: string }>,
  statuses: {} as Record<string, string>,
}));

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/modules/connection/queries/connection.queries", () => ({
  useConnectionList: () => ({ data: connectionHarness.connections }),
  useConnect: () => ({ mutate: vi.fn(), isPending: false }),
}));

vi.mock("@/modules/connection/state/connection.store", () => ({
  useConnectionModuleStore: (selector: (state: { statuses: Record<string, string> }) => unknown) =>
    selector({ statuses: connectionHarness.statuses }),
}));

import { QueryCommandBar } from "../components/query-command-bar";

const defaultProps = {
  tabId: "tab-1",
  connectionId: null,
  context: { database: null, schema: null },
  onExecuteCurrent: vi.fn(),
  onExecuteAll: vi.fn(),
  onCancel: vi.fn(),
  onExplain: vi.fn(),
  onClear: vi.fn(),
  onExport: vi.fn(),
  onFormat: vi.fn(),
  onExportSql: vi.fn(),
  onImportSql: vi.fn(),
  isExecuting: false,
  isExplaining: false,
  hasConnection: false,
  hasSql: true,
  hasResults: false,
};

async function openMoreMenu() {
  const user = userEvent.setup();
  const buttons = screen.getAllByRole("button");
  await user.click(buttons[buttons.length - 1]);
  return user;
}

describe("QueryCommandBar — Export Results availability", () => {
  beforeEach(() => {
    connectionHarness.connections = [];
    connectionHarness.statuses = {};
  });

  it("disables Export Results when SQL exists but there are no results", async () => {
    render(<QueryCommandBar {...defaultProps} hasSql hasResults={false} />);
    await openMoreMenu();
    const item = await screen.findByText("query.exportResults");
    expect(item).toHaveAttribute("data-disabled");
  });

  it("enables Export Results when a result set exists", async () => {
    render(<QueryCommandBar {...defaultProps} hasSql hasResults />);
    await openMoreMenu();
    const item = await screen.findByText("query.exportResults");
    expect(item).not.toHaveAttribute("data-disabled");
  });

  it("keeps Export SQL driven by SQL text, not result state", async () => {
    render(<QueryCommandBar {...defaultProps} hasSql hasResults={false} />);
    await openMoreMenu();
    const item = await screen.findByText("query.exportSql");
    expect(item).not.toHaveAttribute("data-disabled");
  });

  it("shows reconnect for a known disconnected connection", () => {
    connectionHarness.connections = [{ id: "conn-1", name: "Local", database: "app" }];
    connectionHarness.statuses = { "conn-1": "disconnected" };
    render(<QueryCommandBar {...defaultProps} connectionId="conn-1" hasConnection hasResults />);

    expect(screen.getByRole("button", { name: "workspace.reconnect" })).toBeInTheDocument();
  });
});
