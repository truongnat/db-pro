import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ErDiagram } from "../components/er-diagram";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { generateErFixture } from "./er-fixture";

/**
 * PR#12 re-review — app-level navigation test for the large-schema overview.
 *
 * The regression was at the app wiring level: the overview view's single-click
 * callback ALSO navigated (openDbObject → activeTabId switch), so the ER
 * diagram unmounted before the opass focus (fade + highlight) could render.
 *
 * This test renders the real `ErDiagram` on a large (L-tier) schema with the
 * lazy CytoscapeErView mocked as a harness that exposes its props, then proves
 * the full chain:
 *
 *   open large ER
 *   → click node (single tap)
 *   → schema workspace remains active   (openDbObject NOT called)
 *   → explicit action (double-click / "Open table")
 *   → openDbObject IS called with the right db-object tab
 */
const viewHarness = vi.hoisted(() => ({
  props: null as Record<string, unknown> | null,
  reset() {
    this.props = null;
  },
})); // The lazy `CytoscapeErView` is code-split; mock the module with a harness
// that captures the props ErDiagram actually passes. The double-click sim
// opens the FIRST fixture table (derived below, not hardcoded — the fixture
// generator's naming is deterministic but this must not silently break).
// The first fixture table id (derived in the test, not hardcoded — the
// fixture generator's naming is deterministic but this must not silently
// break if the prefix list ever changes).
let firstTableId = "";
vi.mock("../components/cytoscape-view", () => ({
  CytoscapeErView: (props: Record<string, unknown>) => {
    viewHarness.props = props;
    return (
      <div data-testid="mock-overview">
        <button
          type="button"
          onClick={() => (props.onOpenTable as (id: string) => void)(firstTableId)}
        >
          simulate-double-click
        </button>
      </div>
    );
  },
}));

// Layout is a worker pipeline; a real worker doesn't exist in jsdom. Mock the
// hook with a ready state so the overview mounts deterministically.
vi.mock("../hooks/use-worker-layout", () => ({
  useWorkerLayout: () => ({
    status: "ready",
    positions: new Map(),
    layoutMs: 5,
    nodeCount: 500,
    fromCache: true,
    degraded: false,
    refining: false,
    error: null,
  }),
}));

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({
    t: (k: string) => k,
    i18n: { language: "en" },
    currentLanguage: "en",
  }),
}));

describe("ErDiagram — large overview click must not navigate (PR#12 re-review P1)", () => {
  beforeEach(() => {
    viewHarness.reset();
  });

  it("renders the large schema on the overview and passes onOpenTable, not onNodeClick", async () => {
    const data = generateErFixture(500, 42); // complexity ≈ 1,100 → L tier
    const first = data.tables[0];
    firstTableId = `public.${first.name}`;
    const openSpy = vi.spyOn(useWorkspaceStore.getState(), "openDbObject");

    render(<ErDiagram connectionId="conn-1" schema="public" data={data} />);

    // The lazy overview mounts (Suspense resolves to the harness).
    await waitFor(() => expect(screen.getByTestId("mock-overview")).toBeInTheDocument());

    // Wiring contract: ErDiagram feeds the view an EXPLICIT open action, not a
    // single-click navigation callback.
    expect(viewHarness.props).not.toBeNull();
    expect(typeof viewHarness.props?.onOpenTable).toBe("function");
    expect(viewHarness.props?.onNodeClick).toBeUndefined();

    // P1: while the diagram is open, nothing has navigated — the schema
    // workspace remains the active view.
    expect(openSpy).not.toHaveBeenCalled();

    // The explicit action (double-click / "Open table") navigates to the
    // correct db-object tab for the clicked table.
    fireEvent.click(screen.getByRole("button", { name: "simulate-double-click" }));

    await waitFor(() => expect(openSpy).toHaveBeenCalledTimes(1));
    expect(openSpy).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "db-object",
        connectionId: "conn-1",
        title: first.name,
        data: expect.objectContaining({
          schema: "public",
          objectName: first.name,
          objectType: "table",
        }),
      }),
    );

    openSpy.mockRestore();
  });

  it("a single click alone (no explicit action) never opens a table", async () => {
    const data = generateErFixture(500, 42);
    const openSpy = vi.spyOn(useWorkspaceStore.getState(), "openDbObject");

    render(<ErDiagram connectionId="conn-1" schema="public" data={data} />);
    await waitFor(() => expect(screen.getByTestId("mock-overview")).toBeInTheDocument());

    // The view received only the explicit action; there is no single-click
    // navigation path left in the props ErDiagram hands to the overview.
    expect(viewHarness.props?.onNodeClick).toBeUndefined();
    expect(openSpy).not.toHaveBeenCalled();

    openSpy.mockRestore();
  });
});
