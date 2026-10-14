//! Tests for `../handler.rs` (moved out of the source file).
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
