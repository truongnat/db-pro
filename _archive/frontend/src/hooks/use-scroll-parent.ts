import { useEffect, useState } from "react";

const SCROLLABLE_OVERFLOW = new Set(["auto", "scroll", "overlay"]);

export interface ScrollParent {
  /** Nearest scrollable ancestor, or `null` until it has been resolved. */
  scrollElement: HTMLElement | null;
  /** Distance from the top of that ancestor to the observed element. */
  scrollMargin: number;
}

/**
 * Resolves the nearest scrollable ancestor of an element.
 *
 * Virtualised lists inside a panel (the Explorer tree, sidebar search results)
 * must window against the *panel's* scroller. Giving each list its own
 * `max-h-* overflow-y-auto` box produces a scroll area inside a scroll area,
 * which is the classic tree-explorer trap: the wheel is captured by whichever
 * box happens to be under the cursor, and the outer panel stops responding.
 *
 * Returns `scrollMargin` so callers can hand it to `useVirtualizer` and
 * subtract it from `item.start` — the virtualizer measures from the scroll
 * element, not from the list's own top edge.
 */
export function useScrollParent(ref: { readonly current: HTMLElement | null }): ScrollParent {
  const [scrollParent, setScrollParent] = useState<ScrollParent>({
    scrollElement: null,
    scrollMargin: 0,
  });

  useEffect(() => {
    const node = ref.current;
    if (!node) return;

    // Only an element the walk *stopped on because it scrolls* counts. Walking
    // off the top of the tree leaves `ancestor` pointing at <body>, which is
    // truthy but is not a scroll container we should window against.
    let found: HTMLElement | null = null;
    for (
      let ancestor = node.parentElement;
      ancestor && ancestor !== document.body;
      ancestor = ancestor.parentElement
    ) {
      if (SCROLLABLE_OVERFLOW.has(window.getComputedStyle(ancestor).overflowY)) {
        found = ancestor;
        break;
      }
    }
    if (!found) return;

    const scrollMargin =
      node.getBoundingClientRect().top - found.getBoundingClientRect().top + found.scrollTop;

    setScrollParent({ scrollElement: found, scrollMargin });
    // Resolved once per mount: a panel's scroll container does not move, and
    // re-walking the DOM on every render would defeat the purpose.
  }, []);

  return scrollParent;
}
