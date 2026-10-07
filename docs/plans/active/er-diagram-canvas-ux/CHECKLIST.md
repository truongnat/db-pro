# Checklist — ER Diagram Canvas UX

- [x] `ErGraph`: `move_node`, edge bbox recompute, `apply_position_overrides`
- [x] `ErLayoutSnapshot` serde type + `dbpro.native.er-layouts-v1` key
- [x] `DiagramState`: drag state, `auto_fit_done`, `saved_layouts`,
      `capture_zoom_override`
- [x] Canvas: no `ScrollArea`; drag-pan; wheel zoom at pointer; node drag;
      minimap; auto-fit + search fit
- [x] Toolbar: compact icon row per owner direction
- [x] Node header two-line + measured truncation; edge lanes + dimming
- [x] Storage restore/persist wiring (`app_storage`, `app_state`, `app_lifecycle`)
- [x] Diagram capture fixture seeds schema (`open_diagram_workspace_for_capture`)
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace` (via workspace test/clippy compiles)
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `cargo build --release --locked -p db-pro-native`
- [x] Runtime captures 1280×800 / 1440×900* / 1920×1080* (dark) + 1280×800 light
      (*height capped by the host display, see VERIFICATION)
- [x] LOD banding fix after light-capture review (Compact < 0.45)
- [x] Capture-harness fixes: `with_visible` for pinned runs, `persist_window`
      off in capture mode, startup maximize skipped under `DB_PRO_WINDOW_SIZE`
