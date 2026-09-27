# Badge

Dense status and metadata badge for native egui surfaces. Badges support semantic variants, optional leading status dots, optional Lucide icons, and compact density.

## Usage

```rust
Badge::new("Active", theme)
    .variant(BadgeVariant::Success)
    .dot(true)
    .show(ui);
```

Use `Badge::compact(true)` for very tight metadata rows. If both `dot(true)` and `icon(...)` are provided, the dot is rendered and the icon is ignored, matching the existing leading-status priority. Badges are non-interactive; their text is exposed to egui accessibility output as a label. Width is intrinsic to the full text, so callers should use concise status wording.

## Public API

- `Badge`: builder-style egui widget.
- `BadgeVariant`: semantic color variants (`Default`, `Secondary`, `Outline`, `Destructive`, `Success`, `Warning`, `Info`).
- `BadgePalette`: variant-to-theme color mapping for tests and advanced internal callers.
- `BadgeMetrics`: density and sizing calculations for tests and advanced internal callers.

## Layers

- `mod.rs`: public entry and API exports.
- `ui.rs`: egui allocation and painting.
- `handler.rs`: pure palette, density, leading-content and text-position calculations.
- `config.rs`: Badge-owned dimensions; shared radius/stroke values remain in `crates/ui/src/tokens.rs`.
