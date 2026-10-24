// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use super::config::*;
use super::layout::*;
use super::mapper::apply_connection_snippet;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::card::Card;
use crate::components::collapsible::Collapsible;
use crate::components::input::Input;
use crate::components::selection::Checkbox;
use crate::components::tabs::SegmentedTabs;
use crate::components::Label;
use crate::tokens::*;
use crate::{DbProTheme, UiCommand, UiSslMode};
use egui::{FontFamily, FontId, RichText};
use lucide_icons::Icon;

/// Wrap an optional panel in a `Card` with a `Collapsible` header whose open
/// state lives on `ConnectionDialogState` (persists while the dialog is open).
fn collapsing_panel<R>(
    ui: &mut egui::Ui,
    id: egui::Id,
    open: &mut bool,
    title: &str,
    theme: DbProTheme,
    body: impl FnOnce(&mut egui::Ui) -> R,
) {
    Card::new(theme).show(ui, |ui| {
        Collapsible::new(open, theme).id(id).title(title).show(ui, body);
    });
}

impl<'view, 'bridge> super::view::ConnectionDialogView<'view, 'bridge> {
    /// Panel 1: Cloud Presets & URI Importer
    pub(crate) fn draw_cloud_presets_panel(&mut self, ui: &mut egui::Ui) {
        // The open flag is split into a local so the body closure can keep
        // borrowing the rest of `self`.
        let mut open = self.dialog.cloud_presets_open;
        collapsing_panel(
            ui,
            ui.make_persistent_id("conn_cloud_presets_panel"),
            &mut open,
            t!("connection.cloud_presets_title").as_ref(),
            self.theme,
            |ui| {
                ui.add_space(SPACE_XS);
                ui.horizontal(|ui| {
                    let mut selected_idx = CLOUD_PRESET_OPTIONS
                        .iter()
                        .position(|option| option.key == self.dialog.draft.cloud_preset)
                        .map(|index| index + 1)
                        .unwrap_or(0);
                    let preset_labels: Vec<String> = std::iter::once("None (manual)".to_owned())
                        .chain(CLOUD_PRESET_OPTIONS.iter().map(|option| option.label.to_owned()))
                        .collect();
                    let selected_text = if self.dialog.draft.cloud_preset.is_empty() {
                        t!("connection.cloud_preset_placeholder").to_string()
                    } else {
                        CLOUD_PRESET_OPTIONS
                            .iter()
                            .find(|option| option.key == self.dialog.draft.cloud_preset)
                            .map(|option| option.label.to_owned())
                            .unwrap_or_else(|| self.dialog.draft.cloud_preset.clone())
                    };
                    let idx_before = selected_idx;
                    crate::components::Select::new("cloud_preset", &mut selected_idx, &preset_labels)
                        .theme(self.theme)
                        .size(crate::components::SelectSize::Sm)
                        .width(260.0)
                        .selected_text(selected_text)
                        .show(ui);
                    if selected_idx != idx_before && selected_idx < preset_labels.len() {
                        self.dialog.draft.cloud_preset = if selected_idx == 0 {
                            String::new()
                        } else {
                            CLOUD_PRESET_OPTIONS[selected_idx - 1].key.to_owned()
                        };
                    }
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
                            Ok(()) => self.dialog.clear_error(),
                            Err(err) => self.dialog.set_error(err),
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
        self.dialog.cloud_presets_open = open;
    }

    /// Panel 2: SSL / TLS Custom Certificates — only reachable for Verify modes.
    pub(crate) fn draw_ssl_certificates_panel(&mut self, ui: &mut egui::Ui) {
        if !matches!(self.dialog.draft.ssl_mode, UiSslMode::VerifyCa | UiSslMode::VerifyFull) {
            return;
        }

        let gap = SPACE_SM;
        ui.add_space(SPACE_SM);
        Card::new(self.theme).show(ui, |ui| {
            Label::new(t!("connection.ssl_certs_title"), self.theme).show(ui);
            ui.add_space(SPACE_XS);
            Input::new(&mut self.dialog.draft.ssl_root_cert_path, "/path/to/ca.pem", self.theme)
                .label(t!("connection.root_ca_path"))
                .leading_icon(Icon::FileCode)
                .show(ui);
            ui.add_space(SPACE_XS);
            let widths = row_widths(ui.available_width(), gap, &[1.0, 1.0]);
            form_row(
                ui,
                &widths,
                vec![
                    Box::new(|ui| {
                        Input::new(
                            &mut self.dialog.draft.ssl_client_cert_path,
                            "/path/to/client.crt",
                            self.theme,
                        )
                        .label(t!("connection.client_cert_path"))
                        .width(widths[0])
                        .show(ui);
                    }),
                    Box::new(|ui| {
                        Input::new(
                            &mut self.dialog.draft.ssl_client_key_path,
                            "/path/to/client.key",
                            self.theme,
                        )
                        .label(t!("connection.client_key_path"))
                        .width(widths[1])
                        .show(ui);
                    }),
                ],
                gap,
            );
        });
    }

    /// Panel 3: SSH Bastion Tunnel
    pub(crate) fn draw_ssh_tunnel_panel(&mut self, ui: &mut egui::Ui) {
        let gap = SPACE_SM;
        ui.add_space(SPACE_SM);
        Card::new(self.theme).show(ui, |ui| {
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

                // Host 38% / Port 14% / User 20% / Key 28%
                let widths = row_widths(ui.available_width(), gap, &[38.0, 14.0, 20.0, 28.0]);
                let key_w = widths[3];
                let show_ssh_host_error =
                    !self.dialog.error.is_empty() && self.dialog.draft.ssh_host.trim().is_empty();
                let show_ssh_port_error =
                    !self.dialog.error.is_empty() && !super::logic::is_valid_port(&self.dialog.draft.ssh_port);
                let ssh_port_error_key: &'static str = if self.dialog.draft.ssh_port.trim().is_empty() {
                    "connection.validation_ssh_port_required"
                } else {
                    "connection.validation_ssh_port_invalid"
                };
                let show_ssh_user_error =
                    !self.dialog.error.is_empty() && self.dialog.draft.ssh_user.trim().is_empty();
                let show_ssh_key_error = !self.dialog.error.is_empty()
                    && self.dialog.draft.ssh_private_key.trim().is_empty();
                let mut browse_key_clicked = false;

                form_row(
                    ui,
                    &widths,
                    vec![
                        Box::new(|ui| {
                            let mut ssh_host =
                                Input::new(&mut self.dialog.draft.ssh_host, "bastion.example.com", self.theme)
                                    .label(t!("connection.ssh_host"))
                                    .required(true)
                                    .width(widths[0])
                                    .leading_icon(Icon::Server);
                            if show_ssh_host_error {
                                ssh_host =
                                    ssh_host.error_text(t!("connection.validation_ssh_host_required"));
                            }
                            ssh_host.show(ui);
                        }),
                        Box::new(|ui| {
                            let mut ssh_port = Input::new(&mut self.dialog.draft.ssh_port, "22", self.theme)
                                .label(t!("connection.port"))
                                .required(true)
                                .width(widths[1])
                                .leading_icon(Icon::Hash);
                            if show_ssh_port_error {
                                ssh_port = ssh_port.error_text(t!(ssh_port_error_key));
                            }
                            ssh_port.show(ui);
                        }),
                        Box::new(|ui| {
                            let mut ssh_user =
                                Input::new(&mut self.dialog.draft.ssh_user, "ubuntu", self.theme)
                                    .label(t!("connection.ssh_user"))
                                    .required(true)
                                    .width(widths[2])
                                    .leading_icon(Icon::User);
                            if show_ssh_user_error {
                                ssh_user =
                                    ssh_user.error_text(t!("connection.validation_ssh_user_required"));
                            }
                            ssh_user.show(ui);
                        }),
                        Box::new(|ui| {
                            ssh_key_cell(
                                ui,
                                key_w,
                                show_ssh_key_error,
                                &mut browse_key_clicked,
                                &mut self.dialog.draft.ssh_private_key,
                                self.theme,
                            );
                        }),
                    ],
                    gap,
                );
                if browse_key_clicked {
                    let request_id = self.command_dispatcher.next_request_id();
                    self.dispatch_command(UiCommand::PickSshPrivateKey { request_id });
                }
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
                        let profiles = self.dialog.ssh_profiles.clone();
                        let profile_labels: Vec<String> =
                            profiles.iter().map(|profile| profile.name.clone()).collect();
                        let mut profile_idx = profiles
                            .iter()
                            .position(|profile| self.dialog.draft.ssh_profile_id == profile.id)
                            .unwrap_or(usize::MAX);
                        let idx_before = profile_idx;
                        crate::components::Select::new("ssh_profile_picker", &mut profile_idx, &profile_labels)
                            .theme(self.theme)
                            .size(crate::components::SelectSize::Sm)
                            .width(220.0)
                            .show(ui);
                        if profile_idx != idx_before && profile_idx < profiles.len() {
                            self.apply_ssh_profile(&profiles[profile_idx].id);
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
        let mut open = self.dialog.tags_panel_open;
        collapsing_panel(
            ui,
            ui.make_persistent_id(id_salt),
            &mut open,
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
        self.dialog.tags_panel_open = open;
    }
}

/// SSH private-key cell: label + path input + browse button, split inside the
/// column width the row already reserved. The browse click is returned via the
/// flag so the caller can dispatch after `form_row` ends (closures borrow
/// disjoint `draft` fields; dispatch needs the whole view).
fn ssh_key_cell(
    ui: &mut egui::Ui,
    key_w: f32,
    show_key_error: bool,
    browse_clicked: &mut bool,
    ssh_private_key: &mut String,
    theme: DbProTheme,
) {
    Label::new(t!("connection.ssh_private_key"), theme)
        .required(true)
        .show(ui);
    ui.add_space(SPACE_XXS);
    ui.horizontal(|ui| {
        let browse_w = 68.0;
        let key_input_w = (key_w - browse_w - SPACE_XS).max(50.0);
        let mut ssh_key = Input::new(ssh_private_key, "~/.ssh/id_rsa", theme)
            .width(key_input_w)
            .leading_icon(Icon::Key);
        if show_key_error {
            ssh_key = ssh_key.error_text(t!("connection.validation_ssh_key_required"));
        }
        ssh_key.show(ui);
        ui.add_space(SPACE_XS);
        *browse_clicked = Button::new(theme)
            .icon(Icon::FolderOpen)
            .text(t!("connection.browse"))
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked();
    });
}
