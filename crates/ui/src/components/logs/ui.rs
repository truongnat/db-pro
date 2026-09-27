use super::{config, handler};
use crate::DbProTheme;
use egui::{Response, RichText, Rounding, Stroke, Ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Notice,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub query_id: Option<String>,
}
impl LogEntry {
    pub fn new(timestamp: impl Into<String>, level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: timestamp.into(),
            level,
            message: message.into(),
            query_id: None,
        }
    }
    pub fn query_id(mut self, id: impl Into<String>) -> Self {
        self.query_id = Some(id.into());
        self
    }
}

pub struct LogViewer<'a> {
    entries: &'a [LogEntry],
    theme: DbProTheme,
}
impl<'a> LogViewer<'a> {
    pub fn new(entries: &'a [LogEntry], theme: DbProTheme) -> Self {
        Self { entries, theme }
    }
    pub fn show(self, ui: &mut Ui) -> Response {
        egui::Frame::none()
            .fill(self.theme.surface_editor)
            .stroke(Stroke::new(config::FRAME_STROKE, self.theme.border_subtle))
            .rounding(Rounding::same(config::FRAME_RADIUS))
            .inner_margin(egui::Margin::symmetric(config::FRAME_MARGIN_X, config::FRAME_MARGIN_Y))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                if self.entries.is_empty() {
                    ui.label(
                        RichText::new(config::EMPTY_MESSAGE)
                            .size(config::EMPTY_FONT_SIZE)
                            .color(self.theme.text_tertiary),
                    );
                    return;
                }
                for entry in self.entries {
                    let (icon, color) = handler::level_style(entry.level, self.theme);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&entry.timestamp)
                                .size(config::LOG_FONT_SIZE)
                                .monospace()
                                .color(self.theme.text_tertiary),
                        );
                        ui.add_space(config::ROW_GAP);
                        ui.label(
                            RichText::new(char::from(icon).to_string())
                                .font(crate::tokens::font_icon(crate::tokens::ICON_XS))
                                .color(color),
                        );
                        ui.label(
                            RichText::new(&entry.message)
                                .size(config::LOG_FONT_SIZE)
                                .monospace()
                                .color(handler::message_color(entry.level, self.theme)),
                        );
                    });
                    ui.add_space(config::ROW_GAP);
                }
            })
            .response
    }
}
