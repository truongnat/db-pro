import { describe, expect, it } from "vitest";
import { DIContainer } from "../di/container";

describe("DIContainer", () => {
  describe("register and resolve", () => {
    it("resolves a registered singleton", () => {
      const container = new DIContainer();
      container.register("svc", () => ({ value: 42 }));
      const svc = container.resolve<{ value: number }>("svc");
      expect(svc.value).toBe(42);
    });

    it("returns the same instance for singletons", () => {
      const container = new DIContainer();
      let callCount = 0;
      container.register("svc", () => {
        callCount++;
        return { id: callCount };
      });
      const a = container.resolve("svc");
      const b = container.resolve("svc");
      expect(a).toBe(b);
      expect(callCount).toBe(1);
    });

    it("creates new instances for non-singletons", () => {
      const container = new DIContainer();
      let callCount = 0;
      container.register(
        "svc",
        () => {
          callCount++;
          return { id: callCount };
        },
        false,
      );
      const a = container.resolve<{ id: number }>("svc");
      const b = container.resolve<{ id: number }>("svc");
      expect(a.id).toBe(1);
      expect(b.id).toBe(2);
      expect(callCount).toBe(2);
    });
  });

  describe("error handling", () => {
    it("throws for unregistered service", () => {
      const container = new DIContainer();
      expect(() => container.resolve("missing")).toThrow('Service "missing" is not registered');
    });

    it("detects circular dependencies", () => {
      const container = new DIContainer();
      container.register("a", (c) => c.resolve("b"));
      container.register("b", (c) => c.resolve("a"));
      expect(() => container.resolve("a")).toThrow("Circular dependency");
    });

    it("throws when registering on frozen container", () => {
      const container = new DIContainer();
      container.freeze();
      expect(() => container.register("svc", () => ({}))).toThrow("frozen");
    });
  });

  describe("has", () => {
    it("returns true for registered service", () => {
      const container = new DIContainer();
      container.register("svc", () => ({}));
      expect(container.has("svc")).toBe(true);
    });

    it("returns false for unknown service", () => {
      const container = new DIContainer();
      expect(container.has("unknown")).toBe(false);
    });

    it("returns true for resolved singleton", () => {
      const container = new DIContainer();
      container.register("svc", () => ({}));
      container.resolve("svc");
      expect(container.has("svc")).toBe(true);
    });
  });

  describe("freeze", () => {
    it("allows resolving after freeze", () => {
      const container = new DIContainer();
      container.register("svc", () => ({ ok: true }));
      container.freeze();
      const svc = container.resolve<{ ok: boolean }>("svc");
      expect(svc.ok).toBe(true);
    });
  });
});
