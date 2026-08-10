import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { CellEditor } from "../components/cell-editor";

describe("CellEditor", () => {
  it("renders input with current value", () => {
    render(
      <CellEditor value={{ type: "text", value: "hello" }} onSave={vi.fn()} onCancel={vi.fn()} />,
    );
    const input = screen.getByRole("textbox");
    expect(input).toHaveValue("hello");
  });

  it("calls onSave with new text value on Enter", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor value={{ type: "text", value: "hello" }} onSave={onSave} onCancel={vi.fn()} />,
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
      <CellEditor value={{ type: "text", value: "hello" }} onSave={vi.fn()} onCancel={onCancel} />,
    );

    const input = screen.getByRole("textbox");
    await user.type(input, "{Escape}");

    expect(onCancel).toHaveBeenCalled();
  });

  it("calls onCancel when value is unchanged", async () => {
    const onCancel = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor value={{ type: "text", value: "hello" }} onSave={vi.fn()} onCancel={onCancel} />,
    );

    const input = screen.getByRole("textbox");
    await user.type(input, "{Enter}");

    expect(onCancel).toHaveBeenCalled();
  });

  it("converts NULL text to null cell", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor value={{ type: "text", value: "hello" }} onSave={onSave} onCancel={vi.fn()} />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "NULL{Enter}");

    expect(onSave).toHaveBeenCalledWith({ type: "null" });
  });

  it("parses int64 values and calls onSave with number", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(<CellEditor value={{ type: "int64", value: 42 }} onSave={onSave} onCancel={vi.fn()} />);

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "99{Enter}");

    expect(onSave).toHaveBeenCalledWith({ type: "int64", value: 99 });
  });

  it("shows error for invalid int64 input", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor value={{ type: "int64", value: 42 }} onSave={onSave} onCancel={vi.fn()} />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "not-a-number{Enter}");

    expect(onSave).not.toHaveBeenCalled();
    expect(screen.getByText("Enter a valid integer")).toBeTruthy();
  });

  it("parses float64 values", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor value={{ type: "float64", value: 3.14 }} onSave={onSave} onCancel={vi.fn()} />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "2.71{Enter}");

    expect(onSave).toHaveBeenCalledWith({ type: "float64", value: 2.71 });
  });

  it("renders checkbox for bool values", () => {
    const onSave = vi.fn();

    render(
      <CellEditor value={{ type: "bool", value: true }} onSave={onSave} onCancel={vi.fn()} />,
    );

    const checkbox = screen.getByRole("checkbox") as HTMLInputElement;
    expect(checkbox.checked).toBe(true);
  });

  it("toggles bool value via checkbox", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor value={{ type: "bool", value: false }} onSave={onSave} onCancel={vi.fn()} />,
    );

    const checkbox = screen.getByRole("checkbox");
    await user.click(checkbox);

    expect(onSave).toHaveBeenCalledWith({ type: "bool", value: true });
  });

  it("emits uuid type for uuid values", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor
        value={{ type: "uuid", value: "00000000-0000-0000-0000-000000000000" }}
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "550e8400-e29b-41d4-a716-446655440000{Enter}");

    expect(onSave).toHaveBeenCalledWith({
      type: "uuid",
      value: "550e8400-e29b-41d4-a716-446655440000",
    });
  });

  it("emits datetime type for datetime values", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor
        value={{ type: "datetime", value: "2024-01-01T00:00:00Z" }}
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "2025-06-15T12:00:00Z{Enter}");

    expect(onSave).toHaveBeenCalledWith({
      type: "datetime",
      value: "2025-06-15T12:00:00Z",
    });
  });

  it("validates and emits json type", async () => {
    const onSave = vi.fn();

    const { container } = render(
      <CellEditor
        value={{ type: "json", value: { key: "val" } }}
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    const input = container.querySelector("input")!;
    input.focus();
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
      window.HTMLInputElement.prototype,
      "value",
    )!.set!;
    nativeInputValueSetter.call(input, '{"a":1}');
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

    expect(onSave).toHaveBeenCalledWith({ type: "json", value: { a: 1 } });
  });

  it("shows error for invalid json", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor
        value={{ type: "json", value: { key: "val" } }}
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "not-json{Enter}");

    expect(onSave).not.toHaveBeenCalled();
    expect(screen.getByText("Invalid JSON")).toBeTruthy();
  });

  it("uses columnType prop to determine type", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();

    render(
      <CellEditor
        value={{ type: "text", value: "hello" }}
        columnType="integer"
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "42{Enter}");

    expect(onSave).toHaveBeenCalledWith({ type: "int64", value: 42 });
  });

  it("renders readonly for bytea column type", () => {
    render(
      <CellEditor
        value={{ type: "bytes", value: [1, 2, 3] }}
        columnType="bytea"
        onSave={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    expect(screen.queryByRole("textbox")).toBeNull();
    expect(screen.getByText(/Binary editing is not supported/)).toBeTruthy();
  });

  it("renders readonly for numeric column type", () => {
    render(
      <CellEditor
        value={{ type: "float64", value: 3.14 }}
        columnType="numeric(30,10)"
        onSave={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    expect(screen.queryByRole("textbox")).toBeNull();
    expect(screen.getByText(/decimal editing is not supported/)).toBeTruthy();
  });

  it("renders readonly for decimal column type", () => {
    render(
      <CellEditor
        value={{ type: "float64", value: 99.99 }}
        columnType="decimal(38,18)"
        onSave={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    expect(screen.queryByRole("textbox")).toBeNull();
    expect(screen.getByText(/decimal editing is not supported/)).toBeTruthy();
  });

  it("renders readonly for bigint column type", () => {
    render(
      <CellEditor
        value={{ type: "int64", value: 42 }}
        columnType="bigint"
        onSave={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    expect(screen.queryByRole("textbox")).toBeNull();
    expect(screen.getByText(/integer editing is not supported/)).toBeTruthy();
  });
});
