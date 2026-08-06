import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { CellEditor } from "../components/cell-editor";

describe("CellEditor", () => {
  it("renders input with current value", () => {
    render(
      <CellEditor
        value={{ type: "text", value: "hello" }}
        onSave={vi.fn()}
        onCancel={vi.fn()}
      />,
    );
    const input = screen.getByRole("textbox");
    expect(input).toHaveValue("hello");
  });

  it("calls onSave with new text value on Enter", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor
        value={{ type: "text", value: "hello" }}
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "world{Enter}");

    expect(onSave).toHaveBeenCalledWith({ type: "text", value: "world" });
  });

  it("calls onCancel on Escape", async () => {
    const onCancel = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor
        value={{ type: "text", value: "hello" }}
        onSave={vi.fn()}
        onCancel={onCancel}
      />,
    );

    const input = screen.getByRole("textbox");
    await user.type(input, "{Escape}");

    expect(onCancel).toHaveBeenCalled();
  });

  it("calls onCancel when value is unchanged", async () => {
    const onCancel = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor
        value={{ type: "text", value: "hello" }}
        onSave={vi.fn()}
        onCancel={onCancel}
      />,
    );

    const input = screen.getByRole("textbox");
    await user.type(input, "{Enter}");

    expect(onCancel).toHaveBeenCalled();
  });

  it("converts NULL text to null cell", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor
        value={{ type: "text", value: "hello" }}
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "NULL{Enter}");

    expect(onSave).toHaveBeenCalledWith({ type: "null" });
  });
});
