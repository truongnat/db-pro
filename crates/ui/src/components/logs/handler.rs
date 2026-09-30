use super::LogLevel;
use crate::DbProTheme;
use egui::Color32;
use lucide_icons::Icon;

pub fn level_style(level: LogLevel, theme: DbProTheme) -> (Icon, Color32) {
    match level {
        LogLevel::Info => (Icon::Info, theme.text_secondary),
        LogLevel::Notice => (Icon::Bell, theme.info),
        LogLevel::Warning => (Icon::TriangleAlert, theme.warning),
        LogLevel::Error => (Icon::CircleX, theme.danger),
    }
}

pub fn message_color(level: LogLevel, theme: DbProTheme) -> Color32 {
    if level == LogLevel::Error {
        theme.danger
    } else {
        theme.text_primary
    }
}

#[cfg(test)]
mod tests {
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
}
