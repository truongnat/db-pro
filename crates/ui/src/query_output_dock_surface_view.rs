//! Query output dock layout and tab chrome.
//!
//! Result panes still need the application adapter because they dispatch
//! query-specific actions. This context owns only dock geometry and output-tab
//! state, leaving pane rendering at the explicit root boundary.
use super::*;

const RESIZE_GRIP_HEIGHT: f32 = 4.0;

/// Height reserved for the output tab strip. An unconstrained `right_to_left`
/// tab row expands to fill the dock's whole available height, so the strip must
/// be bounded here; the result pane then receives the remaining dock budget.
const DOCK_TAB_STRIP_HEIGHT: f32 = 34.0;

pub(super) struct QueryOutputDockContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) workspace: &'a mut WorkspaceFeatureState,
    pub(super) output: &'a mut QueryOutputState,
    pub(super) session: &'a QuerySessionState,
    pub(super) editor: &'a mut QueryEditorState,
}

impl QueryOutputDockContext<'_> {
    pub(super) fn draw_chrome(&mut self, ui: &mut egui::Ui, dock_height: f32) -> f32 {
        // The chrome (resize grip + tab strip) must consume only its own height so the
        // caller can allocate the remaining dock budget to the result pane. The tab row
        // is bounded to `DOCK_TAB_STRIP_HEIGHT` because an unconstrained `right_to_left`
        // row grows to fill the whole dock height, which previously pushed the pane below
        // the visible clip and left the grid blank.
        let chrome_top = ui.cursor().top();
        self.draw_resize_grip(ui);
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), DOCK_TAB_STRIP_HEIGHT),
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
        let chrome_height = ui.cursor().top() - chrome_top;
        Self::body_height(dock_height, chrome_height)
    }

    fn body_height(dock_height: f32, chrome_height: f32) -> f32 {
        (dock_height - chrome_height).max(0.0)
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
    fn dock_body_is_the_dock_height_minus_measured_chrome() {
        // Body fills whatever the chrome (grip + tab strip) leaves of the dock budget.
        assert_eq!(QueryOutputDockContext::body_height(180.0, 34.0), 146.0);
        // A chrome taller than the dock never yields a negative allocation.
        assert_eq!(QueryOutputDockContext::body_height(20.0, 34.0), 0.0);
    }
}
