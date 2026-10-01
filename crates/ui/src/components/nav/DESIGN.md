# Navigation design

Pagination owns a mutable page reference for the duration of rendering. On each `show`, it clamps the page to the 1-based range, paints previous/next icon buttons with directional accessible labels, and applies click transitions through typed handlers. Breadcrumb owns no navigation state: it paints supplied items and returns the index clicked by the caller. Current crumbs are labels; other crumbs are keyboard-operable buttons. PageHeader and SectionHeader are static text composition.

Buttons include visible focus and disabled state; pagination disables movement at either boundary. Breadcrumbs wrap in narrow layouts and can contain multiple current items, each rendered non-interactively. Colors and typography use the shared theme. There are no repeating animations.

Pagination has fixed widget cost. Breadcrumb rendering is O(n) in item count; the caller remains responsible for dispatching returned indices and keeping the page model in sync.
