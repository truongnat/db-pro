use super::*;

impl DbProApp {
    pub(crate) fn open_edit_connection(&mut self, connection: &UiConnectionSummary) {
        self.pending_connection_request = None;
        self.editing_connection_id = Some(connection.id.clone());
        self.connection_draft = UiConnectionDraft {
            name: connection.name.clone(),
            host: connection.host.clone(),
            port: connection.port.to_string(),
            database: connection.database.clone(),
            username: connection.username.clone(),
            password: String::new(),
            driver: if connection.driver == "SQLite" {
                UiDriver::Sqlite
            } else {
                UiDriver::Postgres
            },
            ssl_mode: UiSslMode::Disable,
            readonly: connection.readonly,
            ssh_tunnel_enabled: false,
            ssh_host: String::new(),
            ssh_port: "22".to_owned(),
            ssh_user: String::new(),
            ssh_private_key: String::new(),
        };
        self.connection_error = "Enter the password again to save changes".to_owned();
        self.connection_test_valid = false;
        self.connection_test_draft = None;
        self.connection_dialog_open = true;
    }

    pub(super) fn draw_delete_confirmation(&mut self, ctx: &egui::Context) {
        let Some(connection_id) = self.delete_confirmation_id.clone() else {
            return;
        };
        let name = self
            .connections
            .iter()
            .find(|connection| connection.id == connection_id)
            .map(|connection| connection.name.clone())
            .unwrap_or_else(|| "this connection".to_owned());
        egui::Window::new("Delete connection")
            .collapsible(false)
            .resizable(false)
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.label(format!("Delete {name} and its saved credentials?"));
                ui.add_space(10.0);
                ui.colored_label(self.theme.warning, "This action cannot be undone.");
                ui.horizontal(|ui| {
                    if danger_button(ui, "Delete", self.theme).clicked() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::DeleteConnection {
                            request_id,
                            connection_id: connection_id.clone(),
                        });
                        self.pending_connection_request = Some(request_id);
                        self.runtime_message = format!("Deleting {name}…");
                        self.delete_confirmation_id = None;
                    }
                    if compact_button(ui, "Cancel", self.theme).clicked() {
                        self.delete_confirmation_id = None;
                    }
                });
            });
    }

    pub(super) fn draw_folder_delete_confirmation(&mut self, ctx: &egui::Context) {
        let Some(folder_id) = self.folder_delete_confirmation.clone() else {
            return;
        };
        let folder_name = self
            .query_folders
            .iter()
            .find(|folder| folder.id == folder_id)
            .map(|folder| folder.name.clone())
            .unwrap_or_else(|| "this folder".to_owned());
        let mut open = true;
        egui::Window::new("Delete query folder")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.label(format!("Delete {folder_name} and its saved-query links?"));
                ui.add_space(10.0);
                ui.colored_label(self.theme.warning, "Saved queries in this folder will become unfiled.");
                ui.horizontal(|ui| {
                    if danger_button(ui, "Delete folder", self.theme).clicked() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::DeleteQueryFolder {
                            request_id,
                            id: folder_id.clone(),
                        });
                        self.folder_delete_confirmation = None;
                    }
                    if compact_button(ui, "Cancel", self.theme).clicked() {
                        self.folder_delete_confirmation = None;
                    }
                });
            });
        if !open {
            self.folder_delete_confirmation = None;
        }
    }

    pub(super) fn draw_connection_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.connection_dialog_open;
        let draft_before = self.connection_draft.clone();
        let title = if self.editing_connection_id.is_some() {
            "Edit connection"
        } else {
            "New connection"
        };
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(520.0)
            .min_width(420.0)
            .max_width(640.0)
            .max_height(540.0)
            .scroll([false, true])
            .show(ctx, |ui| self.draw_connection_form(ui));
        self.connection_dialog_open = open && self.connection_dialog_open;
        if self.connection_draft != draft_before {
            self.connection_test_valid = false;
            self.connection_error.clear();
            self.runtime_message = "Connection changed · test again before saving".to_owned();
        }
        if !self.connection_dialog_open {
            self.pending_connection_request = None;
        }
    }

    fn draw_connection_form(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(if self.editing_connection_id.is_some() {
                "Update a safe database connection"
            } else {
                "Create a safe database connection"
            })
            .color(self.theme.text_secondary),
        );
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("Driver");
            ui.selectable_value(&mut self.connection_draft.driver, UiDriver::Postgres, "PostgreSQL");
            ui.selectable_value(&mut self.connection_draft.driver, UiDriver::Sqlite, "SQLite");
        });
        ui.add_space(6.0);
        Self::form_row(ui, "Name", &mut self.connection_draft.name, "Production DB", self.theme);
        if self.connection_draft.driver == UiDriver::Postgres {
            self.draw_postgres_connection_fields(ui);
        } else {
            self.draw_sqlite_connection_fields(ui);
        }
        self.draw_connection_form_footer(ui);
    }

    fn draw_postgres_connection_fields(&mut self, ui: &mut egui::Ui) {
        Self::form_row(ui, "Host", &mut self.connection_draft.host, "localhost", self.theme);
        Self::form_row(ui, "Port", &mut self.connection_draft.port, "5432", self.theme);
        Self::form_row(ui, "Database", &mut self.connection_draft.database, "app", self.theme);
        Self::form_row(
            ui,
            "Username",
            &mut self.connection_draft.username,
            "postgres",
            self.theme,
        );
        Self::password_form_row(ui, &mut self.connection_draft.password, self.theme);
        ui.horizontal_wrapped(|ui| {
            ui.label("SSL");
            for (mode, label) in [
                (UiSslMode::Disable, "Disable"),
                (UiSslMode::Require, "Require"),
                (UiSslMode::VerifyFull, "Verify full"),
            ] {
                ui.selectable_value(&mut self.connection_draft.ssl_mode, mode, label);
            }
        });
        egui::CollapsingHeader::new("SSH tunnel").show(ui, |ui| {
            ui.checkbox(&mut self.connection_draft.ssh_tunnel_enabled, "Use SSH tunnel");
            if self.connection_draft.ssh_tunnel_enabled {
                Self::form_row(
                    ui,
                    "SSH host",
                    &mut self.connection_draft.ssh_host,
                    "bastion.example.com",
                    self.theme,
                );
                Self::form_row(ui, "SSH port", &mut self.connection_draft.ssh_port, "22", self.theme);
                Self::form_row(
                    ui,
                    "SSH user",
                    &mut self.connection_draft.ssh_user,
                    "ubuntu",
                    self.theme,
                );
                ui.horizontal(|ui| {
                    ui.add_sized([96.0, 24.0], egui::Label::new("Private key"));
                    let input_width = (ui.available_width() - 82.0).clamp(160.0, 360.0);
                    input(
                        ui,
                        &mut self.connection_draft.ssh_private_key,
                        "/home/me/.ssh/id_ed25519",
                        input_width,
                        self.theme,
                    );
                    if compact_button(ui, "Browse…", self.theme).clicked() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::PickSshPrivateKey { request_id });
                    }
                });
            }
        });
    }

    fn draw_sqlite_connection_fields(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_sized([96.0, 24.0], egui::Label::new("SQLite file"));
            let input_width = (ui.available_width() - 82.0).clamp(160.0, 360.0);
            input(
                ui,
                &mut self.connection_draft.database,
                "/path/to/db.sqlite",
                input_width,
                self.theme,
            );
            if compact_button(ui, "Browse…", self.theme).clicked() {
                let request_id = self.task_bridge.next_request_id();
                let _ = self.task_bridge.send(UiCommand::PickSqliteFile { request_id });
            }
        });
    }

    fn draw_connection_form_footer(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.connection_draft.readonly, "Read-only connection");
        if !self.connection_error.is_empty() {
            ui.colored_label(self.theme.danger, self.connection_error.as_str());
        }
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            if compact_button(ui, "Test connection", self.theme).clicked() {
                self.dispatch_connection_command(false);
            }
            if self.connection_test_valid {
                ui.label(icon_text(Icon::CircleCheck, "Tested", self.theme.success));
            }
            if primary_button(ui, "Save connection", self.theme).clicked() {
                self.dispatch_connection_command(true);
            }
            if ghost_button(ui, "Cancel", self.theme).clicked() {
                self.connection_dialog_open = false;
            }
        });
    }
    fn form_row(ui: &mut egui::Ui, label: &str, value: &mut String, hint: &str, theme: DbProTheme) {
        ui.horizontal(|ui| {
            ui.add_sized([96.0, 24.0], egui::Label::new(label));
            let input_width = ui.available_width().clamp(160.0, 360.0);
            input(ui, value, hint, input_width, theme);
        });
    }

    fn password_form_row(ui: &mut egui::Ui, value: &mut String, theme: DbProTheme) {
        ui.horizontal(|ui| {
            ui.add_sized([96.0, 24.0], egui::Label::new("Password"));
            let input_width = ui.available_width().clamp(160.0, 360.0);
            password_input(ui, value, "Password", input_width, theme);
        });
    }

    pub(crate) fn dispatch_connection_command(&mut self, save: bool) {
        if self.connection_draft.name.trim().is_empty() || self.connection_draft.database.trim().is_empty() {
            self.connection_error = "Name and database are required".to_owned();
            return;
        }
        if save && self.connection_draft.driver == UiDriver::Postgres && self.connection_draft.password.is_empty() {
            self.connection_error = "Password is required for PostgreSQL".to_owned();
            return;
        }
        if self.connection_draft.driver == UiDriver::Postgres && self.connection_draft.port.parse::<u16>().is_err() {
            self.connection_error = "Port must be a number between 1 and 65535".to_owned();
            return;
        }
        if self.connection_draft.ssh_tunnel_enabled
            && (self.connection_draft.ssh_host.trim().is_empty()
                || self.connection_draft.ssh_user.trim().is_empty()
                || self.connection_draft.ssh_private_key.trim().is_empty())
        {
            self.connection_error = "SSH host, user and private key are required".to_owned();
            return;
        }
        if self.connection_draft.ssh_tunnel_enabled && self.connection_draft.ssh_port.parse::<u16>().is_err() {
            self.connection_error = "SSH port must be a number between 1 and 65535".to_owned();
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        let draft = self.connection_draft.clone();
        let command = if save {
            if let Some(connection_id) = self.editing_connection_id.clone() {
                UiCommand::UpdateConnection {
                    request_id,
                    connection_id,
                    draft,
                }
            } else {
                UiCommand::CreateConnection { request_id, draft }
            }
        } else {
            UiCommand::TestConnection { request_id, draft }
        };
        let _ = self.task_bridge.send(command);
        self.pending_connection_request = Some(request_id);
        if save {
            self.connection_test_valid = false;
        } else {
            self.connection_test_valid = false;
            self.connection_test_draft = Some(self.connection_draft.clone());
        }
        self.connection_error.clear();
        self.runtime_message = if save {
            "Saving connection…"
        } else {
            "Testing connection…"
        }
        .to_owned();
    }
}
