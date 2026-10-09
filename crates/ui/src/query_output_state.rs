//! Feature-owned selection state for query result/output tabs.

use super::OutputTab;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct QueryOutputState {
    pub(super) active_tab: OutputTab,
    pub(super) tabs_by_document: HashMap<String, OutputTab>,
}

impl Default for QueryOutputState {
    fn default() -> Self {
        Self {
            active_tab: OutputTab::Results,
            tabs_by_document: HashMap::new(),
        }
    }
}

impl QueryOutputState {
    pub(super) fn tab_for_document(&self, document_id: &str) -> OutputTab {
        self.tabs_by_document
            .get(document_id)
            .copied()
            .unwrap_or(OutputTab::Results)
    }

    pub(crate) fn active_tab_for_document(&self, document_id: Option<&str>) -> OutputTab {
        // `None` must read back the field `set_active_for_optional_document`
        // writes, otherwise clicking a tab with no open document is a no-op.
        document_id
            .map(|document_id| self.tab_for_document(document_id))
            .unwrap_or(self.active_tab)
    }

    pub(super) fn set_active_for_document(&mut self, document_id: &str, tab: OutputTab) {
        self.active_tab = tab;
        self.tabs_by_document.insert(document_id.to_owned(), tab);
    }

    pub(crate) fn set_active_for_optional_document(&mut self, document_id: Option<&str>, tab: OutputTab) {
        if let Some(document_id) = document_id {
            self.set_active_for_document(document_id, tab);
        } else {
            self.active_tab = tab;
        }
    }

    pub(super) fn set_for_document(&mut self, document_id: &str, tab: OutputTab) {
        self.tabs_by_document.insert(document_id.to_owned(), tab);
    }

    pub(crate) fn set_for_document_and_activate_if_active(
        &mut self,
        document_id: &str,
        active_document_id: Option<&str>,
        tab: OutputTab,
    ) {
        self.set_for_document(document_id, tab);
        if active_document_id == Some(document_id) {
            self.active_tab = tab;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbProTheme;

    #[test]
    fn output_state_defaults_to_results_without_document_overrides() {
        let state = QueryOutputState::default();

        assert_eq!(state.active_tab, OutputTab::Results);
        assert!(state.tabs_by_document.is_empty());
    }

    #[test]
    fn document_tab_overrides_are_owned_by_output_state() {
        let mut state = QueryOutputState::default();

        state.set_active_for_document("query-1", OutputTab::Explain);
        assert_eq!(state.tab_for_document("query-1"), OutputTab::Explain);
        assert_eq!(state.active_tab, OutputTab::Explain);

        state.set_for_document("query-2", OutputTab::Messages);
        assert_eq!(state.tab_for_document("query-2"), OutputTab::Messages);
        assert_eq!(state.active_tab, OutputTab::Explain);
    }

    #[test]
    fn tab_selection_without_document_still_switches_the_visible_pane() {
        let mut state = QueryOutputState::default();

        state.set_active_for_optional_document(None, OutputTab::History);

        assert_eq!(state.active_tab_for_document(None), OutputTab::History);
    }

    #[test]
    // cc-scan:allow LONG_FUNCTION — egui click harness needs setup + drive + assert
    fn clicking_a_dock_tab_switches_the_output_pane() {
        use crate::app::{DbProApp, WorkspaceTab};
        use crate::query::QueryDocument;
        use crate::TaskBridge;

        let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
        let mut app = DbProApp::with_task_bridge(bridge);
        app.workspace.active_tab = WorkspaceTab::Query;
        app.workspace.bottom_panel_open = true;
        app.query
            .session
            .add_document(QueryDocument::new("query-1", "Query 1", ""));
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        app.theme.apply(&ctx);
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0));
        let frame = |app: &mut DbProApp, events: Vec<egui::Event>| {
            crate::test_frame::frame(&ctx, egui::RawInput {
                screen_rect: Some(screen),
                events,
                ..Default::default()
            }, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.set_min_size(screen.size());
                    app.draw_query(ui);
                });
            })
        };

        let output = frame(&mut app, Vec::new());
        let tab_pos = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.text() == "History" => Some(text.pos),
                _ => None,
            })
            .expect("History tab label must render in the dock");
        let click = egui::pos2(tab_pos.x + 12.0, tab_pos.y + 8.0);

        frame(&mut app, vec![egui::Event::PointerMoved(click)]);
        frame(
            &mut app,
            vec![egui::Event::PointerButton {
                pos: click,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        frame(
            &mut app,
            vec![egui::Event::PointerButton {
                pos: click,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );

        let document_id = app.query.session.active_document().map(|document| document.id.as_str());
        assert_eq!(
            app.query.output.active_tab_for_document(document_id),
            OutputTab::History,
            "clicking the History dock tab must switch the pane"
        );
    }
}
