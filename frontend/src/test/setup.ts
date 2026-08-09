import "@testing-library/jest-dom/vitest";

// Polyfill localStorage for Node.js v22+ where localStorage is unavailable
// without --localstorage-file. jsdom normally provides this, but Node's native
// getter may take precedence. Zustand persist middleware requires a working storage.
if (typeof globalThis.localStorage === "undefined" || globalThis.localStorage === null) {
  const memStore = new Map<string, string>();
  const memLocalStorage = {
    getItem(key: string) { return memStore.get(key) ?? null; },
    setItem(key: string, value: string) { memStore.set(key, String(value)); },
    removeItem(key: string) { memStore.delete(key); },
    clear() { memStore.clear(); },
    get length() { return memStore.size; },
    key(i: number) { return [...memStore.keys()][i] ?? null; },
  };
  Object.defineProperty(globalThis, "localStorage", { value: memLocalStorage, writable: true, configurable: true });
}

globalThis.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
};

Element.prototype.hasPointerCapture =
  Element.prototype.hasPointerCapture ??
  function () {
    return false;
  };
Element.prototype.setPointerCapture = Element.prototype.setPointerCapture ?? function () {};
Element.prototype.releasePointerCapture = Element.prototype.releasePointerCapture ?? function () {};
Element.prototype.scrollIntoView = Element.prototype.scrollIntoView ?? function () {};
