//! Root adapter for the native workspace start page.
use super::*;

impl DbProApp {
    pub(super) fn draw_welcome(&mut self, ui: &mut egui::Ui) {
        let active_connection_id = self.connection.lifecycle.active_connection_id().map(str::to_owned);
        let actions = welcome_surface_view::WelcomeSurfaceContext {
            theme: self.theme,
            welcome: &self.welcome,
            catalog: &self.connection.catalog,
            active_connection_id: active_connection_id.as_deref(),
        }
        .draw(ui);
        self.apply_welcome_actions(actions);
    }

    fn apply_welcome_actions(&mut self, actions: Vec<welcome_surface_view::WelcomeAction>) {
        use welcome_surface_view::WelcomeAction;

        for action in actions {
            match action {
                WelcomeAction::NewConnection => self.connection.open_new(),
                WelcomeAction::NewQuery => {
                    self.new_query_document();
                    self.workspace.active_tab = WorkspaceTab::Query;
                }
                WelcomeAction::OpenPalette => self.palette.open(PaletteMode::Commands),
                WelcomeAction::OpenDraftQuery(draft) => {
                    self.set_active_query_text(draft);
                    self.workspace.active_tab = WorkspaceTab::Query;
                    self.feedback.runtime_message = "Opened draft in Query".to_owned();
                }
                WelcomeAction::Connect(connection_id) => {
                    if let Some(connection) = self.connection.catalog.find(&connection_id).cloned() {
                        self.connect_to_connection(&connection);
                    }
                }
            }
        }
    }
}
