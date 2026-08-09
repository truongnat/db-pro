import { beforeEach, describe, expect, it, vi } from "vitest";

/* ----------------------------------------------------------------
 * Mock navigator.platform per-test so we can exercise both the
 * macOS symbol path and the Windows/Linux text path.
 *
 * vi.doMock is NOT hoisted; combined with vi.resetModules() each
 * test gets a fresh evaluation of platform.ts with the desired
 * isMac value.
 * ---------------------------------------------------------------- */

let mockPlatform: string;

vi.mock("lucide-react", () => ({
  Save: () => null,
}));

beforeEach(() => {
  vi.resetModules();
  mockPlatform = "Linux x86_64";
  Object.defineProperty(navigator, "platform", {
    get: () => mockPlatform,
    configurable: true,
  });
});

/* ---- helpers ---- */

async function loadMac() {
  mockPlatform = "MacIntel";
  return await import("../platform");
}

async function loadNonMac() {
  mockPlatform = "Linux x86_64";
  return await import("../platform");
}

/* ================================================================
 * Tests
 * ============================================================= */

describe("isMac", () => {
  it("returns true for MacIntel", async () => {
    const mod = await loadMac();
    expect(mod.isMac).toBe(true);
  });

  it("returns false for Linux", async () => {
    const mod = await loadNonMac();
    expect(mod.isMac).toBe(false);
  });

  it("returns true for MacPPC", async () => {
    mockPlatform = "MacPPC";
    const mod = await import("../platform");
    expect(mod.isMac).toBe(true);
  });
});

describe("formatShortcut — macOS", () => {
  it("primary → ⌘", async () => {
    const { formatShortcut } = await loadMac();
    expect(formatShortcut({ primary: true, key: "P" })).toBe("⌘P");
  });

  it("primary + shift → ⌘⇧", async () => {
    const { formatShortcut } = await loadMac();
    expect(formatShortcut({ primary: true, shiftKey: true, key: "Enter" })).toBe("⌘⇧↵");
  });

  it("ctrlKey (real Control) → ⌃", async () => {
    const { formatShortcut } = await loadMac();
    expect(formatShortcut({ ctrlKey: true, key: "Enter" })).toBe("⌃↵");
  });

  it("primary only, no ctrlKey symbol", async () => {
    const { formatShortcut } = await loadMac();
    const result = formatShortcut({ primary: true, key: "Enter" });
    expect(result).toBe("⌘↵");
    expect(result).not.toContain("⌃");
  });

  it("altKey → ⌥", async () => {
    const { formatShortcut } = await loadMac();
    expect(formatShortcut({ altKey: true, key: "a" })).toBe("⌥A");
  });

  it("all modifiers in canonical order", async () => {
    const { formatShortcut } = await loadMac();
    // canonical: ⌃ ⌘ ⇧ ⌥ <key>
    expect(
      formatShortcut({ ctrlKey: true, primary: true, shiftKey: true, altKey: true, key: "a" }),
    ).toBe("⌃⌘⇧⌥A");
  });

  it("maps symbol keys correctly", async () => {
    const { formatShortcut } = await loadMac();
    expect(formatShortcut({ key: "Enter" })).toBe("↵");
    expect(formatShortcut({ key: "Escape" })).toBe("Esc");
    expect(formatShortcut({ key: "Backspace" })).toBe("⌫");
    expect(formatShortcut({ key: "Delete" })).toBe("⌦");
    expect(formatShortcut({ key: "ArrowUp" })).toBe("↑");
    expect(formatShortcut({ key: "ArrowDown" })).toBe("↓");
    expect(formatShortcut({ key: "ArrowLeft" })).toBe("←");
    expect(formatShortcut({ key: "ArrowRight" })).toBe("→");
    expect(formatShortcut({ key: "Tab" })).toBe("⇥");
  });

  it("single-letter key is uppercased", async () => {
    const { formatShortcut } = await loadMac();
    expect(formatShortcut({ key: "p" })).toBe("P");
    expect(formatShortcut({ key: "k" })).toBe("K");
  });
});

