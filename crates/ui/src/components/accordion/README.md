# Accordion

Native collapsible accordion component supporting single-item or multi-item disclosure expansion.

## Public API

- `Accordion::new(theme)`: Creates an accordion renderer configured with the active `DbProTheme`.
- `AccordionItem::new(id, title)`: Builds an item definition. Optional builder methods: `.icon(lucide_icons::Icon)`, `.badge(str)`, `.disabled(bool)`.
- `AccordionType`: Enum providing `Single { collapsible: bool }` and `Multiple` variants for API compatibility.
- `show_single(ui, item, selected_id, collapsible, content)`: Renders a single-item disclosure item.
- `show_multi(ui, item, open_set, content)`: Renders an item participating in a multi-item open set (`BTreeSet<String>`).

## Behavior & Constraints

- Item IDs must be unique within the accordion scope so focus, animation, and open state do not collide.
- Headers expose `WidgetType::CollapsingHeader` accessibility state with the current enabled/expanded values.
- Enabled focused headers support Space/Enter activation and render the shared focus ring.
- Disabled headers ignore click and keyboard activation events and retain disabled text styling.
- In multi-expansion mode, toggling disabled items preserves their existing open/closed state.
- Expanding and collapsing uses the shared disclosure openness animation: body height clips smoothly, body opacity follows the same progress, and the chevron crossfades between closed/open glyphs.
- Badge and chevron space is reserved before title painting; long titles are clipped within remaining width without UTF-8 truncation.

## Usage Example

```rust
use std::collections::BTreeSet;
use db_pro_ui::components::accordion::{Accordion, AccordionItem};

// Single expansion mode
let mut selected_id = Some("advanced".to_string());
let item = AccordionItem::new("advanced", "Advanced Settings")
    .icon(lucide_icons::Icon::Sliders)
    .badge("Pro");

Accordion::new(theme).show_single(ui, item, &mut selected_id, true, |ui| {
    ui.label("Advanced configuration fields here");
});

// Multi expansion mode
let mut open_set = BTreeSet::from(["panel-1".to_string()]);
let multi_item = AccordionItem::new("panel-1", "Database Pool");

Accordion::new(theme).show_multi(ui, multi_item, &mut open_set, |ui| {
    ui.label("Pool statistics and connection settings");
});
```
