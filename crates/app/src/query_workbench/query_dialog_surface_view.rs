use super::query_execution_state::PendingDestructiveRun;
use super::*;
use crate::app::overlay_state::ExportFormat;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use egui::RichText;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DestructiveDialogAction {
    Confirm,
    Cancel,
}

pub(super) struct DestructiveDialogContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) pending: &'a PendingDestructiveRun,
}

impl DestructiveDialogContext<'_> {
    /// ctx-level overlay variant: the held destructive statement must be
    /// confirmable no matter which output tab or dock state is active.
    pub(super) fn draw_ctx(&self, ctx: &egui::Context) -> Option<DestructiveDialogAction> {
        let mut open = true;
        let mut action = None;
        Dialog::new(&mut open, "Run Destructive Statement?", self.theme)
            .id_salt("destructive_run_dialog")
            .width(560.0)
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    self.draw_destructive_body(ui, &mut action);
                });
            });
        if !open {
            return Some(DestructiveDialogAction::Cancel);
        }
        action
    }


    fn draw_destructive_body(&self, ui: &mut egui::Ui, action: &mut Option<DestructiveDialogAction>) {
        ui.label(
            RichText::new(if self.pending.all_statements() {
                // cc-scan:allow LINE_TOO_LONG — literal must not wrap
                "The script you are about to run contains a statement that can drop or truncate data. Nothing has been sent yet."
            } else {
                "This statement can drop or truncate data. Nothing has been sent yet."
            })
            .color(self.theme.text_primary),
        );
        ui.add_space(SPACE_SM);
        let mut preview = self.pending.sql().to_owned();
        if preview.chars().count() > 600 {
            preview = preview.chars().take(600).collect::<String>() + "…";
        }
        Card::new(self.theme).show(ui, |ui| {
            ui.label(
                RichText::new(preview)
                    .font(font_mono_sm())
                    .color(self.theme.text_secondary),
            );
        });
        if !self.pending.target_label().is_empty() {
            ui.add_space(SPACE_SM);
            ui.label(
                RichText::new(format!("Runs on: {}", self.pending.target_label()))
                    .strong()
                    .color(self.theme.warning),
            );
        }
        ui.add_space(SPACE_SM);
        ui.colored_label(
            self.theme.warning,
            "It is sent to the server exactly as written; the app cannot undo it.",
        );
        ui.add_space(SPACE_MD);
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("Run Destructive Statement")
                .variant(ButtonVariant::Destructive)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(DestructiveDialogAction::Confirm);
            }
            if Button::new(self.theme)
                .text(t!("common.cancel").as_ref())
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(DestructiveDialogAction::Cancel);
            }
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExportDialogAction {
    Export,
    Cancel,
    Overwrite,
    KeepExisting,
}

pub(super) struct ExportDialogContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) overlay: &'a mut OverlayState,
}

impl ExportDialogContext<'_> {
    pub(super) fn draw(&mut self, ctx: &egui::Context) -> Option<ExportDialogAction> {
        let mut open = true;
        let mut action = None;
        Dialog::new(&mut open, t!("query.export_results").as_ref(), self.theme)
            .id_salt("export_results_dialog")
            .width(520.0)
            .description(t!("query.export_results_desc").as_ref())
            .show_ctx(ctx, |ui| self.draw_body(ui, &mut action));
        if !open {
            return Some(ExportDialogAction::Cancel);
        }
        action
    }

    fn draw_body(&mut self, ui: &mut egui::Ui, action: &mut Option<ExportDialogAction>) {
        ui.label(
            RichText::new(t!("query.export_format_label"))
                .font(font_ui_label())
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XS);
        self.draw_format_picker(ui);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new(t!("query.export_output_file"))
                .font(font_ui_label())
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XS);
        Input::new(&mut self.overlay.export_path, "/path/to/results.csv", self.theme).show(ui);
        if matches!(self.overlay.export_format, ExportFormat::SqlInsert | ExportFormat::SqlCopy) {
            ui.add_space(SPACE_XS);
            ui.label(
                RichText::new(t!("query.export_table_name"))
                    .font(font_ui_label())
                    .color(self.theme.text_muted),
            );
            ui.add_space(SPACE_XS);
            Input::new(&mut self.overlay.export_table_name, "exported_rows", self.theme).show(ui);
        }
        ui.add_space(SPACE_XS);
        ui.label(
            RichText::new(t!("query.export_visible_only"))
                .font(font_caption())
                .color(self.theme.text_muted),
        );
        if self.overlay.export_overwrite_pending {
            self.draw_overwrite_confirmation(ui, action);
        }
        ui.add_space(SPACE_MD);
        self.draw_footer(ui, action);
    }

    fn draw_format_picker(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for format in ExportFormat::ALL {
                let selected = self.overlay.export_format == format;
                if Button::new(self.theme)
                    .text(format.label())
                    .variant(if selected {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Ghost
                    })
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.overlay.export_format = format;
                }
            }
        });
    }

    fn draw_footer(&self, ui: &mut egui::Ui, action: &mut Option<ExportDialogAction>) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if Button::new(self.theme)
                .text(t!("common.export").as_ref())
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(ExportDialogAction::Export);
            }
            if Button::new(self.theme)
                .text(t!("common.cancel").as_ref())
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(ExportDialogAction::Cancel);
            }
        });
    }

    fn draw_overwrite_confirmation(&self, ui: &mut egui::Ui, action: &mut Option<ExportDialogAction>) {
        ui.add_space(SPACE_XS);
        ui.colored_label(
            self.theme.warning,
            t!("query.export_overwrite_prompt", path = self.overlay.export_path.trim()).into_owned(),
        );
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text(t!("query.overwrite").as_ref())
                .variant(ButtonVariant::Destructive)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(ExportDialogAction::Overwrite);
            }
            if Button::new(self.theme)
                .text(t!("query.keep_existing_file").as_ref())
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(ExportDialogAction::KeepExisting);
            }
        });
    }
}

#[cfg(test)]
#[path = "tests/query_dialog_surface_view_tests.rs"]
mod tests;

