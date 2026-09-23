use super::git_workspace::{GitDiffResult, GitWorkspaceStatus};
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FilesGitAction {
    Refresh,
    ReloadExternalFile(String),
    DismissExternalFile,
    CommitStaged,
    Stage(String),
    Unstage(String),
    Diff(String),
    Open(String),
}

pub(super) struct FilesGitContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) status: Option<&'a GitWorkspaceStatus>,
    pub(super) last_error: Option<&'a str>,
    pub(super) external_change: Option<&'a str>,
    pub(super) commit_message: &'a mut String,
    pub(super) diff: Option<&'a GitDiffResult>,
}

impl FilesGitContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<FilesGitAction> {
        let mut actions = self.draw_header(ui);
        actions.extend(self.draw_external_change(ui));
        if let Some(error) = self.last_error {
            ui.colored_label(self.theme.danger, error);
        }
        let Some(status) = self.status else {
            self.draw_status_placeholder(ui);
            return actions;
        };
        if !status.available {
            ui.label(RichText::new(&status.message).small().color(self.theme.text_muted));
            return actions;
        }
        actions.extend(self.draw_status(ui, status));
        self.draw_diff(ui);
        actions
    }

    fn draw_header(&self, ui: &mut egui::Ui) -> Vec<FilesGitAction> {
        let mut actions = Vec::new();
        ui.horizontal(|ui| {
            section_label(ui, "GIT", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if Button::new(self.theme)
                    .icon(Icon::RefreshCw)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Refresh git status")
                    .show(ui)
                    .clicked()
                {
                    actions.push(FilesGitAction::Refresh);
                }
            });
        });
        ui.add_space(4.0);
        actions
    }

    fn draw_external_change(&self, ui: &mut egui::Ui) -> Vec<FilesGitAction> {
        let Some(path) = self.external_change else {
            return Vec::new();
        };
        let mut actions = Vec::new();
        ui.colored_label(
            self.theme.warning,
            format!("Disk changed for {path} — unsaved editor buffer was kept."),
        );
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("Reload from disk")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesGitAction::ReloadExternalFile(path.to_owned()));
            }
            if Button::new(self.theme)
                .icon(Icon::X)
                .text("Dismiss")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesGitAction::DismissExternalFile);
            }
        });
        ui.add_space(6.0);
        actions
    }

    fn draw_status_placeholder(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Refresh to probe Git for the active workspace root.")
                .small()
                .color(self.theme.text_muted),
        );
    }

    fn draw_status(&mut self, ui: &mut egui::Ui, status: &GitWorkspaceStatus) -> Vec<FilesGitAction> {
        let mut actions = Vec::new();
        ui.label(
            RichText::new(format!(
                "branch {} · {}",
                status.branch.as_deref().unwrap_or("?"),
                status.message
            ))
            .small()
            .color(self.theme.text_secondary),
        );
        ui.add_space(6.0);
        ui.add(
            egui::TextEdit::singleline(self.commit_message)
                .hint_text("commit message (explicit only — never auto)")
                .desired_width(ui.available_width()),
        );
        if Button::new(self.theme)
            .text("Commit staged")
            .variant(ButtonVariant::Destructive)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            actions.push(FilesGitAction::CommitStaged);
        }
        ui.add_space(8.0);
        if status.entries.is_empty() {
            ui.label(
                RichText::new("Working tree clean.")
                    .small()
                    .color(self.theme.text_muted),
            );
        }
        for entry in status.entries.iter().take(80) {
            actions.extend(self.draw_entry(ui, entry));
        }
        actions
    }

    fn draw_entry(&self, ui: &mut egui::Ui, entry: &super::git_workspace::GitFileStatus) -> Vec<FilesGitAction> {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("[{}] {}", entry.code.trim(), entry.path))
                    .small()
                    .monospace()
                    .color(self.theme.text_primary),
            );
        });
        let actions = self.draw_entry_actions(ui, entry);
        ui.add_space(4.0);
        actions
    }

    fn draw_entry_actions(
        &self,
        ui: &mut egui::Ui,
        entry: &super::git_workspace::GitFileStatus,
    ) -> Vec<FilesGitAction> {
        let mut actions = Vec::new();
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .icon(Icon::Plus)
                .text("Stage")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesGitAction::Stage(entry.path.clone()));
            }
            if Button::new(self.theme)
                .icon(Icon::Minus)
                .text("Unstage")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesGitAction::Unstage(entry.path.clone()));
            }
            if Button::new(self.theme)
                .icon(Icon::GitCompare)
                .text("Diff")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesGitAction::Diff(entry.path.clone()));
            }
            if Button::new(self.theme)
                .icon(Icon::FileCode2)
                .text("Open")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(FilesGitAction::Open(entry.path.clone()));
            }
        });
        actions
    }

    fn draw_diff(&self, ui: &mut egui::Ui) {
        let Some(diff) = self.diff else {
            return;
        };
        ui.add_space(8.0);
        section_label(ui, format!("DIFF · {} vs {}", diff.path, diff.against), self.theme);
        ui.add_space(4.0);
        egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
            ui.label(
                RichText::new(diff.text.chars().take(8_000).collect::<String>())
                    .small()
                    .monospace()
                    .color(self.theme.text_muted),
            );
        });
    }
}
