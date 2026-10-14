//! Tests for `../hover.rs` (moved out of the source file).
use super::*;

fn token(range: (usize, usize)) -> HoveredSqlToken {
    HoveredSqlToken {
        range,
        anchor_rect: Rect::NOTHING,
    }
}

#[test]
fn hover_confirmed_after_delay() {
    let mut state = HoverState::new();
    let t0 = Instant::now();
    let tok = token((0, 6));

    // Frame 1: pointer enters token.
    state.update(Some(tok), t0);
    assert!(state.confirmed_token().is_none());

    // Frame 2: still pending, not enough time.
    state.update(Some(tok), t0 + Duration::from_millis(300));
    assert!(state.confirmed_token().is_none());

    // Frame 3: delay elapsed → confirm.
    state.update(Some(tok), t0 + Duration::from_millis(501));
    assert_eq!(state.confirmed_token().unwrap().range, (0, 6));
}

#[test]
fn hover_cleared_on_pointer_leave() {
    let mut state = HoverState::new();
    let t0 = Instant::now();
    let tok = token((0, 6));

    state.update(Some(tok), t0);
    state.update(Some(tok), t0 + Duration::from_millis(600));
    assert!(state.confirmed_token().is_some());

    state.update(None, t0 + Duration::from_millis(700));
    assert!(state.confirmed_token().is_none());
}

#[test]
fn hover_resets_on_different_token() {
    let mut state = HoverState::new();
    let t0 = Instant::now();

    state.update(Some(token((0, 6))), t0);
    state.update(Some(token((0, 6))), t0 + Duration::from_millis(600));
    assert!(state.confirmed_token().is_some());

    // Move to a different token — confirmed should clear and restart.
    state.update(Some(token((7, 14))), t0 + Duration::from_millis(700));
    assert!(state.confirmed_token().is_none());
}

#[test]
fn pending_repaint_returns_remaining_time() {
    let mut state = HoverState::new();
    let t0 = Instant::now();
    state.update(Some(token((0, 6))), t0);
    let after = state.pending_repaint_after(t0 + Duration::from_millis(200));
    assert!(after.is_some());
    assert!(after.unwrap() > Duration::from_millis(200));
}
