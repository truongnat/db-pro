import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

import { ZoomControls } from "../components/zoom-controls";

describe("ZoomControls", () => {
  it("renders current zoom percentage", () => {
    render(<ZoomControls zoom={100} onZoomChange={vi.fn()} />);
    expect(screen.getByText("100%")).toBeTruthy();
  });

  it("decreases zoom by STEP on minus click", () => {
    const onZoomChange = vi.fn();
    render(<ZoomControls zoom={100} onZoomChange={onZoomChange} />);

    const minusBtn = screen.getByText("−");
    fireEvent.click(minusBtn);

    expect(onZoomChange).toHaveBeenCalledWith(90);
  });

  it("increases zoom by STEP on plus click", () => {
    const onZoomChange = vi.fn();
    render(<ZoomControls zoom={100} onZoomChange={onZoomChange} />);

    const plusBtn = screen.getByText("+");
    fireEvent.click(plusBtn);

    expect(onZoomChange).toHaveBeenCalledWith(110);
  });

  it("clamps to MIN_ZOOM (50)", () => {
    const onZoomChange = vi.fn();
    render(<ZoomControls zoom={50} onZoomChange={onZoomChange} />);

    const minusBtn = screen.getByText("−");
    expect(minusBtn).toBeDisabled();
  });

  it("clamps to MAX_ZOOM (200)", () => {
    const onZoomChange = vi.fn();
    render(<ZoomControls zoom={200} onZoomChange={onZoomChange} />);

    const plusBtn = screen.getByText("+");
    expect(plusBtn).toBeDisabled();
  });

  it("does not go below MIN_ZOOM when clicking minus at boundary", () => {
    const onZoomChange = vi.fn();
    render(<ZoomControls zoom={55} onZoomChange={onZoomChange} />);

    fireEvent.click(screen.getByText("−"));
    expect(onZoomChange).toHaveBeenCalledWith(50);
  });

  it("does not go above MAX_ZOOM when clicking plus at boundary", () => {
    const onZoomChange = vi.fn();
    render(<ZoomControls zoom={195} onZoomChange={onZoomChange} />);

    fireEvent.click(screen.getByText("+"));
    expect(onZoomChange).toHaveBeenCalledWith(200);
  });
});
