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

  it("parses int64 values and calls onSave with number", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor
        value={{ type: "int64", value: 42 }}
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "99{Enter}");

    expect(onSave).toHaveBeenCalledWith({ type: "int64", value: 99 });
  });

  it("calls onCancel for invalid int64 input", async () => {
    const onCancel = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor
        value={{ type: "int64", value: 42 }}
        onSave={vi.fn()}
        onCancel={onCancel}
      />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "not-a-number{Enter}");

    expect(onCancel).toHaveBeenCalled();
  });

  it("parses float64 values", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor
        value={{ type: "float64", value: 3.14 }}
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "2.71{Enter}");

    expect(onSave).toHaveBeenCalledWith({ type: "float64", value: 2.71 });
  });

  it("parses bool values from text", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor
        value={{ type: "bool", value: false }}
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "true{Enter}");

    expect(onSave).toHaveBeenCalledWith({ type: "bool", value: true });
  });

  it("parses bool false from non-'true' text", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor
        value={{ type: "bool", value: true }}
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "no{Enter}");

    expect(onSave).toHaveBeenCalledWith({ type: "bool", value: false });
  });
});
