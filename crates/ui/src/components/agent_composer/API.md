# AgentComposer API

Public types are re-exported from `db_pro_ui::components` and the existing
`components::agent_composer` module path.

## Builder and result

```rust
AgentComposer::new(&mut prompt, selected_model, selected_mode, theme)
    .is_generating(is_generating)
    .token_usage(token_count)
    .show(ui)
```

- `prompt: &mut String` is edited in place. `selected_model` is borrowed text;
  `selected_mode` is a copied `AgentMode`; `theme` is `DbProTheme`.
- `.is_generating(bool)` selects the Stop action while generation is active.
- `.token_usage(usize)` displays token usage; zero is treated as absent.
- `.show(&mut egui::Ui) -> Option<AgentComposerAction>` returns `Some(Submit)`
  for a valid submit or `Some(Stop)` for a stop click; it returns `None` when no
  action is requested.
- `AgentMode::{Chat, Plan, Code}` labels itself as `Chat`, `Plan`, or
  `SQL Agent` respectively.
- `AgentComposerAction::{Submit, Stop, Clear}` is retained for API
  compatibility. The current component emits `Submit` and `Stop`; it does not
  emit `Clear`. Callers may still match `Clear` for compatibility.

## Input and state rules

- Enter submits only when the editor loses focus, Shift is not held, and
  `prompt.trim()` is nonempty. Shift+Enter inserts a newline.
- An idle composer does not submit an empty or whitespace-only prompt, even if
  the button signal reaches the handler. Send is disabled for that state.
- While generating, the action button returns `Stop` regardless of prompt
  contents.
- The caller owns request submission, cancellation, and prompt clearing. A
  returned action is a UI intent, not a completed provider operation.
- The editor is multiline and has no component-defined character limit. Very
  long prompts can require substantial layout and text-edit work each frame.

## Accessibility and layout

The editor exposes the accessible name `Prompt`; icon actions expose `Send` and
`Stop`. Toolbar controls wrap at narrow widths. Runtime keyboard traversal,
focus behavior, and visual contrast still require native-app verification.
