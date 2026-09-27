use crate::tokens::*;
use crate::DbProTheme;
use egui::{Align2, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};

use super::config::{
    PROGRESS_RING_INSET, PROGRESS_RING_TRACK_WIDTH, TERMINAL_DOT_RADIUS, TERMINAL_HEADER_HEIGHT,
    TERMINAL_TITLE_LEFT_INSET, TERMINAL_TITLE_RIGHT_INSET,
};
use super::handler::{
    clamp_progress, mac_dot_colors, mac_dot_positions, normalize_ring_radius, progress_ring_arc_points, terminal_title,
};

/// Terminal-like surface for displaying command outputs, script execution results, or logs.
pub struct TerminalBlock<'a> {
    output: &'a str,
    title: Option<&'a str>,
    theme: DbProTheme,
}

impl<'a> TerminalBlock<'a> {
    pub fn new(output: &'a str, theme: DbProTheme) -> Self {
        Self {
            output,
            title: None,
            theme,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let frame = egui::Frame::none()
            .fill(self.theme.surface_editor)
            .stroke(Stroke::new(STROKE_THIN, self.theme.border_default))
            .rounding(Rounding::same(RADIUS_CARD))
            .inner_margin(egui::Margin::same(0.0));

        frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // Terminal top bar
                let (header_rect, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), TERMINAL_HEADER_HEIGHT), Sense::hover());
                ui.painter().rect_filled(
                    header_rect,
                    Rounding {
                        nw: RADIUS_CARD,
                        ne: RADIUS_CARD,
                        sw: 0.0,
                        se: 0.0,
                    },
                    self.theme.surface_panel,
                );
                ui.painter().hline(
                    header_rect.x_range(),
                    header_rect.bottom(),
                    Stroke::new(STROKE_THIN, self.theme.border_subtle),
                );

                // 3 macOS window control dots
                let dot_positions = mac_dot_positions(header_rect);
                let dot_colors = mac_dot_colors(&self.theme);
                for (pos, color) in dot_positions.into_iter().zip(dot_colors) {
                    ui.painter().circle_filled(pos, TERMINAL_DOT_RADIUS, color);
                }

                // The title is clipped to a region that avoids overlapping the decorative control dots.
                let title_left = (header_rect.left() + TERMINAL_TITLE_LEFT_INSET)
                    .min((header_rect.right() - TERMINAL_TITLE_RIGHT_INSET).max(header_rect.left()));
                let title_right = (header_rect.right() - TERMINAL_TITLE_RIGHT_INSET).max(title_left);
                let title_rect = egui::Rect::from_min_max(
                    egui::pos2(title_left, header_rect.top()),
                    egui::pos2(title_right, header_rect.bottom()),
                );
                ui.painter().with_clip_rect(title_rect).text(
                    title_rect.center(),
                    Align2::CENTER_CENTER,
                    terminal_title(self.title),
                    font_caption(),
                    self.theme.text_secondary,
                );

                // Long commands remain discoverable through horizontal scrolling in narrow panels.
                ui.add_space(SPACE_SM);
                egui::ScrollArea::horizontal()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(SPACE_MD);
                            ui.label(
                                RichText::new(self.output)
                                    .size(FONT_SIZE_MONO_SM)
                                    .monospace()
                                    .color(self.theme.text_primary),
                            );
                            ui.add_space(SPACE_MD);
                        });
                    });
                ui.add_space(SPACE_SM);
            })
            .response
    }
}

/// Circular progress indicator showing percentage completion with smooth arc rendering.
pub struct ProgressRing {
    progress: f32, // 0.0 to 1.0
    radius: f32,
    theme: DbProTheme,
}

impl ProgressRing {
    pub fn new(progress: f32, radius: f32, theme: DbProTheme) -> Self {
        Self {
            progress: clamp_progress(progress),
            radius: normalize_ring_radius(radius),
            theme,
        }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let size = Vec2::splat(self.radius * 2.0);
        let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
        let center = rect.center();

        // Background track circle
        let track_radius = (self.radius - PROGRESS_RING_INSET).max(0.0);
        ui.painter().circle_stroke(
            center,
            track_radius,
            Stroke::new(PROGRESS_RING_TRACK_WIDTH, self.theme.surface_hover),
        );

        // Progress arc
        let arc_points = progress_ring_arc_points(center, self.radius, self.progress);
        if arc_points.len() >= 2 {
            ui.painter().add(egui::Shape::line(
                arc_points,
                Stroke::new(PROGRESS_RING_TRACK_WIDTH, self.theme.accent),
            ));
        }

        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_ui(mut on_ui: impl FnMut(&mut egui::Ui)) {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| on_ui(ui));
        });
    }

    #[test]
    fn terminal_block_renders_without_panicking() {
        let theme = DbProTheme::light();
        run_ui(|ui| {
            let resp = TerminalBlock::new("SELECT 1;", theme).title("Query Output").show(ui);
            assert!(resp.rect.is_finite());
            assert!(resp.rect.width() > 0.0);
        });
    }

    #[test]
    fn progress_ring_renders_without_panicking() {
        let theme = DbProTheme::light();
        run_ui(|ui| {
            let resp = ProgressRing::new(0.65, 18.0, theme).show(ui);
            assert!(resp.rect.is_finite());
            assert_eq!(resp.rect.width(), 36.0);
            assert_eq!(resp.rect.height(), 36.0);
        });
    }

    #[test]
    fn invalid_progress_ring_geometry_remains_finite() {
        let theme = DbProTheme::light();
        run_ui(|ui| {
            let resp = ProgressRing::new(f32::NAN, f32::NEG_INFINITY, theme).show(ui);
            assert!(resp.rect.is_finite());
            assert_eq!(resp.rect.width(), 36.0);
            assert_eq!(resp.rect.height(), 36.0);
        });
    }
}
