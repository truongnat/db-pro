import { beforeEach, describe, expect, it } from "vitest";

import { useCrashRecoveryStore } from "@/commons/stores/crash-recovery.store";

function resetStore() {
  useCrashRecoveryStore.getState().clearAll();
}

describe("Crash recovery store", () => {
  beforeEach(() => {
    resetStore();
  });

  it("starts with no snapshots", () => {
    expect(useCrashRecoveryStore.getState().snapshots).toEqual([]);
  });

  it("saves a snapshot", () => {
    useCrashRecoveryStore.getState().saveSnapshot("tab1", "c1", "Query 1", "SELECT 1");
    const snapshots = useCrashRecoveryStore.getState().snapshots;
    expect(snapshots).toHaveLength(1);
    expect(snapshots[0].tabId).toBe("tab1");
    expect(snapshots[0].connectionId).toBe("c1");
    expect(snapshots[0].title).toBe("Query 1");
    expect(snapshots[0].sql).toBe("SELECT 1");
    expect(snapshots[0].timestamp).toBeGreaterThan(0);
  });

  it("replaces existing snapshot for same tab", () => {
    useCrashRecoveryStore.getState().saveSnapshot("tab1", "c1", "Query 1", "SELECT 1");
    useCrashRecoveryStore.getState().saveSnapshot("tab1", "c1", "Query 1", "SELECT 2");
    const snapshots = useCrashRecoveryStore.getState().snapshots;
    expect(snapshots).toHaveLength(1);
    expect(snapshots[0].sql).toBe("SELECT 2");
  });

  it("stores snapshots for different tabs", () => {
    useCrashRecoveryStore.getState().saveSnapshot("tab1", "c1", "Query 1", "SELECT 1");
    useCrashRecoveryStore.getState().saveSnapshot("tab2", "c1", "Query 2", "SELECT 2");
    expect(useCrashRecoveryStore.getState().snapshots).toHaveLength(2);
  });

  it("removes a snapshot by tab id", () => {
    useCrashRecoveryStore.getState().saveSnapshot("tab1", "c1", "Query 1", "SELECT 1");
    useCrashRecoveryStore.getState().saveSnapshot("tab2", "c1", "Query 2", "SELECT 2");
    useCrashRecoveryStore.getState().removeSnapshot("tab1");
    const snapshots = useCrashRecoveryStore.getState().snapshots;
    expect(snapshots).toHaveLength(1);
    expect(snapshots[0].tabId).toBe("tab2");
  });

  it("clears all snapshots", () => {
    useCrashRecoveryStore.getState().saveSnapshot("tab1", "c1", "Query 1", "SELECT 1");
    useCrashRecoveryStore.getState().saveSnapshot("tab2", "c1", "Query 2", "SELECT 2");
    useCrashRecoveryStore.getState().clearAll();
    expect(useCrashRecoveryStore.getState().snapshots).toEqual([]);
  });

  it("limits snapshots to MAX_SNAPSHOTS (50)", () => {
    for (let i = 0; i < 60; i++) {
      useCrashRecoveryStore.getState().saveSnapshot(`tab${i}`, "c1", `Query ${i}`, `SELECT ${i}`);
    }
    expect(useCrashRecoveryStore.getState().snapshots.length).toBeLessThanOrEqual(50);
  });

  it("supports null connectionId", () => {
    useCrashRecoveryStore.getState().saveSnapshot("tab1", null, "Query 1", "SELECT 1");
    const snapshot = useCrashRecoveryStore.getState().snapshots[0];
    expect(snapshot.connectionId).toBeNull();
  });

  it("orders snapshots newest first", () => {
    useCrashRecoveryStore.getState().saveSnapshot("tab1", "c1", "First", "SELECT 1");
    useCrashRecoveryStore.getState().saveSnapshot("tab2", "c1", "Second", "SELECT 2");
    const snapshots = useCrashRecoveryStore.getState().snapshots;
    expect(snapshots[0].tabId).toBe("tab2");
    expect(snapshots[1].tabId).toBe("tab1");
  });
});
