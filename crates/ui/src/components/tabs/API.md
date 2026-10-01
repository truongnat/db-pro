# Tabs API

```rust
let labels = ["Overview", "Schema", "Data"];
SegmentedTabs::new(&mut selected, &labels, theme).show(ui);
UnderlineTabs::new(&mut selected, &labels, theme).show(ui);
```

- `SegmentedTabs::new(&mut usize, &[&str], theme)` builds a pill track. `.focusable(bool)` controls focus/keyboard participation, defaulting to `true`. `.show(ui)` returns `()` and writes the selected index.
- `UnderlineTabs::new(&mut usize, &[&str], theme)` builds a baseline/underline track. `.show(ui)` returns `()` and writes the selected index.

Both controls respond to click and focused arrow keys (left/up moves backward, right/down forward, with wraparound). Enter/Space selects the focused item. The selected index is state owned by the caller; keep it below the label count. An empty slice displays no items. Invalid selected indices do not panic and display without an active tab until a valid index is selected. There is no per-tab disabled state or tab-panel content management.

Indicator movement respects `DbProTheme::reduce_motion`. Long labels retain their natural measured width; use horizontal scrolling when the track cannot fit. `SegmentedTabs` exposes its `.focusable(false)` escape hatch for decorative/non-interactive tracks.

## Layers

The public types remain re-exported from `components::tabs` and `components`. `ui/` owns egui rendering, `handler.rs` owns input-to-selection decisions, `layout.rs` collects geometry, `track.rs` handles indicator position, and `config.rs` contains tab-specific measurements.
