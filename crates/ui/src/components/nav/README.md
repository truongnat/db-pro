# Navigation Components

Navigation controls and structural header typography components for page layouts, hierarchical trails, and dataset pagination.

## Public API

- `Pagination::new(page, page_count, theme)`: creates a pagination controller bound to a mutable page index.
  - `.enabled(enabled)`: enables or disables page switching buttons.
  - `.show(ui)`: renders the previous button, page count indicator, and next button.
- `page_icon_button(ui, icon, enabled, theme)`: low-level helper for pagination step buttons with directional accessible names, keyboard support, focus indicator, and tooltip.
- `BreadcrumbItem::new(label)`: creates an individual breadcrumb step.
  - `.current(current)`: marks the segment as active (non-clickable, primary text color).
- `Breadcrumb::new(items, theme)`: creates a breadcrumb trail.
  - `.show(ui)`: renders the crumb list with chevron delimiters and returns the clicked index (`Option<usize>`).
- `PageHeader::new(title, theme)`: creates a high-level page title.
  - `.description(description)`: optional subtitle supporting text.
  - `.show(ui)`: renders the vertical title layout.
- `SectionHeader::new(title, theme)`: creates a section or card divider title.
  - `.description(description)`: optional section supporting text.
  - `.show(ui)`: renders the subsection header.

## Behavior & Constraints

- `Pagination` mutates `*self.page` directly and ensures values remain within 1-based `[1, page_count]`.
- Disabled pagination controls display `"No more pages"` hover tooltips and dim appropriately (`text_disabled`).
- Breadcrumbs render interactive ancestor items with hover underlines and chevron separators (`Icon::ChevronRight`), wrapping when horizontal space runs out.
- The active/current breadcrumb item has hover sense disabled and renders in primary text color. Multiple items may be marked current; each such item is rendered non-interactively, while other items remain clickable.
- All typography and colors leverage `DbProTheme` design tokens.

## Usage Example

```rust
use db_pro_ui::components::nav::{Breadcrumb, BreadcrumbItem, PageHeader, Pagination, SectionHeader};

// Page Header
PageHeader::new("Database Connections", theme)
    .description("Configure and manage remote and local database endpoints.")
    .show(ui);

// Breadcrumbs
let crumbs = [
    BreadcrumbItem::new("Databases"),
    BreadcrumbItem::new("postgres"),
    BreadcrumbItem::new("users_table").current(true),
];
if let Some(clicked_idx) = Breadcrumb::new(&crumbs, theme).show(ui) {
    println!("Navigated to crumb index: {clicked_idx}");
}

// Pagination
let mut current_page = 1;
Pagination::new(&mut current_page, 10, theme).show(ui);
```

## Layers

- `mod.rs`: component entry point and public re-exports.
- `config.rs`: sizing constants (button sizes, font sizes, gap dimensions, stroke widths).
- `handler.rs`: pure navigation logic (page clamping, boundary predicates, color selection, formatting).
- `ui.rs`: egui widget rendering and interaction handling.
