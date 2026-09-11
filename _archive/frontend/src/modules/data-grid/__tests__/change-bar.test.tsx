import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, number>) => {
      if (key === "dataGrid.changes.pending") return `${params?.count} pending changes`;
      if (key === "dataGrid.changes.edits") return `${params?.count} edits`;
      if (key === "dataGrid.changes.deletes") return `${params?.count} deletes`;
      if (key === "dataGrid.changes.apply") return "Apply";
      if (key === "dataGrid.changes.applying") return "Applying…";
      if (key === "dataGrid.changes.revertAll") return "Revert all";
      if (key === "dataGrid.changes.retryFailed") return "Retry failed";
      if (key === "dataGrid.changes.applyPartial")
        return `Applied ${params?.applied} of ${params?.total}`;
      if (key === "dataGrid.changes.failedCount") return `${params?.count} failed`;
      if (key === "dataGrid.changes.deleteSelected") return `Delete ${params?.count} selected`;
      return key;
    },
  }),
}));

import { ChangeBar } from "../components/change-bar";
import type { StagedChange } from "../state/staged-changes.store";

function makeEdit(id: string): StagedChange {
  return {
    id,
    kind: "cell-edit",
    pkValues: [{ type: "int64", value: "1" }],
    changes: { name: { type: "text", value: "x" } },
    error: null,
  };
}

function makeDelete(id: string): StagedChange {
  return { id, kind: "row-delete", pkValues: [{ type: "int64", value: "1" }], error: null };
}

function makeFailedEdit(id: string): StagedChange {
  return {
    id,
    kind: "cell-edit",
    pkValues: [{ type: "int64", value: "1" }],
    changes: { name: { type: "text", value: "x" } },
    error: "unique violation",
  };
}

describe("ChangeBar", () => {
  const defaultProps = {
    changes: [],
    isApplying: false,
    onApply: vi.fn(),
    onRevertAll: vi.fn(),
  };

  it("renders nothing when no changes and no selection", () => {
    const { container } = render(<ChangeBar {...defaultProps} />);
    expect(container.innerHTML).toBe("");
  });

  it("shows pending count with edits and deletes", () => {
    render(
      <ChangeBar {...defaultProps} changes={[makeEdit("1"), makeEdit("2"), makeDelete("3")]} />,
    );
    expect(screen.getByText("3 pending changes")).toBeTruthy();
  });

  it("shows Apply button", () => {
    render(<ChangeBar {...defaultProps} changes={[makeEdit("1")]} />);
    expect(screen.getByText("Apply")).toBeTruthy();
  });

  it("disables Apply when isApplying", () => {
    render(<ChangeBar {...defaultProps} changes={[makeEdit("1")]} isApplying />);
    const applyBtn = screen.getByText("Applying…");
    expect(applyBtn).toBeTruthy();
    expect(applyBtn.closest("button")).toBeDisabled();
  });

  it("calls onRevertAll when Revert all is clicked", () => {
    const onRevertAll = vi.fn();
    render(<ChangeBar {...defaultProps} changes={[makeEdit("1")]} onRevertAll={onRevertAll} />);
    fireEvent.click(screen.getByText("Revert all"));
    expect(onRevertAll).toHaveBeenCalled();
  });

  it("shows Retry failed button when failures exist", () => {
    const onRetryFailed = vi.fn();
    render(
      <ChangeBar {...defaultProps} changes={[makeFailedEdit("1")]} onRetryFailed={onRetryFailed} />,
    );
    const retryBtn = screen.getByText("Retry failed");
    expect(retryBtn).toBeTruthy();
    fireEvent.click(retryBtn);
    expect(onRetryFailed).toHaveBeenCalled();
  });

  it("does not show Retry failed when no failures", () => {
    render(<ChangeBar {...defaultProps} changes={[makeEdit("1")]} onRetryFailed={vi.fn()} />);
    expect(screen.queryByText("Retry failed")).toBeNull();
  });

  it("shows partial failure message", () => {
    render(<ChangeBar {...defaultProps} changes={[makeEdit("1"), makeFailedEdit("2")]} />);
    expect(screen.getByText("Applied 1 of 2")).toBeTruthy();
    expect(screen.getByText("1 failed")).toBeTruthy();
  });

  it("shows Delete selected button when selection exists", () => {
    const onBatchDelete = vi.fn();
    render(
      <ChangeBar
        {...defaultProps}
        changes={[makeEdit("1")]}
        onBatchDelete={onBatchDelete}
        selectedRows={new Set([0, 1, 2])}
      />,
    );
    const deleteBtn = screen.getByText("Delete 3 selected");
    expect(deleteBtn).toBeTruthy();
    fireEvent.click(deleteBtn);
    expect(onBatchDelete).toHaveBeenCalledWith(new Set([0, 1, 2]));
  });

  it("shows ChangeBar when only selection exists (no changes)", () => {
    render(<ChangeBar {...defaultProps} onBatchDelete={vi.fn()} selectedRows={new Set([0])} />);
    expect(screen.getByText("Delete 1 selected")).toBeTruthy();
  });

  it("does not show Delete selected when selection is empty", () => {
    render(
      <ChangeBar
        {...defaultProps}
        changes={[makeEdit("1")]}
        onBatchDelete={vi.fn()}
        selectedRows={new Set()}
      />,
    );
    expect(screen.queryByText(/Delete.*selected/)).toBeNull();
  });
});