describe("formatShortcut — Windows/Linux", () => {
  it("primary → Ctrl", async () => {
    const { formatShortcut } = await loadNonMac();
    expect(formatShortcut({ primary: true, key: "P" })).toBe("Ctrl+P");
  });

  it("primary + shift → Ctrl+Shift", async () => {
    const { formatShortcut } = await loadNonMac();
    expect(formatShortcut({ primary: true, shiftKey: true, key: "Enter" })).toBe(
      "Ctrl+Shift+Enter",
    );
  });

  it("ctrlKey → Ctrl", async () => {
    const { formatShortcut } = await loadNonMac();
    expect(formatShortcut({ ctrlKey: true, key: "Enter" })).toBe("Ctrl+Enter");
  });

  it("primary and ctrlKey both collapse to single Ctrl", async () => {
    const { formatShortcut } = await loadNonMac();
    expect(formatShortcut({ primary: true, ctrlKey: true, key: "A" })).toBe("Ctrl+A");
  });

  it("altKey → Alt", async () => {
    const { formatShortcut } = await loadNonMac();
    expect(formatShortcut({ altKey: true, key: "a" })).toBe("Alt+a");
  });

  it("metaKey → Win", async () => {
    const { formatShortcut } = await loadNonMac();
    expect(formatShortcut({ metaKey: true, key: "E" })).toBe("Win+E");
  });

  it("all modifiers in canonical order", async () => {
    const { formatShortcut } = await loadNonMac();
    // canonical: Ctrl Shift Alt Win <key>
    expect(
      formatShortcut({
        ctrlKey: true,
        primary: true,
        shiftKey: true,
        altKey: true,
        metaKey: true,
        key: "A",
      }),
    ).toBe("Ctrl+Shift+Alt+Win+A");
  });

  it("does not use symbol notation", async () => {
    const { formatShortcut } = await loadNonMac();
    expect(formatShortcut({ primary: true, key: "Enter" })).toBe("Ctrl+Enter");
    expect(formatShortcut({ primary: true, key: "Enter" })).not.toContain("↵");
  });
});

describe("formatShortcut — real-world shortcuts", () => {
  it("Run Current: ⌘↵ on macOS", async () => {
    const { formatShortcut } = await loadMac();
    expect(formatShortcut({ primary: true, key: "Enter" })).toBe("⌘↵");
  });

  it("Run All: ⌘⇧↵ on macOS", async () => {
    const { formatShortcut } = await loadMac();
    expect(formatShortcut({ primary: true, shiftKey: true, key: "Enter" })).toBe("⌘⇧↵");
  });

  it("Quick Open: ⌘P on macOS", async () => {
    const { formatShortcut } = await loadMac();
    expect(formatShortcut({ primary: true, key: "P" })).toBe("⌘P");
  });

  it("Command Palette: ⌘⇧P on macOS", async () => {
    const { formatShortcut } = await loadMac();
    expect(formatShortcut({ primary: true, shiftKey: true, key: "P" })).toBe("⌘⇧P");
  });

  it("Run Current: Ctrl+Enter on Windows", async () => {
    const { formatShortcut } = await loadNonMac();
    expect(formatShortcut({ primary: true, key: "Enter" })).toBe("Ctrl+Enter");
  });

  it("Run All: Ctrl+Shift+Enter on Windows", async () => {
    const { formatShortcut } = await loadNonMac();
    expect(formatShortcut({ primary: true, shiftKey: true, key: "Enter" })).toBe(
      "Ctrl+Shift+Enter",
    );
  });
});

describe("formatShortcut — no modifiers", () => {
  it("returns just the symbol key on macOS", async () => {
    const { formatShortcut } = await loadMac();
    expect(formatShortcut({ key: "Enter" })).toBe("↵");
  });

  it("returns just the key name on Windows", async () => {
    const { formatShortcut } = await loadNonMac();
    expect(formatShortcut({ key: "Enter" })).toBe("Enter");
  });
});
