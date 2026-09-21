//! Foreign-data-wrapper presentation and typed user intents.
use super::super::fdw_state::FdwState;
use super::super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FdwSurfaceAction {
    Refresh,
    PreviewDropServer(String),
    RequestDropServer(String),
    PreviewCreate {
        name: String,
        wrapper: String,
        host: String,
        dbname: String,
        port: String,
    },
    Create,
    ClosePreview,
    ConfirmDropServer {
        name: String,
        cascade: bool,
    },
    CancelDrop,
}

pub(super) struct FdwSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut FdwState,
}

impl FdwSurfaceContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<FdwSurfaceAction> {
        let mut actions = Vec::new();
        self.draw_header(ui, &mut actions);
        self.draw_inventory(ui, &mut actions);
        self.draw_create_form(ui, &mut actions);
        self.draw_preview(ui, &mut actions);
        self.draw_drop_confirmation(ui, &mut actions);
        actions
    }

    fn draw_header(&self, ui: &mut egui::Ui, actions: &mut Vec<FdwSurfaceAction>) {
        ui.add_space(SPACE_MD);
        section_label(ui, "FOREIGN DATA (FDW)", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("PostgreSQL-only · passwords/options redacted · CREATE EXTENSION never auto-run")
                .small()
                .color(self.theme.text_muted),
        );
        if secondary_button_with_icon(ui, Icon::RefreshCw, "Load FDW inventory", self.theme).clicked() {
            actions.push(FdwSurfaceAction::Refresh);
        }
        if let Some(error) = &self.state.fdw_error {
            ui.colored_label(self.theme.danger, error);
        }
    }

    fn draw_inventory(&self, ui: &mut egui::Ui, actions: &mut Vec<FdwSurfaceAction>) {
        let Some(inventory) = self.state.fdw_inventory.as_ref() else {
            return;
        };
        ui.label(
            RichText::new(&inventory.message)
                .small()
                .color(self.theme.text_secondary),
        );
        if let Some(hint) = &inventory.extension_hint {
            ui.colored_label(self.theme.warning, hint);
        }
        for wrapper in inventory.wrappers.iter().take(20) {
            ui.label(
                RichText::new(format!("wrapper {} · handler={:?}", wrapper.name, wrapper.handler))
                    .small()
                    .monospace(),
            );
        }
        for server in inventory.servers.iter().take(30) {
            self.draw_server(ui, server, actions);
        }
        for mapping in inventory.user_mappings.iter().take(30) {
            let options = mapping
                .options
                .iter()
                .map(|option| format!("{}={}", option.key, option.display_value()))
                .collect::<Vec<_>>()
                .join(", ");
            ui.label(
                RichText::new(format!(
                    "mapping {}@{} · {}",
                    mapping.user_name, mapping.server_name, options
                ))
                .small()
                .monospace()
                .color(self.theme.text_secondary),
            );
        }
        for table in inventory.foreign_tables.iter().take(40) {
            ui.label(
                RichText::new(format!(
                    "foreign {}.{} → {}",
                    table.schema, table.name, table.server_name
                ))
                .small()
                .monospace(),
            );
        }
    }

    fn draw_server(
        &self,
        ui: &mut egui::Ui,
        server: &db_pro_core::domain::fdw::ForeignServer,
        actions: &mut Vec<FdwSurfaceAction>,
    ) {
        let server_name = server.name.clone();
        let options = server
            .options
            .iter()
            .map(|option| format!("{}={}", option.key, option.display_value()))
            .collect::<Vec<_>>()
            .join(", ");
        card_frame(self.theme).show(ui, |ui| {
            ui.label(
                RichText::new(format!("server {} · fdw={}", server.name, server.fdw_name))
                    .strong()
                    .monospace(),
            );
            if !options.is_empty() {
                ui.label(RichText::new(options).small().color(self.theme.text_muted));
            }
            ui.horizontal(|ui| {
                if ghost_button_with_icon(ui, Icon::FileCode2, "Preview DROP", self.theme).clicked() {
                    actions.push(FdwSurfaceAction::PreviewDropServer(server_name.clone()));
                }
                if danger_button(ui, "Drop…", self.theme).clicked() {
                    actions.push(FdwSurfaceAction::RequestDropServer(server_name.clone()));
                }
            });
        });
        ui.add_space(SPACE_XS);
    }

    fn draw_create_form(&mut self, ui: &mut egui::Ui, actions: &mut Vec<FdwSurfaceAction>) {
        ui.add_space(SPACE_SM);
        ui.label(RichText::new("Create foreign server").small().strong());
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.state.fdw_create_name).hint_text("server name"));
            ui.add(egui::TextEdit::singleline(&mut self.state.fdw_create_wrapper).hint_text("fdw"));
        });
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.state.fdw_create_host).hint_text("host"));
            ui.add(egui::TextEdit::singleline(&mut self.state.fdw_create_dbname).hint_text("dbname"));
            ui.add(egui::TextEdit::singleline(&mut self.state.fdw_create_port).hint_text("port"));
        });
        ui.horizontal(|ui| {
            if ghost_button_with_icon(ui, Icon::FileCode2, "Preview CREATE", self.theme).clicked() {
                actions.push(FdwSurfaceAction::PreviewCreate {
                    name: self.state.fdw_create_name.clone(),
                    wrapper: self.state.fdw_create_wrapper.clone(),
                    host: self.state.fdw_create_host.clone(),
                    dbname: self.state.fdw_create_dbname.clone(),
                    port: self.state.fdw_create_port.clone(),
                });
            }
            if secondary_button(ui, "Create (confirm)", self.theme).clicked() {
                actions.push(FdwSurfaceAction::Create);
            }
        });
    }

    fn draw_preview(&self, ui: &mut egui::Ui, actions: &mut Vec<FdwSurfaceAction>) {
        let Some(preview) = self.state.fdw_ddl_preview.as_ref() else {
            return;
        };
        egui::Window::new("FDW DDL preview")
            .collapsible(false)
            .resizable(true)
            .default_width(480.0)
            .show(ui.ctx(), |ui| {
                ui.label(RichText::new(preview).monospace());
                if secondary_button(ui, "Close", self.theme).clicked() {
                    actions.push(FdwSurfaceAction::ClosePreview);
                }
            });
    }

    fn draw_drop_confirmation(&self, ui: &mut egui::Ui, actions: &mut Vec<FdwSurfaceAction>) {
        let Some(name) = self.state.fdw_drop_confirm.as_ref() else {
            return;
        };
        egui::Window::new("Drop foreign server?")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label(format!(
                    "Drop server `{name}` CASCADE? This removes dependent foreign tables/mappings."
                ));
                ui.horizontal(|ui| {
                    if danger_button(ui, "Drop CASCADE", self.theme).clicked() {
                        actions.push(FdwSurfaceAction::ConfirmDropServer {
                            name: name.clone(),
                            cascade: true,
                        });
                    }
                    if secondary_button(ui, "Cancel", self.theme).clicked() {
                        actions.push(FdwSurfaceAction::CancelDrop);
                    }
                });
            });
    }
}
