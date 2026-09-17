# Verification — sidebar-content-full-width

## Automated

```text
cargo check -p db-pro-ui --all-targets
→ PASS (2026-09-17)
```

## Runtime (pending)

Rebuild `db-pro-native`, open Explorer with a failed connection:

1. Tree hover wash extends to near the blue drag line (not mid-panel).
2. Long error hint truncates near the drag edge, not ~halfway across the sidebar.
3. `New query` / filter and tree share the same content width.
4. Dragging the separator still respects 220–380.

## Providers

n/a — layout-only UI change (PostgreSQL / SQLite unaffected).

---

## Follow-up: automated evidence (2026-09-17, `fix/sidebar-column-overflow`, off `main`)

```text
cargo test --workspace --offline
→ 1185 passed / 0 failed / 42 ignored          (db-pro-ui alone: 516 passed / 0 failed)
cargo clippy --workspace --all-targets --offline -- -D warnings
→ PASS
cargo build --release --locked -p db-pro-native
→ PASS
bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci
→ 15 pass / 1 warn / 0 fail
cargo fmt --all -- --check
→ red at baseline (repo-wide, pre-existing). Every line this branch changed is clean;
  the remaining hits in `sidebar_view.rs` (36/65/95/112/145/172/186/252/300/313/332/352/395)
  and `app_tests.rs:2195` are original untouched code — the added `set_clip_rect` line
  appears in the 65 hunk as context, not as a change.
```

The single clean-code warning is the ratcheted function-length debt
(`SearchInput::show` 98, `draw_sidebar` 66, `draw_sidebar_contents` 216) — all pre-existing
and all held at warning level by `--ratchet`. Zero blocking findings.

### Guards, each verified to fail on the reverted fix

A guard that passes on broken code is worthless, so each was checked by reverting the fix
it protects and confirming the failure message.

| Guard | Fix reverted | Observed failure |
| --- | --- | --- |
| `the_sidebar_paints_nothing_past_its_clip` | toolbar back to a hardcoded `actions_width = 28.0` | `[[180.0 103.0] - [214.0 127.0]] … clip=[[7.0 7.0] - [209.0 893.0]]` — the refresh button paints 5px past the clip |
| `the_sidebar_paints_nothing_past_its_clip` | `SIDEBAR_CLIP_BLEED` back to `0.0` | 3 shapes escape: the `New query` border `painted=[[7.0 47.0] - [209.0 77.0]]` vs `clip=[[8.0 8.0] - [208.0 892.0]]`, the separator rule, and the header action |
| `the_driver_badge_is_fully_visible_in_the_navigator_tree` | badge back to `badge.len() * 6.5 + 8.0` | `the driver badge pads its label by 8 instead of 4` |
| `the_filter_field_border_is_painted_inside_the_field` | `paint_field_chrome` back to stroking `rect` directly | `painted [[7.0 101.0] - [167.0 129.0]] against field [[8.0 102.0] - [166.0 128.0]]` |
| `the_tree_row_spans_its_layout_width_not_its_clip` | row width clamped to the clip | `left: 230.0, right: 240.0` — the row shrinks by the 10px simulated scrollbar |

Each guard also runs at `SIDEBAR_MIN_WIDTH`, `260.0` and `360.0`, because the fix has to
hold across the whole legal sidebar width rather than at one screenshot size.

### Measured after the fix (1440×900, `sidebar_width = 260`)

```text
SUB_PANES outer max_rect=[[8.0 8.0] - [248.0 892.0]] avail=240.0   (was [[8.0 102.0] - [254.0 136.5]], 282 wide)
#26 Rect fill=#FA_FA_FA  bounds=[[8.0 102.0] - [206.0 128.0]]       filter field takes the remainder
#24 Rect fill=#F7_F7_F7  bounds=[[214.0 102.0] - [248.0 126.0]]     refresh button, measured 34 wide, inside the column
#31 Rect stroke w=1.0 rounding=5 bounds=[[8.5 102.5] - [205.5 127.5]] clip=[[8.0 8.0] - [248.0 892.0]]
#35 Rect fill=#EE_EE_EE  bounds=[[205.0 147.0] - [244.0 163.0]] clip=[[8.0 142.0] - [248.0 168.0]]
#36 Text "SQLITE"        bounds=[[209.0 151.5] - [240.0 159.5]]
#37 Text "Native Test"   bounds=[[54.0 149.5] - [119.0 160.5]]
```

The badge pill now ends at `244`, inside the `248` clip, and its label is fully painted
(31px of text in a 39px pill) instead of surviving as a 1px sliver. The field's border sits
at `8.5–205.5`, strictly inside the clip, so no part of it is discarded.

## Runtime (pending)

Rebuild `db-pro-native` and confirm on the affected surface:

1. A connection row shows its driver badge in full, text readable, at 1280×800, 1440×900
   and 1920×1080.
2. The filter field's border is closed on all four sides, and so is the `New query`
   button's.
3. Grow the connection list past the fold: rows keep their full width and the badge does
   not shift left when the scrollbar appears.
4. Dragging the separator still clamps 220–380, and the tree still fills the column at
   `SIDEBAR_MIN_WIDTH`.
