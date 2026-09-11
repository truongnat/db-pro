import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

import { ColorPicker } from "../components/color-picker";

describe("ColorPicker", () => {
  it("renders all preset colors", () => {
    const onChange = vi.fn();
    render(<ColorPicker onChange={onChange} />);

    // 10 preset colors
    const presetButtons = screen.getAllByRole("button").filter((btn) => btn.hasAttribute("title"));
    expect(presetButtons.length).toBe(10);
  });

  it("calls onChange with color when clicking a preset", () => {
    const onChange = vi.fn();
    render(<ColorPicker onChange={onChange} />);

    const redButton = screen.getByTitle("#ef4444");
    fireEvent.click(redButton);

    expect(onChange).toHaveBeenCalledWith("#ef4444");
  });

  it("calls onChange with undefined when clicking the selected color (toggle off)", () => {
    const onChange = vi.fn();
    render(<ColorPicker value="#ef4444" onChange={onChange} />);

    const redButton = screen.getByTitle("#ef4444");
    fireEvent.click(redButton);

    expect(onChange).toHaveBeenCalledWith(undefined);
  });

  it("shows clear button when value is provided", () => {
    const onChange = vi.fn();
    render(<ColorPicker value="#ef4444" onChange={onChange} />);

    const clearButton = screen.getByText("common.actions.clear");
    expect(clearButton).toBeTruthy();
  });

  it("does not show clear button when value is undefined", () => {
    const onChange = vi.fn();
    render(<ColorPicker onChange={onChange} />);

    const clearButton = screen.queryByText("common.actions.clear");
    expect(clearButton).toBeNull();
  });

  it("calls onChange with undefined when clicking clear button", () => {
    const onChange = vi.fn();
    render(<ColorPicker value="#ef4444" onChange={onChange} />);

    const clearButton = screen.getByText("common.actions.clear");
    fireEvent.click(clearButton);

    expect(onChange).toHaveBeenCalledWith(undefined);
  });

  it("toggles custom color input visibility", () => {
    const onChange = vi.fn();
    render(<ColorPicker onChange={onChange} />);

    const customButton = screen.getByText("connection.customColor");
    expect(screen.queryByPlaceholderText("#3b82f6")).toBeNull();

    fireEvent.click(customButton);
    expect(screen.getByPlaceholderText("#3b82f6")).toBeTruthy();

    fireEvent.click(customButton);
    expect(screen.queryByPlaceholderText("#3b82f6")).toBeNull();
  });

  it("validates and accepts valid hex color on Enter", () => {
    const onChange = vi.fn();
    render(<ColorPicker onChange={onChange} />);

    const customButton = screen.getByText("connection.customColor");
    fireEvent.click(customButton);

    const input = screen.getByPlaceholderText("#3b82f6");
    fireEvent.change(input, { target: { value: "#ff5733" } });
    fireEvent.keyDown(input, { key: "Enter" });

    expect(onChange).toHaveBeenCalledWith("#ff5733");
  });

  it("validates and accepts valid hex color on confirm button click", () => {
    const onChange = vi.fn();
    render(<ColorPicker onChange={onChange} />);

    const customButton = screen.getByText("connection.customColor");
    fireEvent.click(customButton);

    const input = screen.getByPlaceholderText("#3b82f6");
    fireEvent.change(input, { target: { value: "#123abc" } });

    const confirmButton = screen.getByText("common.actions.confirm");
    fireEvent.click(confirmButton);

    expect(onChange).toHaveBeenCalledWith("#123abc");
  });

  it("rejects invalid hex color", () => {
    const onChange = vi.fn();
    render(<ColorPicker onChange={onChange} />);

    const customButton = screen.getByText("connection.customColor");
    fireEvent.click(customButton);

    const input = screen.getByPlaceholderText("#3b82f6");
    fireEvent.change(input, { target: { value: "invalid" } });
    fireEvent.keyDown(input, { key: "Enter" });

    expect(onChange).not.toHaveBeenCalled();
  });

  it("rejects hex color with wrong length", () => {
    const onChange = vi.fn();
    render(<ColorPicker onChange={onChange} />);

    const customButton = screen.getByText("connection.customColor");
    fireEvent.click(customButton);

    const input = screen.getByPlaceholderText("#3b82f6");
    fireEvent.change(input, { target: { value: "#fff" } });
    fireEvent.keyDown(input, { key: "Enter" });

    expect(onChange).not.toHaveBeenCalled();
  });

  it("trims whitespace before validating hex color", () => {
    const onChange = vi.fn();
    render(<ColorPicker onChange={onChange} />);

    const customButton = screen.getByText("connection.customColor");
    fireEvent.click(customButton);

    const input = screen.getByPlaceholderText("#3b82f6");
    fireEvent.change(input, { target: { value: "  #aabbcc  " } });
    fireEvent.keyDown(input, { key: "Enter" });

    expect(onChange).toHaveBeenCalledWith("#aabbcc");
  });
});
