# AgentComposer

`AgentComposer` is the native egui prompt composer used by the agent surface. It
renders a multiline SQL/agent prompt editor, model and mode badges, optional token
usage, and a Send/Stop icon button.

## Public API

- `AgentComposer::new(prompt, selected_model, selected_mode, theme)` creates a composer borrowing the editable prompt and selected labels.
- `.is_generating(bool)` switches the action button from Send to Stop.
- `.token_usage(usize)` displays the optional token count.
- `.show(ui)` paints the component and returns `Option<AgentComposerAction>`.
- `AgentMode` keeps the public Chat, Plan, and Code modes; Code is displayed as `SQL Agent`.
- `AgentComposerAction` contains `Submit`, `Stop`, and `Clear` for caller compatibility. The current component emits `Submit` and `Stop`; `Clear` is retained but not emitted.

## Behavior

- Enter submits only when the editor loses focus, Shift is not held, and the prompt is nonblank; Shift+Enter remains available for multiline input.
- Blank or whitespace-only prompts keep Send disabled and never produce `Submit`, including through the keyboard path.
- A generating composer exposes the destructive Stop action regardless of prompt text.
- The editor exposes the accessible `Prompt` label; icon buttons retain the accessible `Send` and `Stop` labels.
- `DbProTheme` and shared token values provide colors, typography, spacing, and the frame style. Badge dimensions are local to this component.

## Layering

`ui.rs` owns egui widgets, text measurement, layout, and painting. `handler.rs`
contains typed prompt/action decisions and badge geometry. `config.rs` contains
only component-owned badge geometry, while `mod.rs` preserves the public entry
point and API.

See [DESIGN.md](DESIGN.md) for the event and rendering flow and [API.md](API.md)
for the full contract and limitations.

## Usage

```rust
use db_pro_ui::components::{AgentComposer, AgentComposerAction, AgentMode};

let mut prompt = String::new();
if let Some(action) = AgentComposer::new(&mut prompt, "GPT-4o", AgentMode::Chat, theme)
    .is_generating(false)
    .show(ui)
{
    match action {
        AgentComposerAction::Submit => submit_prompt(&prompt),
        AgentComposerAction::Stop => cancel_generation(),
        AgentComposerAction::Clear => prompt.clear(),
    }
}
```
