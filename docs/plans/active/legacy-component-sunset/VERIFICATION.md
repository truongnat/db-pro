# Verification — Legacy Component Sunset

## Wave 0 (dead-fn deletion)

Commands actually run:

- `rg -n -w '<fn>' -g '*.rs' crates` per candidate → 0 call/import-shape hits for
  all 11 (also zero for `UiLevel` outside legacy.rs).
- `cargo check -p db-pro-ui` → PASS (4.16s, no warnings).
- `cargo check --workspace` → PASS (8.64s): core, infrastructure, ui, app,
  runtime, native-app all compile — proof no external callers exist.

Not yet run: fmt --check workspace, clippy, tests, captures (scheduled Wave 6).


## Wave 1 (stroke tokens + focus-ring unification) — commit `2985e3fb`

- `cargo check -p db-pro-ui` PASS; zero `Stroke::new(<numeric>)` left in components
  (rg sweep verified).
- Runtime evidence: `paint_focus_ring` itself was already runtime-verified in the
  keyboard-correctness wave (files tabs / tools cards focus rings captured at
  1280×800–1920×1080). Wave 1 routes six `has_focus` sites to that recipe.
- **Gap (pending):** capture hooks cannot inject keyboard focus, so the per-site
  ring rects (Radio circle→fully-rounded rect ring, Slider thumb bounding rect)
  are source-verified only. Owner manual Tab pass or a focus-injection hook
  remains open — state for the ring sites is RUNTIME_VERIFY.

## Wave 2 (radius alignment + overlay spacing) — commit `e89d555f`

- `cargo check -p db-pro-ui` PASS.
- Captures 1280×800: Component Gallery `selection` (dark `/tmp/w2_selection_dark.png`,
  light `/tmp/w2_selection_light.png`) — checkboxes, switches, radios, dropdown
  select, slider all render cleanly in both themes; no layout breaks.
  `overlays` (dark `/tmp/w2_overlays_dark.png`, light `/tmp/w2_overlays_light.png`)
  — trigger rows + command-palette surface render cleanly both themes.
- **Gap:** open popover/dropdown/toast surfaces (radius 12→8, 10→8) are
  interaction-triggered; static gallery captures show triggers only. Visual check
  of an open popover remains pending.

## Wave 3 (config const aliases) — commit `1c2586f0`

- `cargo check -p db-pro-ui` PASS; `cargo test -p db-pro-ui` 715 PASS (subagent).
- Captures recorded (`/tmp/w3_disclosure_{dark,light}.png`, 1280×800) are
  **PROVISIONAL**: the capture binary was built after wave 2 and before the
  wave-3 commit; they show the pre-`RADIUS_FULL` pill geometry. Recapture on
  the post-wave-4 build before accepting them as evidence.
- Alias-only claim verified mechanically: every old→new pair extracted from
  `git show 1c2586f0` matches the token's value (1.0→STROKE_THIN, 4→SPACE_XS,
  8→SPACE_SM/RADIUS_MD/CARD, 11→FONT_SIZE_KBD/BADGE, 12→FONT_SIZE_CAPTION,
  13→FONT_SIZE_UI_LABEL, 15→FONT_SIZE_BODY, 16→SPACE_LG/ICON_DEFAULT,
  20→SPACE_XL, 24→SPACE_2XL, 32→SPACE_3XL). Sole intentional visual change:
  `BADGE_CORNER_RADIUS` 9→RADIUS_FULL (collapsible pill 8→FULL in ui.rs).