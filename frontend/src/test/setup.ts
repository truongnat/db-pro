import "@testing-library/jest-dom/vitest";

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
