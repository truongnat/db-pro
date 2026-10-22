use egui::{Pos2, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModalDismissal {
    Escape,
    Backdrop,
}

/// Resolves modal dismissal signals without changing the caller-owned open state.
pub(super) fn modal_dismissal(
    is_topmost: bool,
    escape_pressed: bool,
    backdrop_clicked: bool,
    pointer_position: Option<Pos2>,
    card_rect: Option<Rect>,
) -> Option<ModalDismissal> {
    if !is_topmost {
        return None;
    }
    if escape_pressed {
        return Some(ModalDismissal::Escape);
    }
    if !backdrop_clicked {
        return None;
    }

    let click_is_inside_card = pointer_position
        .zip(card_rect)
        .is_some_and(|(position, rect)| rect.contains(position));
    (!click_is_inside_card).then_some(ModalDismissal::Backdrop)
}

/// Body height when the caller pinned the dialog box, clamped to what the
/// viewport can show. `None` keeps the content-driven (shrink-to-fit) body.
pub(super) fn pinned_body_height(declared: Option<f32>, body_cap: f32) -> Option<f32> {
    declared.map(|height| height.min(body_cap))
}

/// Height the card centers on.
///
/// Once the card has been measured, that measurement is the only honest center.
/// Before the first measurement a pinned dialog centers on its own box (body +
/// reserved chrome) and a content-sized one on the viewport estimate. The pin
/// matters because a pinned card cannot grow, so the measured center is stable
/// from frame two instead of sliding upward as asynchronous rows arrive.
pub(super) fn dialog_centering_height(
    pinned_body: Option<f32>,
    prev_height: Option<f32>,
    chrome_height: f32,
) -> Option<f32> {
    prev_height.or_else(|| pinned_body.map(|body| body + chrome_height))
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
