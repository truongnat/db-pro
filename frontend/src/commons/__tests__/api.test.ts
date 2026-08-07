import { describe, expect, it, vi } from "vitest";

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

import { apiInvoke } from "../utils/api";

describe("apiInvoke", () => {
  it("calls invoke with command and args", async () => {
    mockInvoke.mockResolvedValueOnce({ id: "1" });
    const result = await apiInvoke("get_connection", { id: "1" });
    expect(result).toEqual({ id: "1" });
    expect(mockInvoke).toHaveBeenCalledWith("get_connection", { id: "1" });
  });

  it("calls invoke without args when not provided", async () => {
    mockInvoke.mockResolvedValueOnce([]);
    const result = await apiInvoke("list_connections");
    expect(result).toEqual([]);
    expect(mockInvoke).toHaveBeenCalledWith("list_connections", undefined);
  });

  it("throws translated error on failure", async () => {
    const commandError = {
      error: "TIMEOUT",
      message: "query timed out",
      message_id: "error.timeout",
    };
    mockInvoke.mockRejectedValueOnce(commandError);

    await expect(apiInvoke("execute_query")).rejects.toMatchObject({
      code: "TIMEOUT",
      technicalMessage: "query timed out",
      messageId: "error.timeout",
    });
  });

  it("handles string errors as UNKNOWN", async () => {
    mockInvoke.mockRejectedValueOnce("something broke");

    await expect(apiInvoke("test_cmd")).rejects.toMatchObject({
      code: "UNKNOWN",
      technicalMessage: "something broke",
    });
  });

  it("handles Error instance with timeout detection", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("connection timeout"));

    await expect(apiInvoke("test_cmd")).rejects.toMatchObject({
      code: "TIMEOUT",
    });
  });
});
