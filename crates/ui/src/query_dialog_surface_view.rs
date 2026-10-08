use super::query_execution_state::PendingDestructiveRun;
use super::*;
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
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> DestructiveDialogAction {
        let mut open = true;
        let mut action = DestructiveDialogAction::Cancel;
        Dialog::new(&mut open, "Run Destructive Statement?", self.theme)
            .id_salt("destructive_run_dialog")
            .width(560.0)
            .show(ui, |ui| self.draw_destructive_body(ui, &mut action));
        if !open {
            DestructiveDialogAction::Cancel
        } else {
            action
        }
    }

    fn draw_destructive_body(&self, ui: &mut egui::Ui, action: &mut DestructiveDialogAction) {
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
        editor_frame(self.theme).show(ui, |ui| {
            ui.label(
                RichText::new(preview)
                    .font(font_mono_sm())
                    .color(self.theme.text_secondary),
            );
        });
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
                *action = DestructiveDialogAction::Confirm;
            }
            if Button::new(self.theme)
                .text("Cancel")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = DestructiveDialogAction::Cancel;
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
        Dialog::new(&mut open, "Export Results", self.theme)
            .id_salt("export_results_dialog")
            .width(520.0)
            .description("Write the visible result rows to a file on disk.")
            .show_ctx(ctx, |ui| self.draw_body(ui, &mut action));
        if !open {
            return Some(ExportDialogAction::Cancel);
        }
        action
    }

    fn draw_body(&mut self, ui: &mut egui::Ui, action: &mut Option<ExportDialogAction>) {
        ui.label(
            RichText::new("Format")
                .font(font_ui_label())
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XS);
        self.draw_format_picker(ui);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("Output file")
                .font(font_ui_label())
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XS);
        input_full_width(ui, &mut self.overlay.export_path, "/path/to/results.csv", self.theme);
        ui.add_space(SPACE_XS);
        ui.label(
            RichText::new("Rows beyond the visible limit are not included in the export.")
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
            for (value, label) in [
                ("CSV", "CSV"),
                ("TSV", "TSV"),
                ("JSON", "JSON"),
                ("MD", "Markdown"),
                ("SQL", "INSERT"),
                ("COPY", "COPY"),
            ] {
                let selected = self.overlay.export_format == value;
                if Button::new(self.theme)
                    .text(label)
                    .variant(if selected {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Ghost
                    })
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.overlay.export_format = value.to_owned();
                }
            }
        });
    }

    fn draw_footer(&self, ui: &mut egui::Ui, action: &mut Option<ExportDialogAction>) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if Button::new(self.theme)
                .text("Export")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(ExportDialogAction::Export);
            }
            if Button::new(self.theme)
                .text("Cancel")
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
            format!("{} already exists. Overwrite it?", self.overlay.export_path.trim()),
        );
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("Overwrite")
                .variant(ButtonVariant::Destructive)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(ExportDialogAction::Overwrite);
            }
            if Button::new(self.theme)
                .text("Keep existing file")
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
mod tests {
    use super::*;

    #[test]
    fn dialog_actions_keep_confirmation_boundaries_explicit() {
        assert_ne!(DestructiveDialogAction::Confirm, DestructiveDialogAction::Cancel);
        assert_ne!(ExportDialogAction::Export, ExportDialogAction::Overwrite);
    }
}
