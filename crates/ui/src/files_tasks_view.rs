use super::ide_workspace::TaskRunResult;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FilesTasksAction {
    RunTask,
    RunBenchmark,
}

pub(super) struct FilesTasksContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) command: &'a mut String,
    pub(super) last_task: Option<&'a TaskRunResult>,
}

impl FilesTasksContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<FilesTasksAction> {
        let mut actions = self.draw_command_form(ui);
        self.draw_last_task(ui);
        actions.extend(self.draw_benchmark(ui));
        actions
    }

    fn draw_command_form(&mut self, ui: &mut egui::Ui) -> Vec<FilesTasksAction> {
        let mut actions = Vec::new();
        ui.add(
            egui::TextEdit::singleline(self.command)
                .hint_text("shell command in workspace root…")
                .desired_width(ui.available_width()),
        );
        ui.add_space(4.0);
        if Button::new(self.theme)
            .text("Run")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            actions.push(FilesTasksAction::RunTask);
        }
        actions
    }

    fn draw_last_task(&self, ui: &mut egui::Ui) {
        let Some(result) = self.last_task else {
            return;
        };
        ui.add_space(6.0);
        ui.label(
            RichText::new(format!(
                "$ {} · exit {:?} · {}ms",
                result.command, result.exit_code, result.duration_ms
            ))
            .small()
            .monospace()
            .color(self.theme.text_secondary),
        );
        if !result.stdout.is_empty() {
            ui.label(
                RichText::new(result.stdout.chars().take(800).collect::<String>())
                    .small()
                    .monospace()
                    .color(self.theme.text_muted),
            );
        }
        if !result.stderr.is_empty() {
            ui.label(
                RichText::new(result.stderr.chars().take(400).collect::<String>())
                    .small()
                    .monospace()
                    .color(self.theme.danger),
            );
        }
    }

    fn draw_benchmark(&self, ui: &mut egui::Ui) -> Vec<FilesTasksAction> {
        let mut actions = Vec::new();
        ui.add_space(8.0);
        section_label(ui, "BENCHMARK (local timing)", self.theme);
        if Button::new(self.theme)
            .text("Run sample suite")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            actions.push(FilesTasksAction::RunBenchmark);
        }
        actions
    }
}
