import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { CommandPalette } from "@/commons/components/command-palette";
import { useCommandStore } from "@/commons/stores/command.store";

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
    render(<CommandPalette />);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("renders dialog when isOpen is true", () => {
    useCommandStore.getState().open();
    render(<CommandPalette />);
    expect(screen.getByRole("dialog")).toBeTruthy();
  });

  it("closes dialog when close() is called", () => {
    useCommandStore.getState().open();
    const { rerender } = render(<CommandPalette />);
    expect(screen.getByRole("dialog")).toBeTruthy();

    useCommandStore.getState().close();
    rerender(<CommandPalette />);
    expect(screen.queryByRole("dialog")).toBeNull();
  });
});
