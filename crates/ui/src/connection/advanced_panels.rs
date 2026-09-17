use super::config::*;
use super::layout::*;
use super::mapper::apply_connection_snippet;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::input::Input;
use crate::components::tabs::SegmentedTabs;
use crate::tokens::*;
use crate::{DbProApp, DbProTheme, UiCommand, UiSslMode};
use egui::{FontFamily, FontId, Frame, Margin, RichText, Rounding, Stroke};
use lucide_icons::Icon;

impl DbProApp {
    /// Panel 1: Cloud Presets & URI Importer
    pub(crate) fn draw_cloud_presets_panel(&mut self, ui: &mut egui::Ui) {
        Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            rounding: Rounding::same(RADIUS_CARD),
            inner_margin: Margin::symmetric(12.0, 8.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            egui::CollapsingHeader::new(
                RichText::new("Cloud Presets & URI Importer")
                    .font(DbProTheme::ui_medium_font(11.5))
                    .color(self.theme.text_primary),
            )
            .id_salt("conn_cloud_presets_panel")
            .show(ui, |ui| {
                ui.add_space(SPACE_XS);
                ui.horizontal(|ui| {
                    let selected = if self.connection_draft.cloud_preset.is_empty() {
                        "Select a cloud preset…".to_owned()
                    } else {
                        self.connection_draft.cloud_preset.clone()
                    };
                    egui::ComboBox::from_id_salt("cloud_preset")
                        .selected_text(selected)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.connection_draft.cloud_preset,
                                String::new(),
                                "None (manual)",
                            );
                            for option in CLOUD_PRESET_OPTIONS {
                                ui.selectable_value(
                                    &mut self.connection_draft.cloud_preset,
                                    option.key.to_owned(),
                                    option.label,
                                );
                            }
                        });
                    if Button::new(self.theme)
                        .text("Apply Preset")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.apply_cloud_preset();
                    }
                });
                ui.add_space(SPACE_XS);
                ui.horizontal(|ui| {
                    let mut auth_idx = if self.connection_draft.auth_kind == "ephemeral_token" {
                        1
                    } else {
                        0
                    };
                    SegmentedTabs::new(&mut auth_idx, AUTH_KIND_OPTIONS, self.theme).show(ui);
                    self.connection_draft.auth_kind = if auth_idx == 1 {
                        "ephemeral_token".into()
                    } else {
                        "password".into()
                    };
                    if self.connection_draft.auth_kind == "ephemeral_token" {
                        ui.colored_label(
                            self.theme.warning,
                            "Token is session-only — never stored as a long-lived password",
                        );
                    }
                });
                ui.add_space(SPACE_XS);
                Input::new(
                    &mut self.connection_draft.cloud_snippet,
                    "postgresql://user:secret@host:5432/db?sslmode=verify-full",
                    self.theme,
                )
                .label("Paste Connection URI")
                .show(ui);
                ui.add_space(SPACE_XXS);
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .text("Import from URI")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        let snippet = self.connection_draft.cloud_snippet.clone();
                        match apply_connection_snippet(&mut self.connection_draft, &snippet) {
                            Ok(()) => self.connection_error.clear(),
                            Err(err) => self.connection_error = err,
                        }
                    }
                });
                if !self.connection_draft.cloud_guidance.is_empty() {
                    ui.add_space(SPACE_XXS);
                    ui.label(
                        RichText::new(&self.connection_draft.cloud_guidance)
                            .small()
                            .color(self.theme.text_secondary),
                    );
                }
            });
        });
    }

    /// Panel 2: SSL / TLS Custom Certificates
    pub(crate) fn draw_ssl_certificates_panel(&mut self, ui: &mut egui::Ui) {
        if !matches!(
            self.connection_draft.ssl_mode,
            UiSslMode::VerifyCa | UiSslMode::VerifyFull
        ) {
            return;
        }

        let gap = SPACE_SM;
        ui.add_space(SPACE_SM);
        Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            rounding: Rounding::same(RADIUS_CARD),
            inner_margin: Margin::symmetric(12.0, 8.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.label(
                RichText::new("SSL Certificates")
                    .font(DbProTheme::ui_medium_font(11.5))
                    .color(self.theme.text_primary),
            );
            ui.add_space(SPACE_XS);
            Input::new(
                &mut self.connection_draft.ssl_root_cert_path,
                "/path/to/ca.pem",
                self.theme,
            )
            .label("Root CA Certificate Path")
            .leading_icon(Icon::FileCode)
            .show(ui);
            ui.add_space(SPACE_XS);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                let panel_half_w = calculate_half_row_width(ui.available_width(), gap);
                ui.vertical(|ui| {
                    ui.set_width(panel_half_w);
                    ui.set_max_width(panel_half_w);
                    Input::new(
                        &mut self.connection_draft.ssl_client_cert_path,
                        "/path/to/client.crt",
                        self.theme,
                    )
                    .label("Client Certificate (optional)")
                    .width(panel_half_w)
                    .show(ui);
                });
                ui.vertical(|ui| {
                    let w = ui.available_width();
                    ui.set_width(w);
                    ui.set_max_width(w);
                    Input::new(
                        &mut self.connection_draft.ssl_client_key_path,
                        "/path/to/client.key",
                        self.theme,
                    )
                    .label("Client Key (optional)")
                    .width(w)
                    .show(ui);
                });
            });
        });
    }

    /// Panel 3: SSH Bastion Tunnel
    pub(crate) fn draw_ssh_tunnel_panel(&mut self, ui: &mut egui::Ui) {
        let gap = SPACE_SM;
        ui.add_space(SPACE_SM);
        Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            rounding: Rounding::same(RADIUS_CARD),
            inner_margin: Margin::symmetric(12.0, 8.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(Icon::Shield).to_string())
                        .font(FontId::new(13.0, FontFamily::Name("lucide".into())))
                        .color(self.theme.accent),
                );
                ui.add_space(SPACE_XXS);
                ui.checkbox(
                    &mut self.connection_draft.ssh_tunnel_enabled,
                    RichText::new("Connect via SSH Bastion Tunnel")
                        .strong()
                        .color(self.theme.text_primary),
                );
            });

            ui.add_space(SPACE_XXS);
            ui.label(
                RichText::new(super::view::SSH_QUALIFICATION_HINT)
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );

            if self.connection_draft.ssh_tunnel_enabled {
                ui.add_space(SPACE_SM);

                let SshRowWidths {
                    host_w,
                    port_w,
                    user_w,
                    key_w,
                } = calculate_ssh_row_widths(ui.available_width(), gap);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;

                    ui.vertical(|ui| {
                        ui.set_width(host_w);
                        ui.set_max_width(host_w);
                        Input::new(&mut self.connection_draft.ssh_host, "bastion.example.com", self.theme)
                            .label("SSH Host")
                            .width(host_w)
                            .leading_icon(Icon::Server)
                            .show(ui);
                    });
                    ui.vertical(|ui| {
                        ui.set_width(port_w);
                        ui.set_max_width(port_w);
                        Input::new(&mut self.connection_draft.ssh_port, "22", self.theme)
                            .label("Port")
                            .width(port_w)
                            .leading_icon(Icon::Hash)
                            .show(ui);
                    });
                    ui.vertical(|ui| {
                        ui.set_width(user_w);
                        ui.set_max_width(user_w);
                        Input::new(&mut self.connection_draft.ssh_user, "ubuntu", self.theme)
                            .label("SSH User")
                            .width(user_w)
                            .leading_icon(Icon::User)
                            .show(ui);
                    });
                    ui.vertical(|ui| {
                        ui.set_width(key_w);
                        ui.set_max_width(key_w);
                        ui.label(
                            RichText::new("Private Key")
                                .size(12.0)
                                .strong()
                                .color(self.theme.text_secondary),
                        );
                        ui.add_space(SPACE_XXS);
                        ui.horizontal(|ui| {
                            let browse_w = 68.0;
                            let key_input_w = (key_w - browse_w - SPACE_XS).max(50.0);
                            ui.vertical(|ui| {
                                ui.set_width(key_input_w);
                                ui.set_max_width(key_input_w);
                                Input::new(&mut self.connection_draft.ssh_private_key, "~/.ssh/id_rsa", self.theme)
                                    .width(key_input_w)
                                    .leading_icon(Icon::Key)
                                    .show(ui);
                            });
                            ui.add_space(SPACE_XS);
                            if Button::new(self.theme)
                                .icon(Icon::FolderOpen)
                                .text("Browse")
                                .variant(ButtonVariant::Secondary)
                                .size(ButtonSize::Sm)
                                .show(ui)
                                .clicked()
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(UiCommand::PickSshPrivateKey { request_id });
                            }
                        });
                    });
                });
                ui.add_space(SPACE_XS);
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .text("Save as SSH profile")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.save_draft_as_ssh_profile();
                    }
                    if !self.ssh_profiles.is_empty() {
                        ui.label(RichText::new("Use profile:").small().color(self.theme.text_muted));
                        for profile in self.ssh_profiles.clone() {
                            let selected = self.connection_draft.ssh_profile_id == profile.id;
                            if ui.selectable_label(selected, &profile.name).clicked() {
                                self.apply_ssh_profile(&profile.id);
                            }
                        }
                    }
                });
                if !self.connection_draft.ssh_profile_id.is_empty() {
                    ui.label(
                        RichText::new(format!(
                            "Referenced SSH profile id `{}` (secrets stay in vault)",
                            self.connection_draft.ssh_profile_id
                        ))
                        .small()
                        .color(self.theme.text_muted),
                    );
                }
            }
        });
    }

    /// Panel 4: Tags & Metadata
    pub(crate) fn draw_tags_metadata_panel(
        &mut self,
        ui: &mut egui::Ui,
        id_salt: &'static str,
        placeholder: &'static str,
    ) {
        ui.add_space(SPACE_SM);
        Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            rounding: Rounding::same(RADIUS_CARD),
            inner_margin: Margin::symmetric(12.0, 8.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            egui::CollapsingHeader::new(
                RichText::new("Tags & Metadata")
                    .font(DbProTheme::ui_medium_font(11.5))
                    .color(self.theme.text_primary),
            )
            .id_salt(id_salt)
            .show(ui, |ui| {
                ui.add_space(SPACE_XS);
                Input::new(&mut self.connection_draft.tags, placeholder, self.theme)
                    .label("Tags (comma separated)")
                    .leading_icon(Icon::Tag)
                    .show(ui);
            });
        });
    }
}
