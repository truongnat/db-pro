import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/modules/connection/queries/connection.queries", () => ({
  useConnectionList: () => ({ data: [] }),
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
});
