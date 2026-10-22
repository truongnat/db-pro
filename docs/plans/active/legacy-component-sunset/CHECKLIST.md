# Checklist — Legacy Component Sunset

- [x] Wave 0: delete 11 verified-dead legacy fns + `UiLevel`/`level_colors` — workspace check PASS
- [x] Wave 1: `Stroke::new(<literal>)` → `STROKE_*` (23 sites) + focus-ring unification — committed `2985e3fb`
- [x] Wave 2: floating-surface radii → `RADIUS_*` roles + overlay spacing consts — `e89d555f`
- [x] Wave 3: config const aliases → tokens; `toggle::ICON_TEXT_GAP` rename; `INPUT_AUX_GAP` deleted — `1c2586f0`
- [x] Wave 4: `selection/ui.rs` literals → config/tokens — `11ff76d7`
- [x] Wave 5: 77 button sites + 39 helper sites migrated; `legacy.rs` deleted entirely — `a67107a6`, `fef890eb`, `da4c7bfa`
- [x] Wave 6 gates: fmt ✓ clippy 0 ✓ tests 2257/0 ✓ ui-architecture PASS ✓ release build PASS ✓ cc-ratchet 0 fail ✓
- [ ] Pending (RUNTIME_VERIFY): open-popover radius visual check (Quick Open capture hangs — pre-existing infra), `has_focus` ring manual Tab pass (no focus-injection hook), 1440×900/1920×1080 captures
