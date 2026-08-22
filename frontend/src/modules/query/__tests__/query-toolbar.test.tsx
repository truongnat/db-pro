import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

import { QueryToolbar } from "../components/query-toolbar";
import { QueryCommandBar } from "../components/query-command-bar";

vi.mock("@/modules/connection/queries/connection.queries", () => ({
  useConnectionList: () => ({
    data: [{ id: "conn-1", name: "Test DB", database: "testdb" }],
  }),
}));

vi.mock("../stores/schema-catalog.store", () => ({
  useSchemaCatalogStore: (selector: (state: unknown) => unknown) =>
    selector({
      catalogs: new Map(),
    }),
}));

const defaultProps = {
  onExecuteCurrent: vi.fn(),
  onExecuteAll: vi.fn(),
  onCancel: vi.fn(),
  onExplain: vi.fn(),
  onClear: vi.fn(),
  onExport: vi.fn(),
  onFormat: vi.fn(),
  onSaveQuery: vi.fn(),
  onExportSql: vi.fn(),
  onImportSql: vi.fn(),
  isExecuting: false,
  isExplaining: false,
  hasConnection: true,
  hasSql: true,
};

function renderToolbar(overrides: Partial<typeof defaultProps> = {}) {
  return render(<QueryToolbar {...defaultProps} {...overrides} />);
}

describe("QueryToolbar — Run split button", () => {
  it("clicking Run button calls onExecuteCurrent directly", async () => {
    const user = userEvent.setup();
    renderToolbar();
    const runButton = screen.getByRole("button", { name: /query\.run/i });
    await user.click(runButton);
    expect(defaultProps.onExecuteCurrent).toHaveBeenCalledTimes(1);
    expect(defaultProps.onExecuteAll).not.toHaveBeenCalled();
  });

  it("Run button does NOT open dropdown", async () => {
    const user = userEvent.setup();
    renderToolbar();
    const runButton = screen.getByRole("button", { name: /query\.run/i });
    await user.click(runButton);
    // Menu items should NOT be visible after clicking Run
    expect(screen.queryByText("query.runCurrent")).not.toBeInTheDocument();
  });

  it("clicking chevron opens dropdown with Run Current and Run All", async () => {
    const user = userEvent.setup();
    renderToolbar();
    const chevronButton = screen.getByRole("button", { name: /run options/i });
    await user.click(chevronButton);
    expect(await screen.findByText("query.runCurrent")).toBeInTheDocument();
    expect(await screen.findByText("query.runAll")).toBeInTheDocument();
  });

  it("dropdown does NOT contain Explain (it has its own toolbar button)", async () => {
    const user = userEvent.setup();
    renderToolbar();
    const chevronButton = screen.getByRole("button", { name: /run options/i });
    await user.click(chevronButton);
    // Wait for menu to appear
    await screen.findByText("query.runCurrent");
    // The dropdown menu should have only 2 items (Run Current, Run All)
    const dropdownItems = screen.getAllByRole("menuitem");
    const dropdownTexts = dropdownItems.map((el) => el.textContent);
    expect(dropdownTexts.some((t) => t?.includes("query.explain"))).toBe(false);
  });

  it("Run is disabled when no connection", () => {
    renderToolbar({ hasConnection: false });
    const runButton = screen.getByRole("button", { name: /query\.run/i });
    expect(runButton).toBeDisabled();
  });

  it("Run is disabled when no SQL", () => {
    renderToolbar({ hasSql: false });
    const runButton = screen.getByRole("button", { name: /query\.run/i });
    expect(runButton).toBeDisabled();
  });

  it("Run is disabled while executing", () => {
    renderToolbar({ isExecuting: true });
    const runButton = screen.getByRole("button", { name: /query\.run/i });
    expect(runButton).toBeDisabled();
  });

  it("chevron is disabled when no connection", () => {
    renderToolbar({ hasConnection: false });
    const chevronButton = screen.getByRole("button", { name: /run options/i });
    expect(chevronButton).toBeDisabled();
  });

  it("Stop button is visible while executing", () => {
    renderToolbar({ isExecuting: true });
    expect(screen.getByText("common.actions.cancel")).toBeInTheDocument();
  });

  it("Stop button is hidden when not executing", () => {
    renderToolbar({ isExecuting: false });
    expect(screen.queryByText("common.actions.cancel")).not.toBeInTheDocument();
  });

  it("Explain exists as a separate toolbar button", () => {
    renderToolbar();
    // There should be an Explain button in the toolbar (not in dropdown)
    const explainButtons = screen.getAllByRole("button");
    const explainButton = explainButtons.find((btn) => btn.textContent?.includes("query.explain"));
    expect(explainButton).toBeDefined();
  });
});

describe("QueryCommandBar — Export enablement (QA-P2-22)", () => {
  const commandBarProps = {
    tabId: "tab-1",
    connectionId: "conn-1",
    context: { database: "testdb", schema: null },
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
    hasConnection: true,
    hasSql: true,
  };

  it("disables Export Results when hasResults is false", async () => {
    const user = userEvent.setup();
    render(<QueryCommandBar {...commandBarProps} hasResults={false} />);

    const moreButton = screen.getAllByRole("button").pop()!;
    await user.click(moreButton);

    const exportItem = await screen.findByText("query.exportResults");
    expect(exportItem.closest("[data-disabled]")).not.toBeNull();
  });

  it("enables Export Results when hasResults is true, even if hasSql is false", async () => {
    const user = userEvent.setup();
    render(<QueryCommandBar {...commandBarProps} hasSql={false} hasResults={true} />);

    const moreButton = screen.getAllByRole("button").pop()!;
    await user.click(moreButton);

    const exportItem = await screen.findByText("query.exportResults");
    expect(exportItem.closest("[data-disabled]")).toBeNull();
  });
});
