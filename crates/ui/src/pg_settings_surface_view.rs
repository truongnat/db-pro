//! PostgreSQL settings presentation and typed user intents.
use super::super::pg_settings_state::PgSettingsState;
use super::super::*;
use crate::components::dialog::Dialog;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PgSettingsSurfaceAction {
    Refresh,
    BeginEdit { name: String, value: String },
    Reset(String),
    PreviewAlterSystem { name: String, value: String },
    ApplySession { name: String, value: String },
    CancelEdit,
    ClosePreview,
}

pub(super) struct PgSettingsSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut PgSettingsState,
}

impl PgSettingsSurfaceContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<PgSettingsSurfaceAction> {
        let mut actions = Vec::new();
        self.draw_header(ui, &mut actions);
        self.draw_settings(ui, &mut actions);
        self.draw_edit_dialog(ui, &mut actions);
        self.draw_preview(ui, &mut actions);
        actions
    }

    fn draw_header(&mut self, ui: &mut egui::Ui, actions: &mut Vec<PgSettingsSurfaceAction>) {
        ui.add_space(SPACE_MD);
        section_label(ui, "SERVER SETTINGS (pg_settings)", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("PostgreSQL-only · session SET/RESET for user-context GUCs · ALTER SYSTEM is preview-only")
                .small()
                .color(self.theme.text_muted),
        );
        if secondary_button_with_icon(ui, Icon::RefreshCw, "Load settings", self.theme).clicked() {
            actions.push(PgSettingsSurfaceAction::Refresh);
        }
        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Filter").small().color(self.theme.text_muted));
            ui.text_edit_singleline(&mut self.state.pg_settings_filter);
        });
        if let Some(error) = &self.state.pg_settings_error {
            ui.colored_label(self.theme.danger, error);
        }
    }

    fn draw_settings(&self, ui: &mut egui::Ui, actions: &mut Vec<PgSettingsSurfaceAction>) {
        let Some(snapshot) = self.state.pg_settings.as_ref() else {
            return;
        };
        ui.label(
            RichText::new(format!(
                "{} · fetched @ {} ms",
                snapshot.message, snapshot.fetched_at_ms
            ))
            .small()
            .color(self.theme.text_muted),
        );
        let filter = self.state.pg_settings_filter.to_ascii_lowercase();
        let settings: Vec<_> = snapshot
            .settings
            .iter()
            .filter(|setting| {
                filter.is_empty()
                    || setting.name.to_ascii_lowercase().contains(&filter)
                    || setting.category.to_ascii_lowercase().contains(&filter)
                    || setting.source.to_ascii_lowercase().contains(&filter)
            })
            .take(60)
            .collect();
        for setting in settings {
            self.draw_setting_card(ui, setting, actions);
        }
    }

    fn draw_setting_card(
        &self,
        ui: &mut egui::Ui,
        setting: &db_pro_core::domain::pg_settings::PgSetting,
        actions: &mut Vec<PgSettingsSurfaceAction>,
    ) {
        let name = setting.name.clone();
        let value = setting.setting.clone();
        card_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&setting.name)
                        .strong()
                        .monospace()
                        .color(self.theme.text_primary),
                );
                if setting.pending_restart {
                    badge(ui, "pending restart", self.theme.warning, self.theme.text_primary);
                }
                if setting.sensitive {
                    badge(ui, "redacted", self.theme.surface_active, self.theme.text_secondary);
                }
            });
            ui.label(
                RichText::new(format!(
                    "{} · context={} · source={} · {}",
                    setting.category,
                    setting.context,
                    setting.source,
                    setting.display_setting()
                ))
                .small()
                .color(self.theme.text_secondary),
            );
            if let Some(description) = &setting.short_desc {
                ui.label(RichText::new(description).small().color(self.theme.text_muted));
            }
            ui.label(
                RichText::new(setting.mutability_reason())
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.horizontal(|ui| {
                if setting.session_mutable() && !setting.sensitive {
                    if ghost_button_with_icon(ui, Icon::Pencil, "Edit session", self.theme).clicked() {
                        actions.push(PgSettingsSurfaceAction::BeginEdit {
                            name: name.clone(),
                            value: value.clone(),
                        });
                    }
                    if secondary_button(ui, "RESET", self.theme).clicked() {
                        actions.push(PgSettingsSurfaceAction::Reset(name.clone()));
                    }
                }
                if !setting.sensitive
                    && ghost_button_with_icon(ui, Icon::FileCode2, "Preview ALTER SYSTEM", self.theme).clicked()
                {
                    actions.push(PgSettingsSurfaceAction::PreviewAlterSystem {
                        name: name.clone(),
                        value: value.clone(),
                    });
                }
            });
        });
        ui.add_space(SPACE_SM);
    }

    fn draw_edit_dialog(&mut self, ui: &mut egui::Ui, actions: &mut Vec<PgSettingsSurfaceAction>) {
        if self.state.pg_settings_edit_name.is_empty() {
            return;
        }
        let name = self.state.pg_settings_edit_name.clone();
        let mut open = true;
        Dialog::new(&mut open, format!("SET SESSION · {name}"), self.theme)
            .width(460.0)
            .id_salt("pg_settings_edit_dialog")
            .show_framed_ctx(ui.ctx(), |frame| {
                frame.body(|ui| {
                    ui.text_edit_singleline(&mut self.state.pg_settings_edit_value);
                });
                frame.footer(|ui| {
                    if secondary_button(ui, "Apply SET", self.theme).clicked() {
                        actions.push(PgSettingsSurfaceAction::ApplySession {
                            name: name.clone(),
                            value: self.state.pg_settings_edit_value.clone(),
                        });
                    }
                    if ghost_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                        actions.push(PgSettingsSurfaceAction::CancelEdit);
                    }
                });
            });
        if !open {
            actions.push(PgSettingsSurfaceAction::CancelEdit);
        }
    }

    fn draw_preview(&self, ui: &mut egui::Ui, actions: &mut Vec<PgSettingsSurfaceAction>) {
        let Some(preview) = self.state.pg_settings_preview.as_ref() else {
            return;
        };
        let mut open = true;
        Dialog::new(&mut open, "ALTER SYSTEM preview", self.theme)
            .width(560.0)
            .id_salt("pg_settings_preview_dialog")
            .show_framed_ctx(ui.ctx(), |frame| {
                frame.body(|ui| {
                    ui.label(RichText::new(&preview.note).small().color(self.theme.warning));
                    ui.label(RichText::new(&preview.sql).monospace());
                });
                frame.footer(|ui| {
                    if secondary_button(ui, "Close", self.theme).clicked() {
                        actions.push(PgSettingsSurfaceAction::ClosePreview);
                    }
                });
            });
        if !open {
            actions.push(PgSettingsSurfaceAction::ClosePreview);
        }
    }
}
