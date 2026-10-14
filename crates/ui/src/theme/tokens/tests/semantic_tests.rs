//! Tests for `../semantic.rs` (moved out of the source file).
use super::*;

#[test]
fn every_role_group_has_light_and_dark_treatment() {
    let light = SemanticTokens::light();
    let dark = SemanticTokens::dark();

    assert_ne!(light.background.canvas, dark.background.canvas);
    assert_ne!(light.background.subtle, dark.background.subtle);
    assert_ne!(light.foreground.primary, dark.foreground.primary);
    assert_ne!(light.foreground.muted, dark.foreground.muted);
    assert_ne!(light.border.default, dark.border.default);
    assert_ne!(light.border.focus, dark.border.focus);
    assert_ne!(light.accent.solid, dark.accent.solid);
    assert_ne!(light.status.success.solid, dark.status.success.solid);
    assert_ne!(light.status.danger.fill, dark.status.danger.fill);
    assert_ne!(light.syntax.keyword, dark.syntax.keyword);
}

#[test]
fn status_roles_carry_fill_border_subtle_and_foreground() {
    for theme_dark in [false, true] {
        let tokens = if theme_dark {
            SemanticTokens::dark()
        } else {
            SemanticTokens::light()
        };
        for role in [
            tokens.status.success,
            tokens.status.warning,
            tokens.status.danger,
            tokens.status.info,
        ] {
            assert_eq!(role.fill, role.solid.linear_multiply(STATUS_FILL_OPACITY));
            assert_eq!(role.border, role.solid.linear_multiply(STATUS_BORDER_OPACITY));
            assert_eq!(role.subtle, subtle_wash(role.solid, theme_dark));
            assert_eq!(role.foreground, role.solid);
        }
    }
}

#[test]
fn light_muted_coincides_with_tertiary_but_dark_deliberately_differs() {
    let light = SemanticTokens::light();
    let dark = SemanticTokens::dark();
    assert_eq!(light.foreground.muted, light.foreground.tertiary);
    assert_ne!(dark.foreground.muted, dark.foreground.tertiary);
}
