# Select design

`mod.rs` keeps the public component entry point. `ui.rs` and `ui/` own egui layout, focus, popup painting, and the option list. `handler.rs` owns selection and dismissal decisions plus popup geometry; `config.rs` contains Select-specific sizing values. Shared spacing, typography, and colors come from tokens and `DbProTheme`.

The trigger reports its label and selected value as widget metadata. Pointer and keyboard input are converted into handler signals; the handler clamps arrow navigation to loaded options, resolves click selection, and closes the popup for Escape, Enter, or outside clicks. The optional “Load more” row stays outside the selectable option count and sets the caller's request flag.

Rendering measures the option labels and constructs a bounded popup per visible frame. The list height is capped at eight rows, and the popup chooses the side with available screen space. Large option collections still require caller-side paging; this component does not virtualize the supplied slice. Long trigger labels are clipped within the measured trigger region.

## Interaction and accessibility

The trigger is keyboard-focusable and exposes an accessible name using `.label(...)` plus the selected label. Arrow keys navigate loaded entries; Enter selects or opens, Escape closes, and clicking outside dismisses. Disabled styling is inherited from the egui response/theme. Ensure labels distinguish choices and provide a stable unique ID salt. Reduced motion does not affect this non-animated popup.
