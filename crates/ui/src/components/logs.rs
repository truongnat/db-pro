//! Database Execution Logs, Notices, and Diagnostics components.
//!
//! Implements LogViewer, LogEntry, DatabaseError, and DatabaseNotice
//! per `open-ai-refer.md`.

use crate::tokens::*;
use crate::DbProTheme;
use egui::{Response, RichText, Rounding, Stroke, Ui};
use lucide_icons::Icon;

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
        let frame = egui::Frame::none()
            .fill(self.theme.surface_editor)
            .stroke(Stroke::new(STROKE_THIN, self.theme.border_subtle))
            .rounding(Rounding::same(RADIUS_CARD))
            .inner_margin(egui::Margin::symmetric(SPACE_MD, SPACE_SM));

        frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                if self.entries.is_empty() {
                    ui.label(
                        RichText::new("No execution logs recorded.")
                            .size(FONT_SIZE_CAPTION)
                            .color(self.theme.text_tertiary),
                    );
                } else {
                    for entry in self.entries {
                        let (icon, color) = match entry.level {
                            LogLevel::Info => (Icon::Info, self.theme.text_secondary),
                            LogLevel::Notice => (Icon::Bell, self.theme.info),
                            LogLevel::Warning => (Icon::TriangleAlert, self.theme.warning),
                            LogLevel::Error => (Icon::CircleX, self.theme.danger),
                        };

                        ui.horizontal(|ui| {
                            // Timestamp
                            ui.label(
                                RichText::new(&entry.timestamp)
                                    .size(FONT_SIZE_MONO_SM)
                                    .monospace()
                                    .color(self.theme.text_tertiary),
                            );

                            ui.add_space(SPACE_XXS);

                            // Level Icon
                            ui.label(
                                RichText::new(char::from(icon).to_string())
                                    .font(font_icon(ICON_XS))
                                    .color(color),
                            );

                            // Message
                            ui.label(RichText::new(&entry.message).size(FONT_SIZE_MONO_SM).monospace().color(
                                if entry.level == LogLevel::Error {
                                    self.theme.danger
                                } else {
                                    self.theme.text_primary
                                },
                            ));
                        });

                        ui.add_space(SPACE_XXS);
                    }
                }
            })
            .response
    }
}
