use super::config::*;
use super::layout::*;
use super::mapper::apply_connection_snippet;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::input::Input;
use crate::components::selection::Checkbox;
use crate::components::tabs::SegmentedTabs;
use crate::tokens::*;
use crate::{DbProTheme, UiCommand, UiSslMode};
use egui::{
    Align2, FontFamily, FontId, Frame, Margin, RichText, Rounding, Sense, Stroke, Vec2, WidgetInfo, WidgetType,
};
use lucide_icons::Icon;

impl<'view, 'bridge> super::view::ConnectionDialogView<'view, 'bridge> {
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
            show_click_only_collapsing_section(
                ui,
                "conn_cloud_presets_panel",
                t!("connection.cloud_presets_title").as_ref(),
                self.theme,
                |ui| {
                    ui.add_space(SPACE_XS);
                    ui.horizontal(|ui| {
                        let selected = if self.dialog.draft.cloud_preset.is_empty() {
                            "Select a cloud preset…".to_owned()
                        } else {
                            self.dialog.draft.cloud_preset.clone()
                        };
                        egui::ComboBox::from_id_salt("cloud_preset")
                            .selected_text(selected)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.dialog.draft.cloud_preset,
                                    String::new(),
                                    "None (manual)",
                                );
                                for option in CLOUD_PRESET_OPTIONS {
                                    ui.selectable_value(
                                        &mut self.dialog.draft.cloud_preset,
                                        option.key.to_owned(),
                                        option.label,
                                    );
                                }
                            });
                        if Button::new(self.theme)
                            .text(t!("connection.apply_preset"))
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
                        let mut auth_idx = if self.dialog.draft.auth_kind == "ephemeral_token" {
                            1
                        } else {
                            0
                        };
                        SegmentedTabs::new(&mut auth_idx, AUTH_KIND_OPTIONS, self.theme).show(ui);
                        self.dialog.draft.auth_kind = if auth_idx == 1 {
                            "ephemeral_token".into()
                        } else {
                            "password".into()
                        };
                        if self.dialog.draft.auth_kind == "ephemeral_token" {
                            ui.colored_label(self.theme.warning, t!("connection.token_session_warning"));
                        }
                    });
                    ui.add_space(SPACE_XS);
                    Input::new(
                        &mut self.dialog.draft.cloud_snippet,
                        "postgresql://user:secret@host:5432/db?sslmode=verify-full",
                        self.theme,
                    )
                    .label(t!("connection.paste_uri"))
                    .show(ui);
                    ui.add_space(SPACE_XXS);
                    ui.horizontal(|ui| {
                        if Button::new(self.theme)
                            .text(t!("connection.import_uri"))
                            .variant(ButtonVariant::Secondary)
                            .size(ButtonSize::Sm)
                            .show(ui)
                            .clicked()
                        {
                            let snippet = self.dialog.draft.cloud_snippet.clone();
                            match apply_connection_snippet(&mut self.dialog.draft, &snippet) {
                                Ok(()) => self.dialog.error.clear(),
                                Err(err) => self.dialog.error = err,
                            }
                        }
                    });
                    if !self.dialog.draft.cloud_guidance.is_empty() {
                        ui.add_space(SPACE_XXS);
                        ui.label(
                            RichText::new(&self.dialog.draft.cloud_guidance)
                                .small()
                                .color(self.theme.text_secondary),
                        );
                    }
                },
            );
        });
    }

    /// Panel 2: SSL / TLS Custom Certificates
    pub(crate) fn draw_ssl_certificates_panel(&mut self, ui: &mut egui::Ui) {
        if !matches!(self.dialog.draft.ssl_mode, UiSslMode::VerifyCa | UiSslMode::VerifyFull) {
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
                RichText::new(t!("connection.ssl_certs_title"))
                    .font(DbProTheme::ui_medium_font(11.5))
                    .color(self.theme.text_primary),
            );
            ui.add_space(SPACE_XS);
            Input::new(&mut self.dialog.draft.ssl_root_cert_path, "/path/to/ca.pem", self.theme)
                .label(t!("connection.root_ca_path"))
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
                        &mut self.dialog.draft.ssl_client_cert_path,
                        "/path/to/client.crt",
                        self.theme,
                    )
                    .label(t!("connection.client_cert_path"))
                    .width(panel_half_w)
                    .show(ui);
                });
                ui.vertical(|ui| {
                    let w = ui.available_width();
                    ui.set_width(w);
                    ui.set_max_width(w);
                    Input::new(
                        &mut self.dialog.draft.ssl_client_key_path,
                        "/path/to/client.key",
                        self.theme,
                    )
                    .label(t!("connection.client_key_path"))
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
                Checkbox::new(
                    &mut self.dialog.draft.ssh_tunnel_enabled,
                    t!("connection.ssh_bastion_title").as_ref(),
                    self.theme,
                )
                .focusable(false)
                .show(ui);
            });

            ui.add_space(SPACE_XXS);
            ui.label(
                RichText::new(super::view::SSH_QUALIFICATION_HINT)
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );

            if self.dialog.draft.ssh_tunnel_enabled {
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
                        Input::new(&mut self.dialog.draft.ssh_host, "bastion.example.com", self.theme)
                            .label(t!("connection.ssh_host"))
                            .width(host_w)
                            .leading_icon(Icon::Server)
                            .show(ui);
                    });
                    ui.vertical(|ui| {
                        ui.set_width(port_w);
                        ui.set_max_width(port_w);
                        Input::new(&mut self.dialog.draft.ssh_port, "22", self.theme)
                            .label(t!("connection.port"))
                            .width(port_w)
                            .leading_icon(Icon::Hash)
                            .show(ui);
                    });
                    ui.vertical(|ui| {
                        ui.set_width(user_w);
                        ui.set_max_width(user_w);
                        Input::new(&mut self.dialog.draft.ssh_user, "ubuntu", self.theme)
                            .label(t!("connection.ssh_user"))
                            .width(user_w)
                            .leading_icon(Icon::User)
                            .show(ui);
                    });
                    ui.vertical(|ui| {
                        ui.set_width(key_w);
                        ui.set_max_width(key_w);
                        ui.label(
                            RichText::new(t!("connection.ssh_private_key"))
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
                                Input::new(&mut self.dialog.draft.ssh_private_key, "~/.ssh/id_rsa", self.theme)
                                    .width(key_input_w)
                                    .leading_icon(Icon::Key)
                                    .show(ui);
                            });
                            ui.add_space(SPACE_XS);
                            if Button::new(self.theme)
                                .icon(Icon::FolderOpen)
                                .text(t!("connection.browse"))
                                .variant(ButtonVariant::Secondary)
                                .size(ButtonSize::Sm)
                                .show(ui)
                                .clicked()
                            {
                                let request_id = self.command_dispatcher.next_request_id();
                                self.dispatch_command(UiCommand::PickSshPrivateKey { request_id });
                            }
                        });
                    });
                });
                ui.add_space(SPACE_XS);
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .text(t!("connection.save_as_ssh_profile"))
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.save_draft_as_ssh_profile();
                    }
                    if !self.dialog.ssh_profiles.is_empty() {
                        ui.label(
                            RichText::new(t!("connection.use_profile"))
                                .small()
                                .color(self.theme.text_muted),
                        );
                        for profile in self.dialog.ssh_profiles.clone() {
                            let selected = self.dialog.draft.ssh_profile_id == profile.id;
                            if ui.selectable_label(selected, &profile.name).clicked() {
                                self.apply_ssh_profile(&profile.id);
                            }
                        }
                    }
                });
                if !self.dialog.draft.ssh_profile_id.is_empty() {
                    ui.label(
                        RichText::new(t!(
                            "status.referenced_ssh_profile",
                            id = self.dialog.draft.ssh_profile_id.as_str()
                        ))
                        .small()
                        .color(self.theme.text_muted),
                    );
                }
            }
        });
    }

    /// Panel 4: Tags & Metadata
    pub(crate) fn draw_tags_metadata_panel(&mut self, ui: &mut egui::Ui, id_salt: &'static str, placeholder: &str) {
        ui.add_space(SPACE_SM);
        Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            rounding: Rounding::same(RADIUS_CARD),
            inner_margin: Margin::symmetric(12.0, 8.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            show_click_only_collapsing_section(
                ui,
                id_salt,
                t!("connection.tags_metadata").as_ref(),
                self.theme,
                |ui| {
                    ui.add_space(SPACE_XS);
                    Input::new(&mut self.dialog.draft.tags, placeholder, self.theme)
                        .label(t!("connection.tags_label"))
                        .leading_icon(Icon::Tag)
                        .show(ui);
                },
            );
        });
    }
}

