import { describe, expect, it } from "vitest";
import { render } from "@testing-library/react";

import { useScrollParent } from "../use-scroll-parent";

function Probe({
  nodeRef,
  onResolve,
}: {
  nodeRef: { readonly current: HTMLElement | null };
  onResolve: (value: { scrollElement: HTMLElement | null; scrollMargin: number }) => void;
}) {
  onResolve(useScrollParent(nodeRef));
  return <div />;
}

describe("useScrollParent", () => {
  it("walks past non-scrolling ancestors to the panel scroller", () => {
    const scroller = document.createElement("div");
    scroller.style.overflowY = "auto";
    scroller.setAttribute("data-testid", "panel-scroller");

    const middle = document.createElement("div");
    const leaf = document.createElement("div");
    scroller.append(middle);
    middle.append(leaf);
    document.body.append(scroller);

    const nodeRef: { current: HTMLElement | null } = { current: leaf };

    let resolved: { scrollElement: HTMLElement | null; scrollMargin: number } = {
      scrollElement: null,
      scrollMargin: 0,
    };

    render(<Probe nodeRef={nodeRef} onResolve={(v) => (resolved = v)} />);

    expect(resolved.scrollElement).toBe(scroller);
    expect(resolved.scrollMargin).toBeTypeOf("number");

    scroller.remove();
  });

  it("stops at the first scrollable ancestor, not the outermost one", () => {
    const outer = document.createElement("div");
    outer.style.overflowY = "auto";
    const inner = document.createElement("div");
    inner.style.overflowY = "scroll";
    const leaf = document.createElement("div");
    outer.append(inner);
    inner.append(leaf);
    document.body.append(outer);

    const nodeRef: { current: HTMLElement | null } = { current: leaf };

    let resolved: HTMLElement | null = null;
    render(<Probe nodeRef={nodeRef} onResolve={(v) => (resolved = v.scrollElement)} />);

    expect(resolved).toBe(inner);
    outer.remove();
  });

  it("leaves scrollElement null when nothing above scrolls", () => {
    const leaf = document.createElement("div");
    document.body.append(leaf);

    const nodeRef: { current: HTMLElement | null } = { current: leaf };

    let resolved: HTMLElement | null | undefined;
    render(<Probe nodeRef={nodeRef} onResolve={(v) => (resolved = v.scrollElement)} />);

    expect(resolved).toBeNull();
    leaf.remove();
  });
});

describe("sidebar search results do not create a nested scroll area", () => {
  it("SearchView renders no overflow-y-auto results box", async () => {
    // The sidebar is already the scroller; a second one inside it captures the
    // wheel and is the exact defect E1 in docs/ui-audit/runtime-ux-audit-v3.md.
    const source = await import(
      "@/commons/components/shell/sidebar-views/search-view.tsx?raw"
    ).catch(() => null);

    if (!source) {
      // Fall back to reading the file so the assertion is not silently skipped.
      const { readFileSync } = await import("node:fs");
      const text = readFileSync(
        "src/commons/components/shell/sidebar-views/search-view.tsx",
        "utf8",
      );
      expect(text).not.toContain("overflow-y-auto");
      expect(text).toContain("useScrollParent");
      return;
    }

    const text = (source as { default: string }).default;
    expect(text).not.toContain("overflow-y-auto");
    expect(text).toContain("useScrollParent");
  });
});
