//! Native shell frame layout.
//!
//! The lifecycle adapter decides which feature surface to render. This module
//! owns the central-panel geometry and the shell spacing policy around it.
use super::*;

pub(super) struct ShellFrameContext {
    pub(super) theme: DbProTheme,
}

pub(super) fn draw_central_panel<F>(ctx: &egui::Context, context: &ShellFrameContext, draw_workspace: F)
where
    F: FnOnce(&mut egui::Ui),
{
    egui::CentralPanel::default()
        .frame(egui::Frame {
            // Flush to the sidebar splitter; match `SHELL_SPLIT_INSET` / sidebar
            // `pad_right` so the body lines up with the navigator across the divider.
            fill: context.theme.surface_panel,
            inner_margin: egui::Margin {
                left: SHELL_SPLIT_INSET,
                right: SHELL_SPLIT_INSET,
                top: 0.0,
                bottom: 0.0,
            },
            outer_margin: egui::Margin::ZERO,
            stroke: egui::Stroke::NONE,
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.set_min_size(ui.available_size());
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            draw_workspace(ui);
        });
}
