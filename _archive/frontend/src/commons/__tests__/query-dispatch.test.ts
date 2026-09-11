import { afterEach, describe, expect, it, vi } from "vitest";
import { onQueryAction, dispatchQueryAction } from "../commands/query-dispatch";

describe("query-dispatch", () => {
  // Clean up listeners after each test by calling unsubscribers
  const unsubscribers: (() => void)[] = [];

  afterEach(() => {
    unsubscribers.forEach((unsub) => unsub());
    unsubscribers.length = 0;
  });

  it("dispatches action to registered listener", () => {
    const handler = vi.fn();
    unsubscribers.push(onQueryAction("execute", handler));

    dispatchQueryAction("execute");
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it("dispatches to multiple listeners for same action", () => {
    const h1 = vi.fn();
    const h2 = vi.fn();
    unsubscribers.push(onQueryAction("format", h1));
    unsubscribers.push(onQueryAction("format", h2));

    dispatchQueryAction("format");
    expect(h1).toHaveBeenCalledTimes(1);
    expect(h2).toHaveBeenCalledTimes(1);
  });

  it("does not dispatch to listeners of different actions", () => {
    const handler = vi.fn();
    unsubscribers.push(onQueryAction("execute", handler));

    dispatchQueryAction("cancel");
    expect(handler).not.toHaveBeenCalled();
  });

  it("unsubscribes correctly", () => {
    const handler = vi.fn();
    const unsub = onQueryAction("clear", handler);
    unsubscribers.push(onQueryAction("clear", vi.fn())); // another listener to keep the Set alive

    unsub(); // remove our handler
    dispatchQueryAction("clear");
    expect(handler).not.toHaveBeenCalled();
  });

  it("handles dispatching with no listeners gracefully", () => {
    // Should not throw
    dispatchQueryAction("saveQuery");
  });

  it("supports all action types", () => {
    const actions = [
      "execute",
      "executeCurrent",
      "explain",
      "format",
      "clear",
      "cancel",
      "export",
      "importSql",
      "exportSql",
      "saveQuery",
    ] as const;

    for (const action of actions) {
      const handler = vi.fn();
      unsubscribers.push(onQueryAction(action, handler));
      dispatchQueryAction(action);
      expect(handler).toHaveBeenCalledTimes(1);
    }
  });
});
