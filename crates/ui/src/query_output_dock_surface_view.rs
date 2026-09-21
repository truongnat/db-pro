//! Query output dock layout and tab chrome.
//!
//! Result panes still need the application adapter because they dispatch
//! query-specific actions. This context owns only dock geometry and output-tab
//! state, leaving pane rendering at the explicit root boundary.
use super::*;

const RESIZE_GRIP_HEIGHT: f32 = 4.0;

pub(super) struct QueryOutputDockContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) workspace: &'a mut WorkspaceFeatureState,
    pub(super) output: &'a mut QueryOutputState,
    pub(super) session: &'a QuerySessionState,
    pub(super) editor: &'a mut QueryEditorState,
}

impl QueryOutputDockContext<'_> {
    pub(super) fn draw_chrome(&mut self, ui: &mut egui::Ui, dock_height: f32) -> f32 {
        self.draw_resize_grip(ui);
        let body_height = Self::body_height(dock_height);
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), body_height),
            Layout::top_down(Align::Min),
            |ui| {
                let mut tabs_context = query_output_tabs_view::QueryOutputTabsContext {
                    theme: self.theme,
                    output: self.output,
                    session: self.session,
                    editor: self.editor,
                    bottom_panel_open: &mut self.workspace.bottom_panel_open,
                };
                query_output_tabs_view::draw_output_tabs(&mut tabs_context, ui, true);
            },
        );
        body_height
    }

    fn body_height(dock_height: f32) -> f32 {
        (dock_height - RESIZE_GRIP_HEIGHT).max(OUTPUT_MIN_HEIGHT - RESIZE_GRIP_HEIGHT)
    }

    fn draw_resize_grip(&mut self, ui: &mut egui::Ui) {
        let (grip_rect, grip_response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), RESIZE_GRIP_HEIGHT),
            egui::Sense::drag(),
        );
        ui.painter().rect_filled(
            grip_rect,
            0.0,
            if grip_response.hovered() || grip_response.dragged() {
                self.theme.border_strong
            } else {
                self.theme.border_subtle
            },
        );
        if grip_response.dragged() {
            let next_height = self.workspace.bottom_panel_height - grip_response.drag_delta().y;
            self.workspace.set_bottom_panel_height(next_height);
            self.editor.query_output_dock_maximized = false;
        }
        grip_response.on_hover_cursor(egui::CursorIcon::ResizeVertical);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dock_body_preserves_the_minimum_content_height() {
        assert_eq!(
            QueryOutputDockContext::body_height(0.0),
            OUTPUT_MIN_HEIGHT - RESIZE_GRIP_HEIGHT
        );
    }
}
