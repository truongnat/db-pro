# Checklist — Legacy Component Sunset

- [x] Wave 0: delete 11 verified-dead legacy fns (`input_full_width`, `password_input`, `icon_button`, `spinner`, `progress_bar`, `skeleton`, `switch`, `segmented_control`, `kbd_chip`, `tag_chip`, `toast`) + `UiLevel`/`level_colors` orphans — `cargo check --workspace` PASS
- [ ] Wave 1: `Stroke::new(<literal>)` → `STROKE_*` (~23 sites) + focus-ring unification on `paint_focus_ring`
- [ ] Wave 2: `floating_surface` radius policy + overlay spacing consts
- [ ] Wave 3: config const aliases → `crate::tokens::*`; rename `toggle::ICON_TEXT_GAP`; fix `select/handler.rs` wrong-role margin
- [ ] Wave 4: `selection/ui.rs` literals → config/tokens
- [ ] Wave 5: migrate live legacy callers to core components; shrink `legacy.rs`
- [ ] Wave 6: gates — fmt, workspace check, clippy -D warnings, ui-architecture scan, tests; captures for radius/focus-ring changes (1280×800 light+dark)