fn show_click_only_collapsing_section(
    ui: &mut egui::Ui,
    id_salt: &'static str,
    title: &str,
    theme: DbProTheme,
    add_body: impl FnOnce(&mut egui::Ui),
) {
    let id = ui.make_persistent_id(id_salt);
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
    let header_height = ui.spacing().interact_size.y;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), header_height),
        Sense {
            click: true,
            drag: false,
            focusable: false,
        },
    );
    if response.clicked() {
        state.toggle(ui);
    }
    response.widget_info(|| WidgetInfo::labeled(WidgetType::CollapsingHeader, true, title));

    let icon = if state.is_open() {
        Icon::ChevronDown
    } else {
        Icon::ChevronRight
    };
    ui.painter().text(
        rect.left_center() + egui::vec2(4.0, 0.0),
        Align2::LEFT_CENTER,
        char::from(icon).to_string(),
        FontId::new(13.0, FontFamily::Name("lucide".into())),
        theme.text_muted,
    );
    ui.painter().text(
        rect.left_center() + egui::vec2(22.0, 0.0),
        Align2::LEFT_CENTER,
        title,
        DbProTheme::ui_medium_font(11.5),
        theme.text_primary,
    );

    state.show_body_indented(&response, ui, add_body);
    state.store(ui.ctx());
}
