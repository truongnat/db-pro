use super::config::INPUT_MIN_WIDTH;
use super::layout::resolve_field_width;

#[test]
fn fills_available_width_when_unrequested() {
    assert_eq!(resolve_field_width(None, 320.0), 320.0);
}

#[test]
fn requested_width_never_exceeds_the_container() {
    assert_eq!(resolve_field_width(Some(420.0), 260.0), 260.0);
}

#[test]
fn narrow_containers_keep_a_usable_floor() {
    assert_eq!(resolve_field_width(None, 40.0), INPUT_MIN_WIDTH);
    assert_eq!(resolve_field_width(Some(200.0), 40.0), INPUT_MIN_WIDTH);
}

#[test]
fn requested_width_below_floor_is_raised() {
    assert_eq!(resolve_field_width(Some(60.0), 320.0), INPUT_MIN_WIDTH);
}
