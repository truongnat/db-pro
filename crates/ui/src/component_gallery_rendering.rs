// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use super::*;
use egui::{Color32, FontFamily, FontId, Pos2, Rect, RichText, Rounding, Stroke, Ui, Vec2};

impl DbProApp {
    pub(super) fn draw_gallery_rendering_section(&mut self, ui: &mut Ui) {
        self.draw_section_heading(
            ui,
            "Rendering diagnostics",
            "Inspect alpha composition, physical-pixel geometry, text, and overlays using the active native renderer.",
        );

        let ppp = ui.ctx().pixels_per_point();
        let native_ppp = ui.ctx().native_pixels_per_point().unwrap_or(1.0);
        ui.label(format!(
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
            "Backend: eframe Glow · egui 0.29.x · pixels_per_point {:.2} (native {:.2}, zoom {:.2}) · MSAA: off (eframe default)",
            ppp,
            native_ppp,
            ppp / native_ppp
        ));
        ui.label(
            RichText::new(
                // cc-scan:allow LINE_TOO_LONG — literal must not wrap
                "This screen cannot switch the Glow sampler or MSAA at runtime. The linear-light row is a computed reference, not a second renderer.",
            )
            .small()
            .color(self.theme.text_tertiary),
        );
        ui.checkbox(
            &mut self.gallery_state.show_linear_blend_reference,
            "Show computed linear-light blend reference",
        );
        ui.add_space(SPACE_LG);

        self.draw_rendering_blend_samples(ui);
        ui.add_space(SPACE_2XL);
        self.draw_rendering_stroke_samples(ui, ppp);
        ui.add_space(SPACE_2XL);
        self.draw_rendering_text_samples(ui);
        ui.add_space(SPACE_2XL);
        self.draw_rendering_overlay_samples(ui);
    }

