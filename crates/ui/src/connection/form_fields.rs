use super::config::*;
use super::layout::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::input::{Input, PasswordInput};
use crate::components::tabs::SegmentedTabs;
use crate::tokens::*;
use crate::{DbProApp, DbProTheme};
use egui::{FontFamily, FontId, RichText};
use lucide_icons::Icon;

impl DbProApp {
    /// Draw general connection profile fields (Name, Group, Favorite, Environment, Safety).
    pub(crate) fn draw_general_profile_fields(&mut self, ui: &mut egui::Ui) {
        let avail = ui.available_width();
        let gap = SPACE_SM;

        ui.label(
            RichText::new("GENERAL")
                .font(DbProTheme::ui_medium_font(10.5))
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XXS);

        // Row 1: Connection Name, Folder/Group, Favorite
        let ProfileRowWidths { name_w, group_w, fav_w } = calculate_profile_row_widths(avail, gap);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.vertical(|ui| {
                ui.set_width(name_w);
                ui.set_max_width(name_w);
                Input::new(
                    &mut self.connection_draft.name,
                    "e.g. Production PostgreSQL",
                    self.theme,
                )
                .label("Connection Name")
                .width(name_w)
                .leading_icon(Icon::Tag)
                .clearable(true)
                .show(ui);
            });
            ui.vertical(|ui| {
                ui.set_width(group_w);
                ui.set_max_width(group_w);
                Input::new(&mut self.connection_draft.group, "e.g. Acme / Local", self.theme)
                    .label("Folder / Group")
                    .width(group_w)
                    .leading_icon(Icon::Folder)
                    .clearable(true)
                    .show(ui);
            });
            ui.vertical(|ui| {
                ui.set_width(fav_w);
                ui.set_max_width(fav_w);
                ui.label(
                    RichText::new("Favorite")
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(3.0);
                let (fav_icon, fav_text) = if self.connection_draft.favorite {
                    (Icon::Star, "Saved")
                } else {
                    (Icon::Star, "Off")
                };
                if Button::new(self.theme)
                    .icon(fav_icon)
                    .text(fav_text)
                    .variant(if self.connection_draft.favorite {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Ghost
                    })
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.connection_draft.favorite = !self.connection_draft.favorite;
                }
            });
        });
        ui.add_space(SPACE_SM);

        // Row 2: Environment Selector & Read-only mode
        let EnvRowWidths { env_w, ro_w } = calculate_env_row_widths(avail, gap);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.vertical(|ui| {
                ui.set_width(env_w);
                ui.set_max_width(env_w);
                ui.label(
                    RichText::new("Environment")
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(SPACE_XXS);
                let mut env_idx = environment_to_index(&self.connection_draft.environment);
                SegmentedTabs::new(&mut env_idx, ENVIRONMENT_OPTIONS, self.theme).show(ui);
                self.connection_draft.environment = index_to_environment(env_idx).to_owned();
            });
            ui.vertical(|ui| {
                ui.set_width(ro_w);
                ui.set_max_width(ro_w);
                ui.label(
                    RichText::new("Safety Policy")
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(SPACE_XXS);
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.connection_draft.readonly, "Read-only mode");
                    if self.connection_draft.environment == "Production" {
                        ui.label(
                            RichText::new("⚠ Production guard")
                                .font(font_caption())
                                .color(self.theme.warning),
                        );
                    }
                });
            });
        });
        ui.add_space(SPACE_MD);
    }

    /// Draw server & credentials section for network-based databases (PostgreSQL, MySQL, SQL Server).
    pub(crate) fn draw_server_credentials_fields(&mut self, ui: &mut egui::Ui) {
        let avail = ui.available_width();
        let gap = SPACE_SM;

        ui.label(
            RichText::new("SERVER & CREDENTIALS")
                .font(DbProTheme::ui_medium_font(10.5))
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XXS);

        // Row 1: Host (68%) + Port (32%)
        let ServerRowWidths { host_w, port_w } = calculate_server_row_widths(avail, gap);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.vertical(|ui| {
                ui.set_width(host_w);
                ui.set_max_width(host_w);
                Input::new(&mut self.connection_draft.host, "localhost", self.theme)
                    .label("Host / Server Address")
                    .width(host_w)
                    .leading_icon(Icon::Server)
                    .show(ui);
            });
            ui.vertical(|ui| {
                ui.set_width(port_w);
                ui.set_max_width(port_w);
                let default_port = default_port_for_driver(self.connection_draft.driver);
                Input::new(&mut self.connection_draft.port, default_port, self.theme)
                    .label("Port")
                    .width(port_w)
                    .leading_icon(Icon::Hash)
                    .show(ui);
            });
        });
        ui.add_space(SPACE_SM);

        // Row 2: Database Name (50%) + Username (50%)
        let half_w = calculate_half_row_width(avail, gap);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.vertical(|ui| {
                ui.set_width(half_w);
                ui.set_max_width(half_w);
                let default_db = default_database_for_driver(self.connection_draft.driver);
                Input::new(&mut self.connection_draft.database, default_db, self.theme)
                    .label("Database Name")
                    .width(half_w)
                    .leading_icon(Icon::Database)
                    .clearable(true)
                    .show(ui);
            });
            ui.vertical(|ui| {
                ui.set_width(half_w);
                ui.set_max_width(half_w);
                let default_user = default_username_for_driver(self.connection_draft.driver);
                Input::new(&mut self.connection_draft.username, default_user, self.theme)
                    .label("Username")
                    .width(half_w)
                    .leading_icon(Icon::User)
                    .show(ui);
            });
        });
        ui.add_space(SPACE_SM);

        // Row 3: Password/Auth Token (50%) + SSL Mode (50%)
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.vertical(|ui| {
                ui.set_width(half_w);
                ui.set_max_width(half_w);
                let pwd_placeholder = if self.connection_draft.auth_kind == "ephemeral_token" {
                    "Paste short-lived IAM/access token (not stored)"
                } else if self.editing_connection_id.is_some() {
                    "•••••••• (Leave blank to keep saved password)"
                } else {
                    "Optional (Leave blank if no password)"
                };
                PasswordInput::new(
                    &mut self.connection_draft.password,
                    pwd_placeholder,
                    &mut self.connection_show_password,
                    self.theme,
                )
                .label(if self.connection_draft.auth_kind == "ephemeral_token" {
                    "Access Token"
                } else {
                    "Password"
                })
                .width(half_w)
                .show(ui);
            });
            ui.vertical(|ui| {
                ui.set_width(half_w);
                ui.set_max_width(half_w);
                ui.label(
                    RichText::new("SSL / TLS Mode")
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(SPACE_XXS);
                let mut ssl_idx = ssl_mode_to_index(self.connection_draft.ssl_mode);
                SegmentedTabs::new(&mut ssl_idx, SSL_MODE_OPTIONS, self.theme).show(ui);
                self.connection_draft.ssl_mode = index_to_ssl_mode(ssl_idx);
                ui.add_space(SPACE_XXS);
                let guidance_color = match self.connection_draft.ssl_mode {
                    crate::UiSslMode::Disable => self.theme.danger,
                    crate::UiSslMode::Require | crate::UiSslMode::VerifyCa => self.theme.warning,
                    crate::UiSslMode::VerifyFull => self.theme.text_muted,
                };
                ui.label(
                    RichText::new(super::view::ssl_mode_guidance(self.connection_draft.ssl_mode))
                        .size(10.0)
                        .color(guidance_color),
                );
            });
        });

        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(char::from(Icon::ShieldCheck).to_string())
                    .font(FontId::new(11.0, FontFamily::Name("lucide".into())))
                    .color(self.theme.accent),
            );
            ui.label(
                RichText::new("Credentials encrypted with AES-256-GCM in local vault.")
                    .size(11.0)
                    .color(self.theme.text_muted),
            );
        });

        ui.add_space(SPACE_MD);
    }
}
