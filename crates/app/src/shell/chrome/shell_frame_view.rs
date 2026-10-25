//! Native shell frame layout.
//!
//! The lifecycle adapter decides which feature surface to render. This module
//! owns the central-panel geometry and the shell spacing policy around it.
use super::*;

pub(super) struct ShellFrameContext {
    pub(super) theme: DbProTheme,
    pub(super) settings_mode: bool,
}

pub(super) fn draw_central_panel<F>(ui: &mut egui::Ui, context: &ShellFrameContext, draw_workspace: F)
where
    F: FnOnce(&mut egui::Ui),
{
    // CentralPanel renders the active workspace (Welcome, Query, Table, Diagrams).
    // Use ShellSurface::App so the workspace canvas steps down from the chrome panels
    // (Sidebar, Statusbar, Output Dock) instead of reusing the exact same fill.
    let surface = ShellSurface::App;
    // Flush to the sidebar splitter; match `SHELL_SPLIT_INSET` / sidebar
    // `pad_right` so the body lines up with the navigator across the divider.
    let split_inset = if context.settings_mode { 0 } else { SHELL_SPLIT_INSET as i8 };
    egui::CentralPanel::default()
        .frame(shell_surface_frame(context.theme, surface, egui::Margin::symmetric(split_inset, 0), false))
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            // egui propagates `item_spacing` into every Frame/ScrollArea/child Ui
            // below, and only top-level panels reset it. Zeroing it here used to
            // make every workspace view that forgot an explicit `add_space`
            // render its labels, buttons and pickers fused; the theme's own row
            // rhythm (SPACE_SM × SPACE_XS) is the safe default instead. Views
            // that want tighter packing override locally.
            ui.spacing_mut().item_spacing =
                egui::vec2(crate::tokens::SPACE_SM, crate::tokens::SPACE_XS);
            draw_workspace(ui);
        });
}
