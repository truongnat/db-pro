# Navigation API

Import from `db_pro_ui::components::nav`.

```rust
PageHeader::new("Connections", theme).description("Manage database endpoints").show(ui);
let items = [BreadcrumbItem::new("Databases"), BreadcrumbItem::new("Clients").current(true)];
if let Some(index) = Breadcrumb::new(&items, theme).show(ui) { open_crumb(index); }
Pagination::new(&mut page, page_count, theme).enabled(is_ready).show(ui);
```

- `Pagination::new(&mut page, page_count, theme)` binds a 1-based mutable page index; `.enabled(bool)` controls whether movement is allowed; `.show(ui) -> Response` paints controls and mutates the page on click. `page_count` is normalized to at least one and the current page is clamped during rendering.
- `page_icon_button(ui, icon, enabled, theme) -> Response` is the low-level accessible previous/next button helper.
- `BreadcrumbItem::new(label).current(bool)` describes one crumb. `Breadcrumb::new(items, theme).show(ui) -> Option<usize>` returns the index of a clicked non-current item for caller dispatch.
- `PageHeader::new(title, theme).description(text).show(ui)` and `SectionHeader::new(title, theme).description(text).show(ui)` render static hierarchy labels.

Current breadcrumb items are not interactive; all other items can return an index. Pagination's page reference remains caller-owned. Labels, persistence, and the action performed after navigation remain caller responsibilities.
