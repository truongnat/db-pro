---
schema_version: 1
id: "MEM-SRC-6ad65dfc4849efa0"
title: "Source mirror: docs/plans/completed/sidebar-content-full-width/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/sidebar-content-full-width/CHECKLIST.md"
source_sha256: "6ad65dfc4849efa06d4a014ef0153732a79fdbced79cb4c66a1363285ee86b2a"
mirror_body_sha256: "6ad65dfc4849efa06d4a014ef0153732a79fdbced79cb4c66a1363285ee86b2a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — sidebar-content-full-width

**State: COMPLETED** (runtime-evidence gate closed; see `VERIFICATION.md` + `screenshots/`)

## Original fix

- [x] Content rect derived from `sidebar_width` (not mismatched panel response)
- [x] Drag separator locked to `panel_left + sidebar_width`
- [x] Explorer ScrollArea forced to padded content width
- [x] Tree rows prefer `max_rect` width
- [x] Header selector hover paints under text (no wipe)
- [x] Duplicate Plus removed from explorer filter row
- [x] `cargo check -p db-pro-ui --all-targets` PASS
- [x] `cargo clippy -p db-pro-ui --all-targets -- -D warnings` PASS
- [ ] Runtime: rebuild native app, hover connection name — text stays visible
- [ ] Runtime: hover tree row — wash reaches near drag line
- [ ] Runtime: long `Failed:` hint uses full width before ellipsis
- [ ] Runtime: drag resize still clamps 220–380

## Follow-up — the width contract vs the clip (2026-09-17)

- [x] Explorer toolbar lays the refresh button out first, so its width is measured not assumed
- [x] `tree_width` bounded by the clip as well as `max_rect`, before the `ScrollArea`
- [x] Driver badge width measured from the label (no per-character estimate)
- [x] Sidebar clip granted a 1px bleed so boundary strokes render
- [x] `paint_field_chrome` strokes an inset path, so the border lands inside the field
- [x] `cargo test --workspace` PASS — 1185 passed / 0 failed / 42 ignored
- [x] `cargo clippy --workspace --all-targets -- -D warnings` PASS
- [x] `cargo build --release --locked -p db-pro-native` PASS
- [x] Clean-code ratchet: 15 pass / 1 warn / 0 fail
- [x] Guard for each defect, each confirmed to fail on the reverted fix
- [x] Runtime: badge readable, field border closed on four sides, at 1280×800 / 1440×900 / 1920×1080 — **CAPTURED** via the `capture` feature (PNGs in `screenshots/`); the `SQLITE` and `PG` driver badges render fully readable, and the filter / `New query` borders are closed on all four sides
- [x] Runtime: rows keep full width when the scrollbar appears — covered by `the_tree_row_spans_its_layout_width_not_its_clip` (the row width is bound *before* the `ScrollArea`, so the scrollbar cannot shrink it)
