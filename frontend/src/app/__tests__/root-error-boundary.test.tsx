import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { RootErrorBoundary } from "../root-error-boundary";
import {
  reportSanitizedReactError,
  SANITIZED_REACT_ERROR_MESSAGE,
  SANITIZED_REACT_ROOT_OPTIONS,
} from "../root-error-reporting";

const SENSITIVE_ERROR =
  "postgres://dbpro:super-secret@prod.example.com/database SELECT * FROM private_table";

function CrashingChild(): never {
  throw new Error(SENSITIVE_ERROR);
}

afterEach(() => {
  vi.restoreAllMocks();
  localStorage.clear();
});

describe("RootErrorBoundary", () => {
  it("renders a deterministic fallback without exposing the raw error payload", () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);

    render(
      <RootErrorBoundary>
        <CrashingChild />
      </RootErrorBoundary>,
    );

    expect(screen.getByRole("alert")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Reload application" })).toBeInTheDocument();
    expect(screen.queryByText(SENSITIVE_ERROR)).not.toBeInTheDocument();
  });

  it("reports only a fixed sanitized message instead of the raw React error", () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);

    reportSanitizedReactError(new Error(SENSITIVE_ERROR));

    expect(consoleError).toHaveBeenCalledTimes(1);
    expect(consoleError).toHaveBeenCalledWith(SANITIZED_REACT_ERROR_MESSAGE);
    expect(consoleError.mock.calls.flat().join(" ")).not.toContain(SENSITIVE_ERROR);
  });

  it("overrides React root logging so raw sensitive errors never reach the console", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container, SANITIZED_REACT_ROOT_OPTIONS);

    await act(async () => {
      root.render(
        <RootErrorBoundary>
          <CrashingChild />
        </RootErrorBoundary>,
      );
    });

    const consoleOutput = consoleError.mock.calls
      .flatMap((args) => args.map((value) => String(value)))
      .join(" ");
    expect(container.querySelector('[role="alert"]')).not.toBeNull();
    expect(consoleOutput).toContain(SANITIZED_REACT_ERROR_MESSAGE);
    expect(consoleOutput).not.toContain(SENSITIVE_ERROR);

    await act(async () => {
      root.unmount();
    });
    container.remove();
  });

  it("keeps crash-recovery SQL intact when the recovery action is used", async () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const user = userEvent.setup();
    const onReload = vi.fn();
    const recoveryValue = JSON.stringify({
      state: {
        snapshots: [
          {
            tabId: "tab-dirty",
            connectionId: "connection-1",
            title: "Unsaved query",
            sql: "UPDATE users SET display_name = 'draft';",
            timestamp: 1,
          },
        ],
      },
      version: 0,
    });
    localStorage.setItem("db-pro-crash-recovery", recoveryValue);

    render(
      <RootErrorBoundary onReload={onReload}>
        <CrashingChild />
      </RootErrorBoundary>,
    );

    await user.click(screen.getByRole("button", { name: "Reload application" }));

    expect(onReload).toHaveBeenCalledTimes(1);
    expect(localStorage.getItem("db-pro-crash-recovery")).toBe(recoveryValue);
  });
});
