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

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
