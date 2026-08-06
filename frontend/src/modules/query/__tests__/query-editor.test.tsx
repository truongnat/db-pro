import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

vi.mock("@monaco-editor/react", () => ({
  default: ({
    value,
    onChange,
  }: {
    value: string;
    onChange: (v: string) => void;
    onMount?: unknown;
    options?: unknown;
  }) => (
    <textarea
      data-testid="monaco-editor"
      value={value}
      onChange={(e) => onChange(e.target.value)}
    />
  ),
}));

import { QueryEditor } from "../components/query-editor";

describe("QueryEditor", () => {
  it("renders with initial value", () => {
    render(<QueryEditor value="SELECT 1" onChange={vi.fn()} onExecute={vi.fn()} />);
    const editor = screen.getByTestId("monaco-editor");
    expect(editor).toHaveValue("SELECT 1");
  });

  it("calls onChange when text changes", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();

    render(<QueryEditor value="" onChange={onChange} onExecute={vi.fn()} />);
    const editor = screen.getByTestId("monaco-editor");
    await user.type(editor, "SELECT");

    expect(onChange).toHaveBeenCalled();
  });

  it("renders editor in the document", () => {
    render(<QueryEditor value="" onChange={vi.fn()} onExecute={vi.fn()} />);
    expect(screen.getByTestId("monaco-editor")).toBeInTheDocument();
  });
});
