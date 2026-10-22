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
- Recaptured on the post-wave-4 binary (built at `11ff76d7`):
  `disclosure` + `selection` sections, dark + light, 1280×800
  (`/tmp/w34_*_{dark,light}.png`). **MD5 identical to the pre-wave-3
  captures** — `RADIUS_FULL` vs 9/8 clamps to the same pixels on the
  ~18px badge height, so the wave-3 pill change is a render no-op and
  waves 3–4 introduce **zero visual regression** on these surfaces.
- Alias-only claim verified mechanically: every old→new pair extracted from
  `git show 1c2586f0` matches the token's value (1.0→STROKE_THIN, 4→SPACE_XS,
  8→SPACE_SM/RADIUS_MD/CARD, 11→FONT_SIZE_KBD/BADGE, 12→FONT_SIZE_CAPTION,
  13→FONT_SIZE_UI_LABEL, 15→FONT_SIZE_BODY, 16→SPACE_LG/ICON_DEFAULT,
  20→SPACE_XL, 24→SPACE_2XL, 32→SPACE_3XL). Sole intentional visual change:
  `BADGE_CORNER_RADIUS` 9→RADIUS_FULL (collapsible pill 8→FULL in ui.rs).

## Wave 4 (selection literals) — commit `11ff76d7`

- `cargo check`/`cargo test -p db-pro-ui` 715 PASS (subagent); `selection`
  captures above (identical renders) — semantic literals intentionally kept:
  `CONTROL_DESC_SIZE` 11.5 proportional (aliasing to `FONT_SIZE_MONO_SM`
  would change font family), `CONTROL_DESC_GAP` 3.0.
## Wave 5a (legacy buttons → components::Button) — uncommitted→commit

- `cargo check --workspace` PASS; `cargo test -p db-pro-app` 676/676 PASS.
- 77 call sites migrated; 8 legacy fns deleted. Sizes intentionally moved to
  `ButtonSize::Default` tokens (legacy 26–30px min_size dropped — intended).
- Captures 1280×800 dark+light: tools hub (`/tmp/w5a_tools_{dark,light}.png`)
  and schema compare (`/tmp/w5a_compare_{dark,light}.png`) — migrated
  ghost/secondary buttons ("Take snapshot", "Diff vs snapshot", "Snapshot",
  "Diff now") render correctly in both themes.

## Wave 5a/5b (legacy callers → core components) — `a67107a6`, pending commit

- `cargo check --workspace` PASS; `cargo test -p db-pro-app` 676/676 both waves.
- 5a: 77 button callsites migrated, captures tools+compare dark/light OK.
- 5b: `plural_count`/`empty_state`/`badge` deleted (3+8+28 sites →
  format_count_with_suffix / chrome::EmptyState / components::Badge);
  `section_label`, `status_dot`, `icon_text`, `tinted` kept — retokenized
  (no core equivalent for pulse ring / muted section label / layout job).
- Captures 1280×800 dark+light: `table_profile` pane with reassert flag
  (`/tmp/w5b_profile_{dark,light}.png`) — migrated badges ("Page sample"
  Secondary, pattern tints Unique/Repeated/Varied) legible in both themes.
  NOTE: `DB_PRO_CAPTURE_REASSERT_SURFACE=1` required — schema load steals
  the active tab otherwise; earlier captures without it showed Welcome.

## Wave 5c (legacy.rs deleted entirely) — pending commit

- `legacy.rs` (414 lines post-5b) removed; `pub use legacy::*` dropped from
  `components/mod.rs`. Relocations: `card_frame` callers → `Card::new(...).frame()`
  (40 sites); `input` → `components::input::Input` (30 sites);
  `selectable_card_frame` → `card::selectable_frame`; `sidebar_item` +
  `section_label` → `nav`; `tab_button` → `tabs/ui/tab_button.rs`;
  `status_dot`+`tinted` → `feedback`; `icon_text`/`icon_layout` → `common::text`;
  all 12 `*_frame` + `ShellSurface` → `common::frames` (retokenized).
