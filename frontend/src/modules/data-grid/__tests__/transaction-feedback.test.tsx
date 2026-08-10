import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

import { TransactionFeedback } from "../components/transaction-feedback";

describe("TransactionFeedback", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders nothing when result is null", () => {
    const { container } = render(
      <TransactionFeedback result={null} onDismiss={vi.fn()} />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("shows success state with count and duration", () => {
    render(
      <TransactionFeedback
        result={{ kind: "success", succeeded: 3, failed: 0, durationMs: 250 }}
        onDismiss={vi.fn()}
      />,
    );
    expect(screen.getByText("3 changes applied")).toBeTruthy();
    expect(screen.getByText("250 ms")).toBeTruthy();
  });

  it("shows partial state with succeeded and failed counts", () => {
    render(
      <TransactionFeedback
        result={{ kind: "partial", succeeded: 2, failed: 1, durationMs: 1500 }}
        onDismiss={vi.fn()}
      />,
    );
    expect(screen.getByText("2 succeeded, 1 failed")).toBeTruthy();
    expect(screen.getByText("1.5 s")).toBeTruthy();
  });

  it("shows failure state", () => {
    render(
      <TransactionFeedback
        result={{ kind: "failure", succeeded: 0, failed: 3, durationMs: 100 }}
        onDismiss={vi.fn()}
      />,
    );
    expect(screen.getByText("Apply failed")).toBeTruthy();
  });

  it("formats duration under 1000ms as ms", () => {
    render(
      <TransactionFeedback
        result={{ kind: "success", succeeded: 1, failed: 0, durationMs: 999 }}
        onDismiss={vi.fn()}
      />,
    );
    expect(screen.getByText("999 ms")).toBeTruthy();
  });

  it("formats duration 1000ms+ as seconds", () => {
    render(
      <TransactionFeedback
        result={{ kind: "success", succeeded: 1, failed: 0, durationMs: 1000 }}
        onDismiss={vi.fn()}
      />,
    );
    expect(screen.getByText("1.0 s")).toBeTruthy();
  });

  it("auto-dismisses after autoDismissMs", () => {
    const onDismiss = vi.fn();
    render(
      <TransactionFeedback
        result={{ kind: "success", succeeded: 1, failed: 0, durationMs: 100 }}
        onDismiss={onDismiss}
        autoDismissMs={3000}
      />,
    );

    expect(screen.getByText("1 change applied")).toBeTruthy();

    vi.advanceTimersByTime(3000);
    expect(onDismiss).toHaveBeenCalled();
  });

  it("dismisses manually via X button", () => {
    const onDismiss = vi.fn();
    render(
      <TransactionFeedback
        result={{ kind: "success", succeeded: 1, failed: 0, durationMs: 100 }}
        onDismiss={onDismiss}
      />,
    );

    const buttons = screen.getAllByRole("button");
    const xButton = buttons[buttons.length - 1];
    fireEvent.click(xButton);

    expect(onDismiss).toHaveBeenCalled();
  });

  it("hides after result becomes null", () => {
    const { rerender } = render(
      <TransactionFeedback
        result={{ kind: "success", succeeded: 1, failed: 0, durationMs: 100 }}
        onDismiss={vi.fn()}
      />,
    );

    expect(screen.getByText("1 change applied")).toBeTruthy();

    rerender(
      <TransactionFeedback result={null} onDismiss={vi.fn()} />,
    );

    expect(screen.queryByText("1 change applied")).toBeNull();
  });
});