    fn draw_rendering_blend_samples(&self, ui: &mut Ui) {
        self.draw_section_heading(
            ui,
            "Alpha blending",
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
            "The top row uses egui's source-over blend. The optional lower row is the sRGB-encoded result of blending in linear light.",
        );

        let samples = [
            ("white 10%", [255, 255, 255], 26),
            ("white 25%", [255, 255, 255], 64),
            ("white 50%", [255, 255, 255], 128),
            ("black 10%", [0, 0, 0], 26),
            ("red 50%", [255, 0, 0], 128),
            ("green 50%", [0, 255, 0], 128),
            ("blue 50%", [0, 0, 255], 128),
        ];
        let backgrounds = [
            ("black", [0, 0, 0]),
            ("#111111", [17, 17, 17]),
            ("#181818", [24, 24, 24]),
            ("gray", [128, 128, 128]),
            ("white", [255, 255, 255]),
        ];

        egui::ScrollArea::horizontal()
            .id_salt("rendering_blend_samples")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(100.0);
                    for (label, _, _) in samples {
                        ui.add_sized(Vec2::new(72.0, 22.0), egui::Label::new(label));
                    }
                });
                for (background_name, background) in backgrounds {
                    draw_blend_row(ui, background_name, background, &samples, false);
                    if self.gallery_state.show_linear_blend_reference {
                        draw_blend_row(ui, "linear ref", background, &samples, true);
                    }
                }
            });
    }

    fn draw_rendering_stroke_samples(&mut self, ui: &mut Ui, ppp: f32) {
        self.draw_section_heading(
            ui,
            "Thin strokes and pixel alignment",
            "Each stroke is sized in physical pixels and converted to logical points using the live pixels_per_point.",
        );

        ui.checkbox(
            &mut self.gallery_state.align_strokes_to_pixels,
            "Align stroke centers to physical pixel centers",
        );
        let align_centers = self.gallery_state.align_strokes_to_pixels;

        let widths_px = [0.5_f32, 1.0, 1.5, 2.0];
        for width_px in widths_px {
            let width = width_px / ppp.max(0.1);
            ui.horizontal(|ui| {
                ui.add_sized(Vec2::new(46.0, 38.0), egui::Label::new(format!("{width_px:.1} px")));
                let (rect, _) = ui.allocate_exact_size(Vec2::new(168.0, 38.0), egui::Sense::hover());
                let y = snap_pixel_center(rect.center().y, ppp, align_centers);
                ui.painter().line_segment(
                    [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                    Stroke::new(width, self.theme.text_primary),
                );
                let (rect, _) = ui.allocate_exact_size(Vec2::new(54.0, 38.0), egui::Sense::hover());
                let x = snap_pixel_center(rect.center().x, ppp, align_centers);
                ui.painter().line_segment(
                    [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                    Stroke::new(width, self.theme.text_primary),
                );
                let (rect, _) = ui.allocate_exact_size(Vec2::new(54.0, 38.0), egui::Sense::hover());
                let shape_rect = Rect::from_center_size(rect.center(), Vec2::splat(28.0));
                ui.painter().rect_stroke(shape_rect, Rounding::same(6.0), Stroke::new(width, self.theme.accent));
                let (rect, _) = ui.allocate_exact_size(Vec2::new(54.0, 38.0), egui::Sense::hover());
                ui.painter()
                    .circle_stroke(rect.center(), 13.0, Stroke::new(width, self.theme.accent));
                let (rect, _) = ui.allocate_exact_size(Vec2::new(120.0, 38.0), egui::Sense::hover());
                let y = snap_pixel_center(rect.center().y, ppp, align_centers);
                ui.painter().line_segment(
                    [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                    Stroke::new(width, self.theme.border_subtle),
                );
            });
        }
    }

    fn draw_rendering_text_samples(&self, ui: &mut Ui) {
        self.draw_section_heading(
            ui,
            "Text rasterization",
            "Compare sizes, configured font families, and alpha on the same glyph sequence.",
        );
        let sizes = [10.0_f32, 11.0, 12.0, 13.0, 14.0, 16.0];
        let alphas = [("100%", 255), ("80%", 204), ("60%", 153), ("40%", 102)];
        let columns = ["regular", "medium", "bold", "monospace"];

        ui.horizontal(|ui| {
            ui.add_sized(Vec2::new(36.0, 22.0), egui::Label::new("pt"));
            for column in columns {
                ui.add_sized(Vec2::new(220.0, 22.0), egui::Label::new(column));
            }
        });

        for size in sizes {
            ui.horizontal_top(|ui| {
                ui.add_sized(Vec2::new(36.0, 20.0), egui::Label::new(format!("{size:.0}")));
                for column in columns {
                    ui.vertical(|ui| {
                        ui.set_min_width(220.0);
                        let family = match column {
                            "medium" => FontFamily::Name("ui_medium".into()),
                            "monospace" => FontFamily::Monospace,
                            _ => FontFamily::Proportional,
                        };
                        for (alpha_label, alpha) in alphas {
                            // Alpha must modulate the active theme's glyph color; a hardcoded
                            // light gray goes invisible on light surfaces.
                            let base = self.theme.text_primary;
                            let color = Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), alpha);
                            let mut text = RichText::new(format!("Query 0x7F3A · {alpha_label}"))
                                .font(FontId::new(size, family.clone()))
                                .color(color);
                            if column == "bold" {
                                text = text.strong();
                            }
                            ui.label(text);
                        }
                    });
                }
            });
            ui.add_space(SPACE_XS);
        }
    }

    fn draw_rendering_overlay_samples(&self, ui: &mut Ui) {
        self.draw_section_heading(
            ui,
            "Overlay and surface states",
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
            "Hover the first surface for a tooltip. The selection and modal samples exercise alpha over theme surfaces.",
        );

        self.draw_rendering_surface_states(ui);
        self.draw_rendering_modal_overlay(ui);
    }

    fn draw_rendering_surface_states(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let theme = self.theme;
            let (rect, response) = ui.allocate_exact_size(Vec2::new(180.0, 72.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, Rounding::same(6.0), theme.surface_panel);
            ui.painter().rect_stroke(rect, Rounding::same(6.0), Stroke::new(1.0, theme.border_subtle));
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Panel · hover me",
                font_body(),
                theme.text_primary,
            );
            response.on_hover_text("Tooltip sample: high contrast text over a floating surface.");

            let (rect, _) = ui.allocate_exact_size(Vec2::new(180.0, 72.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, Rounding::same(6.0), theme.surface_elevated);
            ui.painter().rect_stroke(rect, Rounding::same(6.0), Stroke::new(1.0, theme.border_focus));
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Selected row",
                font_body(),
                theme.text_primary,
            );

            let (rect, _) = ui.allocate_exact_size(Vec2::new(180.0, 72.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, Rounding::same(6.0), theme.surface_hover);
            ui.painter().rect_stroke(rect, Rounding::same(6.0), Stroke::new(1.0, theme.border_default));
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Hover state",
                font_body(),
                theme.text_primary,
            );
        });

    }

    fn draw_rendering_modal_overlay(&self, ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width().min(620.0), 120.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, Rounding::same(8.0), self.theme.surface_app);
        ui.painter().rect_filled(
            rect.shrink(1.0),
            Rounding::same(8.0),
            Color32::from_black_alpha(104),
        );
        let modal = Rect::from_center_size(rect.center(), Vec2::new(260.0, 80.0));
        // cc-scan:allow LINE_TOO_LONG — literal must not wrap
        ui.painter().rect_filled(modal.translate(Vec2::new(0.0, 3.0)), Rounding::same(8.0), Color32::from_black_alpha(90));
        ui.painter().rect_filled(modal, Rounding::same(8.0), self.theme.surface_panel);
        ui.painter().rect_stroke(modal, Rounding::same(8.0), Stroke::new(1.0, self.theme.border_default));
        ui.painter().text(
            modal.center(),
            egui::Align2::CENTER_CENTER,
            "Modal over dimmed surface",
            font_body(),
            self.theme.text_primary,
        );
    }
}

