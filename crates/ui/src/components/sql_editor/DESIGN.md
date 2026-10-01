# SqlEditorToolbar design

`mod.rs` exposes `SqlEditorToolbar` and `SqlEditorAction`. `ui.rs` paints a compact toolbar and converts shared Button clicks to actions; `handler.rs` maps each action to its label, icon, visual variant, accessible label, and visibility policy; `config.rs` holds only toolbar-specific margin and corner composition. Shared stroke, spacing, and button size are used directly from tokens and Button.

## Event flow

The caller supplies `is_running` and `has_selection` for the current frame. When idle, the toolbar emits Run, optional Run Selection, Explain, or Format actions. While running, those primary controls are replaced by Cancel. Ask AI remains available on the trailing side in either state. A click returns one `SqlEditorAction`; the toolbar does not execute, cancel, format, or explain a query itself.

## Layout, cost, and accessibility

The toolbar paints one wrapped row of shared Button widgets. Wrapping lets actions fit in a narrow panel. Its cost is constant for the visible actions. Shared buttons provide keyboard activation and accessible descriptions; selection gating uses current caller state and does not inspect the editor buffer. Callers should update running/selection state from authoritative editor/runtime state and dispatch returned intents once.
