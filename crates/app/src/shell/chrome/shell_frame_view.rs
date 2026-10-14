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
    let surface = if context.settings_mode { ShellSurface::App } else { ShellSurface::Panel };
    // Flush to the sidebar splitter; match `SHELL_SPLIT_INSET` / sidebar
    // `pad_right` so the body lines up with the navigator across the divider.
    let split_inset = if context.settings_mode { 0 } else { SHELL_SPLIT_INSET as i8 };
    egui::CentralPanel::default()
        .frame(shell_surface_frame(context.theme, surface, egui::Margin::symmetric(split_inset, 0), false))
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            draw_workspace(ui);
        });
}
