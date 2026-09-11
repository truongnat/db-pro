import { describe, expect, it, vi, beforeEach } from "vitest";

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

import { ConnectionService, createConnectionService } from "../services/connection.service";
import type { ConnectionConfig } from "../types/connection.types";

const sampleConfig: ConnectionConfig = {
  name: "Test DB",
  host: "localhost",
  port: 5432,
  database: "testdb",
  username: "user",
  driver: "postgres",
  sslMode: "disable",
  queryTimeoutMs: 30000,
  maxRows: 500,
};

const sampleConnection = {
  id: "abc-123",
  ...sampleConfig,
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
};

describe("ConnectionService", () => {
  let service: ConnectionService;

  beforeEach(() => {
    service = new ConnectionService();
    mockInvoke.mockReset();
  });

  it("list calls list_connections", async () => {
    mockInvoke.mockResolvedValue([sampleConnection]);
    const result = await service.list();
    expect(mockInvoke).toHaveBeenCalledWith("list_connections", undefined);
    expect(result).toEqual([sampleConnection]);
  });

  it("get calls get_connection with id", async () => {
    mockInvoke.mockResolvedValue(sampleConnection);
    const result = await service.get("abc-123");
    expect(mockInvoke).toHaveBeenCalledWith("get_connection", { id: "abc-123" });
    expect(result).toEqual(sampleConnection);
  });

  it("get returns null when not found", async () => {
    mockInvoke.mockResolvedValue(null);
    const result = await service.get("nonexistent");
    expect(result).toBeNull();
  });

  it("create calls create_connection with config and password", async () => {
    mockInvoke.mockResolvedValue(sampleConnection);
    const result = await service.create(sampleConfig, "secret");
    expect(mockInvoke).toHaveBeenCalledWith("create_connection", {
      config: sampleConfig,
      password: "secret",
    });
    expect(result).toEqual(sampleConnection);
  });

  it("update calls update_connection", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await service.update("abc-123", sampleConfig, "new-secret");
    expect(mockInvoke).toHaveBeenCalledWith("update_connection", {
      id: "abc-123",
      config: sampleConfig,
      password: "new-secret",
    });
  });

  it("update sends undefined password when omitted", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await service.update("abc-123", sampleConfig);
    expect(mockInvoke).toHaveBeenCalledWith("update_connection", {
      id: "abc-123",
      config: sampleConfig,
      password: undefined,
    });
  });

  it("delete calls delete_connection with id", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await service.delete("abc-123");
    expect(mockInvoke).toHaveBeenCalledWith("delete_connection", { id: "abc-123" });
  });

  it("test calls test_connection with config and password", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await service.test(sampleConfig, "secret");
    expect(mockInvoke).toHaveBeenCalledWith("test_connection", {
      config: sampleConfig,
      password: "secret",
      connectionId: undefined,
    });
  });

  it("test passes connectionId when provided", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await service.test(sampleConfig, "", "conn-1");
    expect(mockInvoke).toHaveBeenCalledWith("test_connection", {
      config: sampleConfig,
      password: "",
      connectionId: "conn-1",
    });
  });

  it("connect calls connect with id", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await service.connect("abc-123");
    expect(mockInvoke).toHaveBeenCalledWith("connect", { id: "abc-123" });
  });

  it("disconnect calls disconnect with id", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await service.disconnect("abc-123");
    expect(mockInvoke).toHaveBeenCalledWith("disconnect", { id: "abc-123" });
  });

  it("testSshTunnel calls test_ssh_tunnel with config", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const sshConfig = { host: "ssh.example.com", port: 22, username: "deploy" };
    await service.testSshTunnel(sshConfig);
    expect(mockInvoke).toHaveBeenCalledWith("test_ssh_tunnel", { config: sshConfig });
  });

  it("createConnectionService returns a ConnectionService", () => {
    const svc = createConnectionService();
    expect(svc).toBeInstanceOf(ConnectionService);
  });
});
