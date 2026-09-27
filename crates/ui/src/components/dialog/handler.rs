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
mod tests {
    use super::*;

    #[test]
    fn only_topmost_modal_responds_to_escape_or_backdrop() {
        assert_eq!(modal_dismissal(false, true, true, None, None), None);
        assert_eq!(
            modal_dismissal(true, true, false, None, None),
            Some(ModalDismissal::Escape)
        );
    }

    #[test]
    fn backdrop_closes_only_when_pointer_is_outside_the_card() {
        let card = Rect::from_min_max(Pos2::new(20.0, 30.0), Pos2::new(120.0, 130.0));

        assert_eq!(
            modal_dismissal(true, false, true, Some(Pos2::new(50.0, 60.0)), Some(card)),
            None
        );
        assert_eq!(
            modal_dismissal(true, false, true, Some(Pos2::new(10.0, 10.0)), Some(card)),
            Some(ModalDismissal::Backdrop)
        );
        assert_eq!(
            modal_dismissal(true, false, true, None, Some(card)),
            Some(ModalDismissal::Backdrop)
        );
    }

    #[test]
    fn escape_has_precedence_over_a_backdrop_click() {
        assert_eq!(
            modal_dismissal(true, true, true, Some(Pos2::ZERO), None),
            Some(ModalDismissal::Escape)
        );
    }
}
