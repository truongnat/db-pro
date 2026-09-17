# Checklist — sidebar-content-full-width

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
