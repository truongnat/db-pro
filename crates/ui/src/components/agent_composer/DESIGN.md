# AgentComposer design

`mod.rs` is the stable public entry point for `AgentComposer`, `AgentMode`, and
`AgentComposerAction`. `ui.rs` owns egui widgets, layout, painting, labels, and
focus metadata. `handler.rs` owns prompt validity, keyboard/button action
decisions, and the small model-badge geometry calculation. `config.rs` contains
only geometry used by that badge.

## Event flow

The multiline editor edits the caller-owned `String`. On Enter, egui reports
focus loss and key/modifier state; `keyboard_action` returns `Submit` only when
Enter caused focus loss, Shift is not held, and trimmed prompt text is nonempty.
Shift+Enter therefore remains text input. The Send/Stop button uses the same
`has_prompt_text` rule: while generating it returns `Stop`, otherwise a click on
a blank prompt produces no action. `show` returns an action to the caller and
does not submit a request itself.

The prompt editor has a persistent accessible `Prompt` label. Send and Stop
icon buttons expose text labels. The toolbar uses wrapped layout so its controls
can move to another row when the available width is small.

## Rendering cost

The component paints its editor, toolbar, and optional token/model/mode badges
once per egui frame. It does not allocate a per-frame collection or perform
provider work. Badge dimensions are computed from the label width and local
constants. Prompt content is held by the caller; this component does not impose
a maximum length or virtualize text editing.
