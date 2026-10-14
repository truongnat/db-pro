//! Feedback/Status component contract (alerts, badges, toasts, spinner, progress,
//! kbd chips, separators, skeletons).
//!
//! Status display states: default plus `info / warning / success / error(danger)`.
//! Loading is expressed by skeleton/spinner primitives rather than a state on a
//! specific widget. When a status element contains interactive children (alert
//! actions, toast buttons), those children resolve interaction states through
//! their own contracts — the `disabled → loading → active → focus → hover →
//! default` chain in [`crate::tokens::component::STATE_PRECEDENCE`] still applies
//! to them.
//!
//! Semantic roles consumed (via `DbProTheme`): `status.*` roles in full —
//! `solid` for icons and strong text, `fill`/`border` for alert frames
//! (`components/alert/handler.rs` reads them from `theme.semantic.status`), and
//! `subtle` via `DbProTheme::*_soft()` for badge/chip fills. `accent.*` drives
//! default badges; `background.elevated` drives secondary badges.
//!
//! Owned values: none moved in this slice — alert frame padding/radius, badge
//! metrics, toast layout, spinner/kbd geometry remain in
//! `components/alert/config.rs`, `components/badge/config.rs`,
//! `components/overlay/config.rs`, and `components/feedback/config.rs` as
//! component geometry. The shared status wash opacities
//! (`STATUS_FILL_OPACITY`, `STATUS_BORDER_OPACITY`) live with the status roles in
//! [`crate::tokens::semantic`], because they describe the role, not one component.
//! Shell chrome (`STATUS_BAR_HEIGHT`) is app-frame geometry in `crate::tokens`,
//! not part of this contract.
