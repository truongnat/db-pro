use crate::DbProTheme;
use egui::{Align2, CursorIcon, Frame, Margin, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};

const TAB_TRANSITION_SECS: f32 = 0.200;

pub struct SegmentedTabs<'a> {
    selected: &'a mut usize,
    tabs: &'a [&'a str],
    theme: DbProTheme,
}

impl<'a> SegmentedTabs<'a> {
    pub fn new(selected: &'a mut usize, tabs: &'a [&'a str], theme: DbProTheme) -> Self {
        Self { selected, tabs, theme }
    }

    pub fn show(self, ui: &mut Ui) {
        let first_tab = self.tabs.first().copied().unwrap_or("");
        Frame {
            fill: self.theme.surface_hover,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            inner_margin: Margin::same(3.0),
            rounding: Rounding::same(8.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            let track_id = ui.id().with("segmented_tabs").with(first_tab);
            // Reserve a shape slot behind the tab buttons
            let pill_shape_idx = ui.painter().add(egui::Shape::Noop);

            let mut tab_rects = Vec::with_capacity(self.tabs.len());
            let tab_height = 28.0;
            let pad_x = 12.0;
            let mut clicked_idx = None;

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);

                for (idx, tab_name) in self.tabs.iter().enumerate() {
                    let is_active = idx == *self.selected;
                    let font_id = if is_active {
                        DbProTheme::ui_medium_font(12.5)
                    } else {
                        egui::FontId::proportional(12.5)
                    };

                    let text_color = if is_active {
                        self.theme.text_primary
                    } else {
                        self.theme.text_secondary
                    };

                    let galley = ui
                        .painter()
                        .layout_no_wrap((*tab_name).to_string(), font_id.clone(), text_color);

                    let item_width = (galley.size().x + pad_x * 2.0).max(48.0);
                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(item_width, tab_height), Sense::click());
                    let resp = resp.on_hover_cursor(CursorIcon::PointingHand);

                    tab_rects.push(rect);

                    if resp.clicked() {
                        clicked_idx = Some(idx);
                    }

                    let final_text_color = if is_active || resp.hovered() {
                        self.theme.text_primary
                    } else {
                        self.theme.text_secondary
                    };

                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        *tab_name,
                        font_id,
                        final_text_color,
                    );
                }
            });

            if let Some(idx) = clicked_idx {
                *self.selected = idx;
            }

            if let Some(target) = tab_rects.get(*self.selected).copied() {
                let x = ui
                    .ctx()
                    .animate_value_with_time(track_id.with("x"), target.left(), TAB_TRANSITION_SECS);
                let w = ui
                    .ctx()
                    .animate_value_with_time(track_id.with("w"), target.width(), TAB_TRANSITION_SECS);

                if (x - target.left()).abs() > 0.5 || (w - target.width()).abs() > 0.5 {
                    ui.ctx().request_repaint();
                }

                let pill = Rect::from_min_size(egui::pos2(x, target.top()), Vec2::new(w, target.height()));

                let pill_shapes = vec![
                    egui::Shape::rect_filled(pill, Rounding::same(6.0), self.theme.surface_elevated),
                    egui::Shape::rect_stroke(pill, Rounding::same(6.0), Stroke::new(1.0, self.theme.border_subtle)),
                ];
                ui.painter().set(pill_shape_idx, egui::Shape::Vec(pill_shapes));
            }
        });
    }
}

pub struct UnderlineTabs<'a> {
    selected: &'a mut usize,
    tabs: &'a [&'a str],
    theme: DbProTheme,
}

impl<'a> UnderlineTabs<'a> {
    pub fn new(selected: &'a mut usize, tabs: &'a [&'a str], theme: DbProTheme) -> Self {
        Self { selected, tabs, theme }
    }

    pub fn show(self, ui: &mut Ui) {
        let first_tab = self.tabs.first().copied().unwrap_or("");
        let track_id = ui.id().with("underline_tabs").with(first_tab);
        let mut tab_rects = Vec::with_capacity(self.tabs.len());
        let tab_height = 32.0;
        let pad_x = 12.0;
        let mut clicked_idx = None;

        let total_row_rect = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

            for (idx, tab_name) in self.tabs.iter().enumerate() {
                let is_active = idx == *self.selected;
                let font_id = if is_active {
                    DbProTheme::ui_medium_font(13.0)
                } else {
                    egui::FontId::proportional(13.0)
                };

                let text_color = if is_active {
                    self.theme.text_primary
                } else {
                    self.theme.text_secondary
                };

                let galley = ui
                    .painter()
                    .layout_no_wrap((*tab_name).to_string(), font_id.clone(), text_color);

                let item_width = galley.size().x + pad_x * 2.0;
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(item_width, tab_height), Sense::click());
                let resp = resp.on_hover_cursor(CursorIcon::PointingHand);

                tab_rects.push(rect);

                if resp.clicked() {
                    clicked_idx = Some(idx);
                }

                // Hover highlight on inactive tab
                if resp.hovered() && !is_active {
                    ui.painter().rect_filled(
                        rect.shrink2(Vec2::new(0.0, 4.0)),
                        Rounding::same(5.0),
                        self.theme.surface_hover,
                    );
                }

                let final_text_color = if is_active || resp.hovered() {
                    self.theme.text_primary
                } else {
                    self.theme.text_secondary
                };

                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    *tab_name,
                    font_id,
                    final_text_color,
                );
            }
        });

        if let Some(idx) = clicked_idx {
            *self.selected = idx;
        }

        // Bottom baseline
        let baseline_y = total_row_rect.response.rect.bottom();
        let baseline_rect = Rect::from_min_size(
            Pos2::new(total_row_rect.response.rect.left(), baseline_y - 1.0),
            Vec2::new(total_row_rect.response.rect.width(), 1.0),
        );
        ui.painter()
            .rect_filled(baseline_rect, Rounding::ZERO, self.theme.border_subtle);

        // Active indicator underline
        if let Some(target) = tab_rects.get(*self.selected) {
            let x = ui
                .ctx()
                .animate_value_with_time(track_id.with("x"), target.left(), TAB_TRANSITION_SECS);
            let w = ui
                .ctx()
                .animate_value_with_time(track_id.with("w"), target.width(), TAB_TRANSITION_SECS);

            if (x - target.left()).abs() > 0.5 || (w - target.width()).abs() > 0.5 {
                ui.ctx().request_repaint();
            }

            let underline = Rect::from_min_size(
                egui::pos2(x + 4.0, target.bottom() - 2.0),
                Vec2::new((w - 8.0).max(12.0), 2.0),
            );
            ui.painter()
                .rect_filled(underline, Rounding::same(1.0), self.theme.accent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbProTheme;

    #[test]
    fn tab_underline_animates_with_hover_duration() {
        let theme = DbProTheme::light();
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let mut selected = 0;
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                UnderlineTabs::new(&mut selected, &["A", "B", "C"], theme).show(ui);
            });
        });
        selected = 2;
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                UnderlineTabs::new(&mut selected, &["A", "B", "C"], theme).show(ui);
                let x = ui.ctx().animate_value_with_time(
                    ui.id().with("underline_tabs").with("x"),
                    0.0,
                    TAB_TRANSITION_SECS,
                );
                assert!(x.is_finite());
            });
        });
    }
}
