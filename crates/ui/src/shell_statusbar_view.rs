//! Native shell statusbar rendering and intent collection.
use super::*;

pub(super) enum ShellStatusbarAction {
    ToggleOutput,
}

pub(super) struct ShellStatusbarContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) icon: Icon,
    pub(super) icon_color: egui::Color32,
    pub(super) label: &'a str,
    pub(super) runtime_status: Option<(String, egui::Color32)>,
    pub(super) connected: bool,
    pub(super) connection_name: &'a str,
    pub(super) driver: &'a str,
    pub(super) database: Option<&'a str>,
    pub(super) schema: Option<&'a str>,
    pub(super) duration_ms: Option<u64>,
    pub(super) editor_status: bool,
    pub(super) cursor_line: usize,
    pub(super) cursor_column: usize,
    pub(super) context_label: &'a str,
}

impl ShellStatusbarContext<'_> {
    pub(super) fn draw(&self, ctx: &egui::Context) -> Option<ShellStatusbarAction> {
        let mut action = None;
        TopBottomPanel::bottom("statusbar")
            .exact_height(28.0)
            .frame(egui::Frame {
                fill: self.theme.surface_panel,
                inner_margin: egui::Margin::symmetric(SPACE_MD, SPACE_XXS),
                stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                ui.horizontal_centered(|ui| {
                    self.draw_identity(ui);
                    self.draw_runtime_status(ui);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if Button::new(self.theme)
                            .icon(Icon::PanelBottom)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip("Toggle output panel")
                            .show(ui)
                            .clicked()
                        {
                            action = Some(ShellStatusbarAction::ToggleOutput);
                        }
                        self.draw_editor_context(ui);
                    });
                });
            });
        action
    }

    fn draw_identity(&self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_SM);
        ui.label(icon_text(self.icon, "", self.icon_color));
        ui.label(
            RichText::new(self.label)
                .font(font_caption())
                .color(self.theme.text_secondary),
        );
        ui.separator();
        if self.connected {
            ui.label(
                RichText::new(self.connection_name)
                    .font(font_caption())
                    .color(self.theme.text_secondary),
            );
        }
        ui.label(
            RichText::new(self.driver)
                .font(font_caption())
                .color(self.theme.text_muted),
        );
        if let Some(database) = self.database {
            ui.label(
                RichText::new(database)
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
        }
        if let Some(schema) = self.schema {
            ui.label(RichText::new(schema).font(font_caption()).color(self.theme.text_muted));
        }
        if let Some(duration_ms) = self.duration_ms {
            ui.label(
                RichText::new(format!("{duration_ms} ms"))
                    .font(font_mono_sm())
                    .color(self.theme.text_muted),
            );
        }
    }

    fn draw_runtime_status(&self, ui: &mut egui::Ui) {
        if let Some((message, message_color)) = &self.runtime_status {
            ui.separator();
            ui.add_sized(
                [260.0, 18.0],
                egui::Label::new(RichText::new(message).font(font_caption()).color(*message_color)),
            );
        }
    }

    fn draw_editor_context(&self, ui: &mut egui::Ui) {
        if self.editor_status {
            ui.label(RichText::new("UTF-8").font(font_mono_sm()).color(self.theme.text_muted));
            ui.label(
                RichText::new(format!("Ln {}, Col {}", self.cursor_line, self.cursor_column))
                    .font(font_mono_sm())
                    .color(self.theme.text_muted),
            );
        } else {
            ui.label(
                RichText::new(self.context_label)
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
        }
    }
}
