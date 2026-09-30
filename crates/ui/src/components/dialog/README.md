# Dialog Components

Native egui modal dialog and right-side sheet primitives. The caller owns each `open` flag and performs any domain action returned by its content controls; these components manage presentation, modal focus, Escape, backdrop behavior, and animation.

## Public API

- `Dialog`, `DialogFrame`, `DialogActionLabels`, `dialog_actions`, and `close_icon_button` are available from `components::dialog::*` and the compatibility path `components::dialog::modal::*`.
- `Sheet` is available from `components::dialog::Sheet` and `components::dialog::sheet::Sheet`.
- `Dialog::show` / `show_ctx` provide a scrollable body. `show_framed` / `show_framed_ctx` expose `DialogFrame` for a sticky footer.
- `Sheet::show` renders a right-side panel.

## Behavior contract

- Set the caller's `open` flag to control visibility. A stable `.id_salt(...)` is recommended when multiple dialogs/sheets can be active from the same call site.
- Dialog Escape and backdrop dismissal belong only to the topmost modal. Backdrop clicks inside the card do not dismiss it; a missing pointer position on a registered backdrop click retains the current close behavior.
- Sheets share topmost Escape routing and focus trapping with dialogs. They intentionally do not dismiss on backdrop clicks; use Escape or the close button.
- Dismissal decisions are typed in `handler.rs`; modal stacking/focus coordination is shared through `modal_guard.rs`, while egui overlay, layout and painting remain in the UI modules.

## Example

```rust
use crate::components::dialog::{dialog_actions, Dialog, DialogActionLabels};

let mut open = true;
Dialog::new(&mut open, "Create index", theme)
    .id_salt("create-index")
    .show(ui, |ui| {
        ui.label("Configure the index before applying it.");
        dialog_actions(
            ui,
            theme,
            DialogActionLabels {
                secondary: "Cancel",
                primary: "Create",
            },
        )
    });
```

The returned content value is optional: it is `None` while closed/fully hidden and `Some(value)` while the overlay is rendered.
