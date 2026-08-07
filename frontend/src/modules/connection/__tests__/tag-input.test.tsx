import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

import { TagInput } from "../components/tag-input";

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("TagInput", () => {
  it("renders existing tags as badges", () => {
    const onChange = vi.fn();
    render(<TagInput tags={["production", "primary"]} onChange={onChange} />);

    expect(screen.getByText("production")).toBeTruthy();
    expect(screen.getByText("primary")).toBeTruthy();
  });

  it("adds a tag on Enter", () => {
    const onChange = vi.fn();
    render(<TagInput tags={[]} onChange={onChange} />);

    const input = screen.getByPlaceholderText("connection.addTag");
    fireEvent.change(input, { target: { value: "new-tag" } });
    fireEvent.keyDown(input, { key: "Enter" });

    expect(onChange).toHaveBeenCalledWith(["new-tag"]);
  });

  it("does not add duplicate tags", () => {
    const onChange = vi.fn();
    render(<TagInput tags={["existing"]} onChange={onChange} />);

    const input = screen.getByPlaceholderText("connection.addTag");
    fireEvent.change(input, { target: { value: "existing" } });
    fireEvent.keyDown(input, { key: "Enter" });

    expect(onChange).not.toHaveBeenCalled();
  });

  it("does not add empty or whitespace-only tags", () => {
    const onChange = vi.fn();
    render(<TagInput tags={[]} onChange={onChange} />);

    const input = screen.getByPlaceholderText("connection.addTag");
    fireEvent.change(input, { target: { value: "   " } });
    fireEvent.keyDown(input, { key: "Enter" });

    expect(onChange).not.toHaveBeenCalled();
  });

  it("removes a tag when clicking the remove button", () => {
    const onChange = vi.fn();
    render(<TagInput tags={["alpha", "beta"]} onChange={onChange} />);

    const removeBtn = screen.getByLabelText("Remove tag alpha");
    fireEvent.click(removeBtn);

    expect(onChange).toHaveBeenCalledWith(["beta"]);
  });

  it("removes last tag on Backspace when input is empty", () => {
    const onChange = vi.fn();
    render(<TagInput tags={["alpha", "beta"]} onChange={onChange} />);

    const input = screen.getByPlaceholderText("connection.addTag");
    fireEvent.keyDown(input, { key: "Backspace" });

    expect(onChange).toHaveBeenCalledWith(["alpha"]);
  });

  it("does not remove tags on Backspace when input has text", () => {
    const onChange = vi.fn();
    render(<TagInput tags={["alpha", "beta"]} onChange={onChange} />);

    const input = screen.getByPlaceholderText("connection.addTag");
    fireEvent.change(input, { target: { value: "some" } });
    fireEvent.keyDown(input, { key: "Backspace" });

    expect(onChange).not.toHaveBeenCalled();
  });

  it("does not remove tags on Backspace when no tags exist", () => {
    const onChange = vi.fn();
    render(<TagInput tags={[]} onChange={onChange} />);

    const input = screen.getByPlaceholderText("connection.addTag");
    fireEvent.keyDown(input, { key: "Backspace" });

    expect(onChange).not.toHaveBeenCalled();
  });

  it("clears input after adding a tag", () => {
    const onChange = vi.fn();
    render(<TagInput tags={[]} onChange={onChange} />);

    const input = screen.getByPlaceholderText("connection.addTag") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "new-tag" } });
    fireEvent.keyDown(input, { key: "Enter" });

    expect(input.value).toBe("");
  });
});
