//! Developer-oriented utility components.
//!
//! Implements TerminalBlock, ProgressRing, MonospaceValue, and StatusDot
//! per `open-ai-refer.md`.

use crate::tokens::*;
use crate::DbProTheme;
use egui::{Align2, Pos2, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};

// ── TerminalBlock Component ──────────────────────────────────────────────────

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
                let (header_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 28.0), Sense::hover());
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
                let dot_colors = [self.theme.danger, self.theme.warning, self.theme.success];
                for (i, c) in dot_colors.into_iter().enumerate() {
                    let dot_pos = Pos2::new(header_rect.left() + 12.0 + (i as f32 * 12.0), header_rect.center().y);
                    ui.painter().circle_filled(dot_pos, 3.5, c);
                }

                // Title
                let title_text = self.title.unwrap_or("Terminal Output");
                ui.painter().text(
                    Pos2::new(header_rect.center().x, header_rect.center().y),
                    Align2::CENTER_CENTER,
                    title_text,
                    font_caption(),
                    self.theme.text_secondary,
                );

                // Body
                ui.add_space(SPACE_SM);
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
                ui.add_space(SPACE_SM);
            })
            .response
    }
}

// ── ProgressRing Component ───────────────────────────────────────────────────

pub struct ProgressRing {
    progress: f32, // 0.0 to 1.0
    radius: f32,
    theme: DbProTheme,
}

impl ProgressRing {
    pub fn new(progress: f32, radius: f32, theme: DbProTheme) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            radius,
            theme,
        }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let size = Vec2::splat(self.radius * 2.0);
        let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
        let center = rect.center();

        // Background track circle
        ui.painter()
            .circle_stroke(center, self.radius - 2.0, Stroke::new(2.5, self.theme.surface_hover));

        // Progress arc
        let sweep_angle = self.progress * std::f32::consts::TAU;
        let points = 32;
        let arc_points: Vec<Pos2> = (0..=points)
            .map(|i| {
                let frac = i as f32 / points as f32;
                let angle = -std::f32::consts::FRAC_PI_2 + frac * sweep_angle;
                Pos2::new(
                    center.x + (self.radius - 2.0) * angle.cos(),
                    center.y + (self.radius - 2.0) * angle.sin(),
                )
            })
            .collect();

        if arc_points.len() >= 2 {
            ui.painter()
                .add(egui::Shape::line(arc_points, Stroke::new(2.5, self.theme.accent)));
        }

        resp
    }
}
