import { describe, expect, it, vi } from "vitest";

const { mockApiInvoke } = vi.hoisted(() => ({ mockApiInvoke: vi.fn() }));

vi.mock("@/commons/utils/api", () => ({
  apiInvoke: mockApiInvoke,
}));

import { BackupService, createBackupService } from "../services/backup.service";

describe("BackupService", () => {
  describe("backup", () => {
    it("calls apiInvoke with correct command and params", async () => {
      const result = { outputPath: "/tmp/backup.sql", sizeBytes: 1024 };
      mockApiInvoke.mockResolvedValueOnce(result);

      const svc = new BackupService();
      const res = await svc.backup({
        connectionId: "conn-1",
        outputPath: "/tmp/backup.sql",
        format: "plain",
        schemas: ["public"],
        tables: ["users"],
      });

      expect(res).toEqual(result);
      expect(mockApiInvoke).toHaveBeenCalledWith("backup_database", {
        req: {
          connectionId: "conn-1",
          outputPath: "/tmp/backup.sql",
          format: "plain",
          schemas: ["public"],
          tables: ["users"],
        },
      });
    });

    it("works without optional schemas/tables", async () => {
      mockApiInvoke.mockResolvedValueOnce({ outputPath: "/tmp/b.sql", sizeBytes: 0 });

      const svc = new BackupService();
      await svc.backup({
        connectionId: "conn-1",
        outputPath: "/tmp/b.sql",
        format: "custom",
      });

      expect(mockApiInvoke).toHaveBeenCalledWith("backup_database", {
        req: {
          connectionId: "conn-1",
          outputPath: "/tmp/b.sql",
          format: "custom",
          schemas: undefined,
          tables: undefined,
        },
      });
    });
  });

  describe("restore", () => {
    it("calls apiInvoke with correct command and params", async () => {
      mockApiInvoke.mockResolvedValueOnce(undefined);

      const svc = new BackupService();
      await svc.restore({
        connectionId: "conn-1",
        inputPath: "/tmp/backup.sql",
        format: "plain",
      });

      expect(mockApiInvoke).toHaveBeenCalledWith("restore_database", {
        req: {
          connectionId: "conn-1",
          inputPath: "/tmp/backup.sql",
          format: "plain",
        },
      });
    });
  });

  describe("createBackupService", () => {
    it("returns a BackupService instance", () => {
      const svc = createBackupService();
      expect(svc).toBeInstanceOf(BackupService);
    });
  });
});
