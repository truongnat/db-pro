use egui::Rect;
use std::time::{Duration, Instant};

/// Raw hover detection: which token the pointer is currently over.
/// Emitted every frame by the renderer when the pointer is inside the editor text area.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoveredSqlToken {
    pub range: (usize, usize),
    pub anchor_rect: Rect,
}

/// Delay before the hover popup is shown, matching VS Code / Zed behaviour.
const HOVER_DELAY: Duration = Duration::from_millis(500);

/// Per-document hover FSM.
///
/// ```text
/// Idle
///   ──pointer enters token──▶ Pending(token, t₀)
///                               │ t > t₀+HOVER_DELAY │
///                               ▼
///                             Confirmed(token)
///                               │ pointer leaves │
///                               ▼
///                             Idle
/// ```
///
/// The state is stored in `QueryDocument` (not in the renderer) so it survives
/// across frames without requiring mutable renderer state.
#[derive(Debug, Default, Clone)]
pub struct HoverState {
    /// Token the pointer is currently sitting over (raw, every frame).
    pending: Option<(HoveredSqlToken, Instant)>,
    /// Token whose popup is currently visible (delayed confirmation).
    confirmed: Option<HoveredSqlToken>,
}

impl HoverState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Called each frame with the raw `hovered_token` from `SqlEditorResponse`.
    /// Returns `true` when the state changed and a repaint is needed.
    pub fn update(&mut self, raw: Option<HoveredSqlToken>, now: Instant) -> bool {
        match (raw, self.pending) {
            // Pointer has moved to a new token — reset pending timer.
            (Some(token), Some((prev, _))) if token.range != prev.range => {
                self.pending = Some((token, now));
                let changed = self.confirmed.is_some();
                self.confirmed = None;
                changed
            }
            // Pointer entered a token from idle state.
            (Some(token), None) => {
                self.pending = Some((token, now));
                false
            }
            // Pointer left the token area — clear everything.
            (None, _) => {
                let changed = self.pending.is_some() || self.confirmed.is_some();
                self.pending = None;
                self.confirmed = None;
                changed
            }
            // Pointer is still on the same token — check if delay elapsed.
            (Some(_), Some((token, started_at))) => {
                if self.confirmed.is_none() && now.duration_since(started_at) >= HOVER_DELAY {
                    self.confirmed = Some(token);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// The token whose hover popup should currently be shown, if any.
    pub fn confirmed_token(&self) -> Option<HoveredSqlToken> {
        self.confirmed
    }

    /// Whether we are waiting for the delay on a pending token (need a repaint
    /// scheduled at the expected confirmation time).
    pub fn pending_repaint_after(&self, now: Instant) -> Option<Duration> {
        let (_, started_at) = self.pending?;
        if self.confirmed.is_some() {
            return None;
        }
        let elapsed = now.duration_since(started_at);
        if elapsed >= HOVER_DELAY {
            None
        } else {
            Some(HOVER_DELAY - elapsed)
        }
    }

    /// Clear all hover state (e.g. when the document switches or a completion opens).
    pub fn clear(&mut self) {
        self.pending = None;
        self.confirmed = None;
    }
}

#[cfg(test)]
mod tests {
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
}