- `cargo check --workspace` PASS, `cargo test -p db-pro-ui` 715 PASS,
  `cargo test -p db-pro-app` 676 PASS (one timing flake passed on rerun).
- Captures 1280×800: settings dark (`/tmp/w5c_settings_dark.png`) — section
  labels + card frame render correctly; gallery `form` dark+light
  (`/tmp/w5c_form_{dark,light}.png`) — migrated `Input` fields + cards clean.

## Final gates — HEAD `da4c7bfa` + doc updates

- `cargo +nightly fmt --all -- --check` → PASS
- `cargo clippy --workspace --all-targets -- -D warnings` → 0 diagnostics
- `cargo test --workspace` → **2257 passed / 0 failed**
- `scripts/check-ui-architecture.sh` → PASS
- `cargo build --release --locked -p db-pro-native` → PASS
- clean-code diff-ratchet → 0 fail (6 warnings, pre-existing)
- **Limitations (state: RUNTIME_VERIFY):**
  - `DB_PRO_CAPTURE_QUICK_OPEN` and `GALLERY_SECTION=alerts` capture runs
    hang (no screenshot reply within 180s) — pre-existing capture-infra gap,
    not from this work; open-popover radius (12→8) verified at source level
    only.
  - `has_focus` ring sites: unified recipe `paint_focus_ring` is
    runtime-verified from the keyboard wave, but per-site rings need a
    focus-injection hook or manual Tab pass.
  - Captures recorded at 1280×800 only; 1440×900/1920×1080 pending (host
    caps effective height at 838 per prior audits).

## Focus-ring per-site check — 2026-10-22 (live app, manual Tab pass)

Drove the running `/tmp/DB Pro.app` binary via `orca computer` + `cliclick kp:tab`
(cliclick sends real key events; orca `press-key` requires OS focus which the
detached binary launch never gets — known trap).

- Tab #1 (after canvas click): accent ring around top-right topbar icon
  → `evidence/focus_ring_topbar.png`
- Tab #2: accent ring around "AI Copilot Assistant" Welcome action card
  → `evidence/focus_ring_card.png`
- Control→site mapping (both are migrated `paint_focus_ring` callers,
  verified in source):
  - topbar icon → `nav::page_icon_button` → `nav/ui.rs:81`
  - Welcome action card → `draw_card_item` →
    `app/src/shell/welcome/welcome_surface_view.rs:204-205`
- Both rings render accent `border_focus` at `STROKE_THICK` expanded —
  the unified recipe.
- Coverage: 2 migrated sites exercised live on the current binary; the
  remaining sites (tree rows, workspace items, dropdown items, agent
  chips/disclosure headers, radio, slider) share the identical helper
  call and are source-verified.

## Focus-visible semantics fix — live verified (2026-10-22)

Problem: many components `request_focus` on pointer click (tabs, calendar,
radio, welcome cards…), so `has_focus()` was true after a mouse click and the
ring painted for pointer users too — `has_focus` ≠ `:focus-visible`.

Fix: `interact::update_focus_visible(ctx)` runs once per frame in
`DbProApp::logic` (app_lifecycle.rs) and tracks last-input-kind in
`ctx.data` (`insert_temp`, per-frame). `focus_visible` defaults false,
turns true on focus keys (Tab/arrows/Enter/Space), false on pointer
press/release. `paint_focus_ring` early-returns when not keyboard-driven —
single gate covers all ~25 call sites.

Live verification (orca + cliclick on the running app):
- Click workspace tab / employees row → tab switched, NO ring.
- Tab in Settings → accent ring on "Light" radio.
- Click "Dark" afterwards → ring gone.
- Deadlock note fixed: `ctx.input` closure captures flags only; `ctx.data`
  read/write happen outside it (advisory caught `input`+`data_mut` sharing
  `ctx.write`).