fn draw_blend_row(
    ui: &mut Ui,
    label: &str,
    background: [u8; 3],
    samples: &[(&str, [u8; 3], u8)],
    linear_reference: bool,
) {
    ui.horizontal(|ui| {
        ui.add_sized(Vec2::new(100.0, 40.0), egui::Label::new(label));
        for (_, foreground, alpha) in samples {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(72.0, 34.0), egui::Sense::hover());
            if linear_reference {
                ui.painter().rect_filled(
                    rect,
                    Rounding::ZERO,
                    linear_blend_reference(background, *foreground, *alpha),
                );
            } else {
                ui.painter().rect_filled(
                    rect,
                    Rounding::ZERO,
                    Color32::from_rgb(background[0], background[1], background[2]),
                );
                ui.painter().rect_filled(
                    rect,
                    Rounding::ZERO,
                    Color32::from_rgba_unmultiplied(foreground[0], foreground[1], foreground[2], *alpha),
                );
            }
            ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, Color32::from_gray(96)));
        }
    });
}

fn linear_blend_reference(background: [u8; 3], foreground: [u8; 3], alpha: u8) -> Color32 {
    // Inputs and the returned opaque swatch are sRGB bytes. Decode RGB to linear light, composite
    // with straight alpha in linear space, then encode the result back to sRGB for Color32.
    let alpha = f32::from(alpha) / 255.0;
    let output_linear: [f32; 3] = std::array::from_fn(|index| {
        let background = srgb_to_linear(f32::from(background[index]) / 255.0);
        let foreground = srgb_to_linear(f32::from(foreground[index]) / 255.0);
        foreground * alpha + background * (1.0 - alpha)
    });
    egui::Rgba::from_rgba_unmultiplied(output_linear[0], output_linear[1], output_linear[2], 1.0).into()
}

fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn snap_pixel_center(point: f32, pixels_per_point: f32, enabled: bool) -> f32 {
    if enabled && pixels_per_point > 0.0 {
        ((point * pixels_per_point).floor() + 0.5) / pixels_per_point
    } else {
        point
    }
}

#[cfg(test)]
mod tests {
    use super::{linear_blend_reference, snap_pixel_center};
    use egui::Color32;

    #[test]
    fn linear_reference_encodes_half_white_over_black_as_srgb_188() {
        assert_eq!(linear_blend_reference([0, 0, 0], [255, 255, 255], 128), Color32::from_gray(188));
    }

    #[test]
    fn pixel_center_snapping_tracks_physical_scale() {
        assert_eq!(snap_pixel_center(10.0, 2.0, true), 10.25);
        assert_eq!(snap_pixel_center(10.0, 1.0, false), 10.0);
    }
}
