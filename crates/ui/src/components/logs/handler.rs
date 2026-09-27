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
    }
}
