# ScrollArea design

`ScrollArea` is a thin builder around egui's native scroll container. It translates horizontal/vertical flags to egui's `[horizontal, vertical]` ordering, applies auto-shrink and optional maximum height, then renders the caller closure. The handler temporarily adjusts scrollbar clip visuals and restores the caller's previous visuals before returning.

The component adds no custom scrollbar painting, keyboard model, or retained data state. egui owns pointer wheel, drag, and keyboard scroll behavior. Keep the child content's focus and accessible labels in the child widgets; the scroll wrapper does not add a role.

Rendering cost is dominated by child content and egui's visible-region layout. The wrapper itself performs constant setup and stores no content copy. It is suitable for ordinary bounded lists; large table-like data should use a virtualized surface.
