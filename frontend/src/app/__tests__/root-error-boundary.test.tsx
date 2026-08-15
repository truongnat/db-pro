import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { RootErrorBoundary } from "../root-error-boundary";

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
