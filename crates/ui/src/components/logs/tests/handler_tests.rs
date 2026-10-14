//! Tests for `../handler.rs` (moved out of the source file).
use super::*;
#[test]
fn only_errors_use_danger_message_color() {
    let theme = DbProTheme::light();
    assert_eq!(message_color(LogLevel::Error, theme), theme.danger);
    assert_eq!(message_color(LogLevel::Info, theme), theme.text_primary);
    assert_eq!(message_color(LogLevel::Notice, theme), theme.text_primary);
    assert_eq!(message_color(LogLevel::Warning, theme), theme.text_primary);
}

#[test]
fn each_level_maps_to_its_semantic_icon_and_color() {
    let theme = DbProTheme::light();
    let (icon, color) = level_style(LogLevel::Info, theme);
    assert!(matches!(icon, Icon::Info));
    assert_eq!(color, theme.text_secondary);
    let (icon, color) = level_style(LogLevel::Notice, theme);
    assert!(matches!(icon, Icon::Bell));
    assert_eq!(color, theme.info);
    let (icon, color) = level_style(LogLevel::Warning, theme);
    assert!(matches!(icon, Icon::TriangleAlert));
    assert_eq!(color, theme.warning);
    let (icon, color) = level_style(LogLevel::Error, theme);
    assert!(matches!(icon, Icon::CircleX));
    assert_eq!(color, theme.danger);
}
