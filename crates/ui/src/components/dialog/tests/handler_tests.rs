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

#[test]
fn a_pinned_body_is_clamped_to_the_viewport_cap() {
    assert_eq!(pinned_body_height(Some(560.0), 812.0), Some(560.0));
    assert_eq!(pinned_body_height(Some(900.0), 812.0), Some(812.0));
    assert_eq!(pinned_body_height(None, 812.0), None);
}

#[test]
fn a_pinned_card_centers_on_its_box_only_until_it_has_been_measured() {
    // Frame one has no measurement, so the pinned box is the center; from frame
    // two the real card wins. The box cannot grow, so this never slides again.
    assert_eq!(dialog_centering_height(Some(560.0), None, 220.0), Some(780.0));
    assert_eq!(dialog_centering_height(Some(560.0), Some(716.0), 220.0), Some(716.0));
    assert_eq!(dialog_centering_height(None, Some(434.0), 220.0), Some(434.0));
    assert_eq!(dialog_centering_height(None, None, 220.0), None);
}
