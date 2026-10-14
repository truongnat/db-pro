//! Tests for `../component.rs` (moved out of the source file).
use super::*;

/// The chain required by the core token contract, pinned so a reorder has to
/// happen deliberately rather than by editing a doc comment.
#[test]
fn state_precedence_is_declared_strongest_first() {
    assert_eq!(
        STATE_PRECEDENCE,
        ["disabled", "loading", "active", "focus", "hover", "default"]
    );
}
