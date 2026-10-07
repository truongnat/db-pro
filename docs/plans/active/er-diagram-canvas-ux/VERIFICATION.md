# Verification — ER Diagram Canvas UX

Status: evidence collected. All gates executed against the current working tree
(uncommitted on `main`, per owner override).

## Gates

| Gate | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all -- --check` | PASS (exit 0) |
| check | `cargo check -p db-pro-ui` + workspace compile via clippy/test | PASS |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| test | `cargo test --workspace` | PASS — 1685 tests, 0 failed (41 ignored = live PG/MySQL/SQLServer/SSH fixtures needing external services) |
| native release | `cargo build --release --locked -p db-pro-native` | PASS (perf-scan built it: 32.5MB < 50MB) |
| perf audit | `bash .skills/perf-audit/scripts/perf-scan.sh` | PASS (4 executed: release build, binary size, check, clippy; ER/bench/db sections skipped — need window server/live DB) |
| clean code | `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | PASS — 0 fail, 5 warnings (pre-existing ratchet size debt: `draw_diagram` 101 ln, `diagram_view.rs` 872 ln, `workspace_actions.rs` 1176 ln, `translate_cmd.rs`, `worker.rs` clones) |

## Runtime evidence

Capture binary must be built with `--features capture` (feature is off by
default; `DB_PRO_CAPTURE_*` env vars are inert without it — a plain
`cargo build` silently produces a binary that opens the app and never writes a
PNG). Screenshots land under `screenshots/`; the live "Sample E-Commerce"
SQLite connection (73 tables / 94 relationships) auto-connected and replaced
the seeded fixture, so the shots document the real schema.

| Viewport | Theme | File | Notes |
|---|---|---|---|
| 1280×800 | dark | `screenshots/er-1280x800-dark.png` | Auto-fit → 21% zoom, Compact pills, minimap, compact toolbar + zoom strip |
| 1280×800 | dark | `screenshots/er-1280x800-zoom1.png` | `DB_PRO_CAPTURE_DIAGRAM_ZOOM=1.3` → Detailed LOD: two-line headers, key/link icon markers, `FK → table.column` targets, crow's-foot + bar endpoints, edge label pills |
| 1440×900* | dark | `screenshots/er-1440x900.png` | Auto-fit → 22%; *macOS capped inner height at 838 logical px (display minus chrome) |
| 1920×1080* | dark | `screenshots/er-1920x1080.png` | Auto-fit → 22%; *height likewise capped at 838 |
| 1280×800 | light | `screenshots/er-1280x800-light.png` | Restored persisted viewport (60% = proof layout persistence survives restarts); Standard LOD renders full column rows in light theme |

Persistence is covered by `diagram_position_overrides_apply_by_table_key`,
`diagram_move_node_updates_rect_and_incident_edge_bbox`, and the storage
round-trip in `app_storage`; the 60% light capture is incidental runtime proof
that a viewport saved in an earlier run was restored. Manual drag/pan gestures
are exercised by `dragging_canvas_updates_pan` and the node-drag canvas tests;
pointer-level drag can only be fully confirmed by the owner in the live app.

## Bugs found & fixed during verification

1. `ErLod` banding: Compact covered zoom < 0.75, so at ~0.6 nodes painted as
   large empty cards (light capture exposed it). Threshold lowered to 0.45;
   tests updated (`lod_monotonic_transitions`, `zoom_changes_lod_not_graph`,
   `diagram_lod_transitions_and_rules`, scene test at zoom 0.5→0.3).
2. Capture-harness stalls (pre-existing, flaky): a `visible:false` window can
   receive no `drawRect` on macOS, so the first frame may never paint; eframe's
   persisted maximized state and `DbProApp`'s startup `Maximized(true)`
   (3 frames) both fought `pin_viewport`. Fixed by `with_visible(true)` +
   `persist_window:false` for capture runs and skipping the startup maximize
   when `DB_PRO_WINDOW_SIZE` is set.
3. Root cause of earlier "hang" runs: capture binary was rebuilt without
   `--features capture`; the env vars are inert and the app simply stays open.
   Documented above to prevent repeat.
4. Pan drag appeared dead (owner smoke test): `Response::drag_delta()` in egui
   0.29 is the per-frame pointer delta, not the distance since press, so
   `pan = origin + drag_delta()` snapped back every frame. Fixed by recording
   `(pan_at_grab, press_origin)` at `drag_started` and recomputing
   `pan = grab_pan + (pointer - press)`; the canvas drag test now asserts the
   full (74, 53) travel across two move frames — the old code would have left
   (10, 5) and failed.
5. "+N more columns" painted over the last real column row — it now takes the
   last row slot (renders `max_cols - 1` columns + the overflow line).
6. Owner review round 2: single click opened tables immediately; edges too
   faint and uninformative; cards thin on info. Now: double-click opens,
   single click pins the node's relationship highlight (click empty canvas to
   unpin), edges carry crow's-foot ("many" at FK side) + bar ("one" at PK
   side) endpoints, edge labels show `fk_col → pk_col` at Detailed always and
   at Standard while a node is highlighted, and column rows gain key/link
   icon markers plus `FK → table.column` targets at Detailed (and
   `PK·FK → target` for composite key columns); header subtitle gains
   row-count when introspection provides it.

## Provider accounting

- SQLite: live 73-table schema rendered + captured (dark + light, 3 viewports).
- PostgreSQL: diagram consumes the provider-neutral `UiSchemaSummary`; no
  provider-specific code path changed. PG live rendering not separately
  captured — no PG instance available on this host.

## Remaining

- Pointer-level drag persistence (drag → save → relaunch → same position) is
  covered by unit tests + snapshot restore evidence, but the physical
  mouse-drag gesture should be smoke-tested by the owner in the live app.
- Plan stays under `active/` pending owner review; move to `completed/` only
  after handoff acceptance.
