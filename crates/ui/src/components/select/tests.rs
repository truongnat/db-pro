use super::layout::dropdown_should_open_above;

#[test]
fn dropdown_flips_above_when_there_is_no_room_below() {
    assert!(dropdown_should_open_above(40.0, 280.0, 260.0));
    assert!(!dropdown_should_open_above(300.0, 40.0, 260.0));
    assert!(!dropdown_should_open_above(200.0, 200.0, 180.0));
}
