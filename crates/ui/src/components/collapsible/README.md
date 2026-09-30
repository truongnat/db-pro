# Collapsible

Native interactive disclosure widget that reveals or conceals custom nested content with animated chevron and opacity transitions.

## Public API

- `Collapsible::new(open: &'a mut bool, theme: DbProTheme)`: Creates a new collapsible builder bound to a mutable boolean toggle state and theme.
- `.title(&'a str)`: Sets the label text displayed on the header row.
- `.icon(lucide_icons::Icon)`: Sets an optional leading Lucide icon between the chevron and title.
- `.badge(&'a str)`: Sets an optional trailing badge pill text.
- `.disabled(bool)`: Disables interaction and renders the header in muted text styling.
- `.id(egui::Id)`: Supplies a stable, unique interaction ID. Provide this for collapsibles inside dynamic lists whose item order can change; otherwise egui's auto ID follows layout order.
- `.show<R>(ui, content)`: Renders the clickable trigger header and conditionally executes the content closure `FnOnce(&mut Ui) -> R`, returning `(egui::Response, Option<R>)`.

## Behavior & Constraints

- Clicking the header toggles the bound `*open` state when enabled. A focused header also toggles on Space or Enter and exposes the `CollapsingHeader` role plus its expanded state to egui accessibility tooling.
- Focused enabled headers render an accent focus ring.
- The bound boolean is controlled state: callers own its lifetime and should not replace it during rendering.
- When `disabled(true)`, the header allocates `Sense::hover()`, ignores pointer and keyboard activation, and suppresses hover/focus styling.
- The leading chevron automatically flips from `Icon::ChevronRight` to `Icon::ChevronDown` as animation progress crosses the threshold.
- Content body rendering uses egui's shared `CollapsingState`: height is clipped and animated with the same openness value used by the chevron and header surface, while the body also fades inside standard content margins.

## Usage Example

```rust
use db_pro_ui::components::collapsible::Collapsible;
use db_pro_ui::DbProTheme;

let mut is_open = false;
let theme = DbProTheme::light();

let (response, result) = Collapsible::new(&mut is_open, theme)
    .title("Advanced Connection Settings")
    .icon(lucide_icons::Icon::Sliders)
    .badge("SSL Required")
    .show(ui, |ui| {
        ui.label("Custom nested settings controls");
    });
```

## Layers

- `mod.rs`: Public entry point and re-exports `Collapsible`.
- `ui.rs`: egui allocation, animation, painter operations, and content frame rendering.
- `handler.rs`: Pure click state transitions, chevron icon selection, semantic color resolution, and badge rectangle math.
- `config.rs`: Component-owned numeric layout dimensions, margins, and animation thresholds.
