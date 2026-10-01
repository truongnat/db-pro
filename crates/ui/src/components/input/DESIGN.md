# Input design

`input::ui` is the presentation entry for Input, SearchInput, PasswordInput, and Textarea; its private submodules each own egui layout, response handling, and painting. The public `input::layout` module remains the stable layout facade. Pure label/counter and field-state decisions live in the handler and shared input token contract.

Single-line Input collects focus, hover, enabled, and error state, then applies the precedence `disabled → error → focus → hover → rest` before painting the border. Its visible label and helper/error text are separate from the editable control; the semantic name uses an explicit access label first, then visible label, then placeholder. Password reveal is a separate labeled button backed by caller-owned visibility state. SearchInput owns its local clear button behavior. Textarea delegates editing to egui and counts Unicode scalar values for its optional character counter without truncating text.

egui owns text editing, cursor movement, Tab traversal, and keyboard activation. Disabled fields do not show focus/error chrome. The four old radius/spacing aliases were removed; Input reads `RADIUS_XS`, `SPACE_SM`, and `SPACE_XS` directly from shared tokens. Rendering cost follows the text's measured/layout size; no field stores a second copy of user content.
