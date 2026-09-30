//! Dialog/Overlay component contract (modal dialogs, sheets, popovers, toasts).
//!
//! States supported: `default` for the open frame plus the overlay layer's
//! `open → closing` motion. Containers do not take hover/focus/active states
//! themselves — child controls resolve their own states through their contracts.
//!
//! Semantic roles consumed (via `DbProTheme`): `background.overlay` for the scrim
//! (`DbProTheme::overlay`, painted in `components/dialog/ui.rs` and
//! `components/dialog/sheet.rs`), `background.floating` for the frame fill,
//! `border.subtle` for the frame stroke, `foreground.primary` for titles, and
//! `accent.solid` for primary actions inside dialogs.
//!
//! Elevation comes from the primitive scale through `DbProTheme::floating_shadow`
//! (`SHADOW_OFFSET_Y`, `SHADOW_BLUR`, `WINDOW_SHADOW_ALPHA`); overlay motion
//! duration is `DURATION_OVERLAY_SECS`.
//!
//! Component tokens (stay in `components/dialog/config.rs` and
//! `components/overlay/config.rs`, labelled there as component geometry):
//! - `DIALOG_RADIUS` (16pt product dialog frame) is intentionally different from
//!   `tokens::RADIUS_DIALOG` (6pt), which only maps egui's own window rounding in
//!   the theme adapter — the two names describe different surfaces;
//! - sheet/dropdown/popover/context-menu radii, translate offsets, content
//!   padding, min/max sizes, and viewport clamping (`DIALOG_CHROME_HEIGHT`).
