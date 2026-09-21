use super::*;
use db_pro_core::domain::saved_task::{SavedTask, SavedTaskPayload, SavedTaskRunStatus};
use lucide_icons::Icon;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SavedTasksSurfaceAction {
    NewSqlTask,
    NewBackupTask,
    SaveDraft,
    CancelDraft,
    RunTask(Uuid),
    DisableSchedule(Uuid),
    EnableSchedule(Uuid),
    DeleteTask(Uuid),
}

pub(super) struct SavedTasksSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) saved_tasks: &'a mut SavedTaskState,
}

impl SavedTasksSurfaceContext<'_> {
    pub(super) fn draw(mut self, ui: &mut egui::Ui) -> Option<SavedTasksSurfaceAction> {
        let mut action = None;
        self.draw_header(ui, &mut action);
        if self.saved_tasks.draft.is_some() {
            self.draw_draft_form(ui, &mut action);
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);
        }
        if self.saved_tasks.store.tasks.is_empty() {
            empty_state(
                ui,
                Icon::ListTodo,
                "No saved tasks",
                "Create a SQL, export, backup, or maintenance task to run manually.",
                self.theme,
            );
            return action;
        }
        let tasks = self.saved_tasks.store.tasks.clone();
        for task in tasks {
            self.draw_task_card(ui, &task, &mut action);
        }
        self.draw_recent_runs(ui);
        action
    }

    fn draw_header(&self, ui: &mut egui::Ui, action: &mut Option<SavedTasksSurfaceAction>) {
        section_label(ui, "SAVED TASKS", self.theme);
        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "Scheduler runs only while this app is open — there is no background daemon. Secrets stay on the connection.",
            )
            .small()
            .color(self.theme.text_muted),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("New SQL task")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(SavedTasksSurfaceAction::NewSqlTask);
            }
            if Button::new(self.theme)
                .text("New backup task")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(SavedTasksSurfaceAction::NewBackupTask);
            }
        });
        ui.add_space(8.0);
    }

    fn draw_draft_form(&mut self, ui: &mut egui::Ui, action: &mut Option<SavedTasksSurfaceAction>) {
        let theme = self.theme;
        let Some(draft) = self.saved_tasks.draft.as_mut() else {
            return;
        };
        ui.label(RichText::new("New task").strong().color(theme.text_primary));
        input_full_width(ui, &mut draft.name, "Name", theme);
        input_full_width(ui, &mut draft.description, "Description", theme);
        Self::draw_payload_fields(ui, &mut draft.payload, theme);
        Self::draw_draft_actions(ui, action, theme);
    }

    fn draw_payload_fields(ui: &mut egui::Ui, payload: &mut SavedTaskPayload, theme: DbProTheme) {
        match payload {
            SavedTaskPayload::Sql { sql } => {
                ui.label(RichText::new("SQL").small().color(theme.text_muted));
                ui.add(
                    egui::TextEdit::multiline(sql)
                        .desired_width(ui.available_width())
                        .desired_rows(4)
                        .code_editor(),
                );
            }
            SavedTaskPayload::Backup {
                output_path,
                custom_format,
            } => {
                input_full_width(ui, output_path, "Output path", theme);
                ui.checkbox(custom_format, "Custom (pg_dump -Fc) format");
            }
            SavedTaskPayload::Export { table, format } => {
                let mut table_text = table.clone().unwrap_or_default();
                input_full_width(ui, &mut table_text, "Table (optional)", theme);
                *table = (!table_text.trim().is_empty()).then_some(table_text);
                input_full_width(ui, format, "Format (csv/tsv/json)", theme);
            }
            SavedTaskPayload::Maintenance { operation, target } => {
                input_full_width(ui, operation, "Operation (vacuum/analyze)", theme);
                let mut target_text = target.clone().unwrap_or_default();
                input_full_width(ui, &mut target_text, "Target (optional)", theme);
                *target = (!target_text.trim().is_empty()).then_some(target_text);
            }
        }
    }

    fn draw_draft_actions(ui: &mut egui::Ui, action: &mut Option<SavedTasksSurfaceAction>, theme: DbProTheme) {
        ui.horizontal(|ui| {
            if Button::new(theme)
                .text("Save task")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(SavedTasksSurfaceAction::SaveDraft);
            }
            if Button::new(theme)
                .text("Cancel")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(SavedTasksSurfaceAction::CancelDraft);
            }
        });
    }

    fn draw_task_card(&self, ui: &mut egui::Ui, task: &SavedTask, action: &mut Option<SavedTasksSurfaceAction>) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&task.name).strong().color(self.theme.text_primary));
                ui.label(RichText::new(task.kind().label()).small().color(self.theme.text_muted));
            });
            if !task.description.is_empty() {
                ui.label(
                    RichText::new(&task.description)
                        .small()
                        .color(self.theme.text_secondary),
                );
            }
            ui.label(
                RichText::new(format!("connection · {}", task.connection_id))
                    .small()
                    .color(self.theme.text_muted),
            );
            self.draw_last_run(ui, task);
            self.draw_task_actions(ui, task, action);
            if let Some(schedule) = &task.schedule {
                let next = schedule
                    .next_run_at
                    .map(|time| time.to_rfc3339())
                    .unwrap_or_else(|| "—".into());
                ui.label(
                    RichText::new(format!(
                        "schedule · {} · next {next}",
                        if schedule.enabled { "on" } else { "off" }
                    ))
                    .small()
                    .color(self.theme.text_muted),
                );
            }
        });
        ui.add_space(6.0);
    }

    fn draw_last_run(&self, ui: &mut egui::Ui, task: &SavedTask) {
        if let Some(run) = &task.last_run {
            let status = match run.status {
                SavedTaskRunStatus::Success => "success",
                SavedTaskRunStatus::Failed => "failed",
                SavedTaskRunStatus::Cancelled => "cancelled",
            };
            ui.label(
                RichText::new(format!("last · {status} · {} ms · {}", run.duration_ms, run.message))
                    .small()
                    .color(self.theme.text_muted),
            );
        }
    }

    fn draw_task_actions(&self, ui: &mut egui::Ui, task: &SavedTask, action: &mut Option<SavedTasksSurfaceAction>) {
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("Run")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(SavedTasksSurfaceAction::RunTask(task.id));
            }
            if task.schedule.as_ref().is_some_and(|schedule| schedule.enabled) {
                if Button::new(self.theme)
                    .text("Disable schedule")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    *action = Some(SavedTasksSurfaceAction::DisableSchedule(task.id));
                }
            } else if Button::new(self.theme)
                .text("Schedule 60s")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(SavedTasksSurfaceAction::EnableSchedule(task.id));
            }
            if Button::new(self.theme)
                .text("Delete")
                .variant(ButtonVariant::Destructive)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *action = Some(SavedTasksSurfaceAction::DeleteTask(task.id));
            }
        });
    }

    fn draw_recent_runs(&self, ui: &mut egui::Ui) {
        if self.saved_tasks.store.runs.is_empty() {
            return;
        }
        ui.separator();
        ui.add_space(6.0);
        section_label(ui, "RECENT RUNS", self.theme);
        for run in self.saved_tasks.store.runs.iter().take(12) {
            let status = match run.status {
                SavedTaskRunStatus::Success => "OK",
                SavedTaskRunStatus::Failed => "FAIL",
                SavedTaskRunStatus::Cancelled => "CANCEL",
            };
            ui.label(
                RichText::new(format!(
                    "{status} · {} ms · {}",
                    run.duration_ms,
                    run.message.chars().take(80).collect::<String>()
                ))
                .small()
                .monospace()
                .color(self.theme.text_muted),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_action_keeps_task_identity_for_root_reduction() {
        let id = Uuid::new_v4();
        assert_eq!(
            SavedTasksSurfaceAction::RunTask(id),
            SavedTasksSurfaceAction::RunTask(id)
        );
    }
}
