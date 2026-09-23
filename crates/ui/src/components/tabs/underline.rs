//! Underline-style tab track with animated active indicator.

use super::config::{
    UNDERLINE_BASELINE_HEIGHT, UNDERLINE_HOVER_INSET_Y, UNDERLINE_HOVER_RADIUS, UNDERLINE_ITEM_GAP,
    UNDERLINE_ITEM_HEIGHT, UNDERLINE_LABEL_PAD_X,
};
use super::layout::{apply_keyboard_selection, apply_selection, track_id, TabHit};
use super::style::{TabItemStyle, TabKind};
use super::track::TabTrackerAnimation;
use crate::components::interact::radio_info;
use crate::DbProTheme;
use egui::{Align2, CursorIcon, Pos2, Rect, Rounding, Sense, Ui, Vec2};

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
        let track_id = track_id(ui, "underline_tabs", first_tab);

        let mut tab_rects = Vec::with_capacity(self.tabs.len());
        let mut clicked_idx = None;
        let mut focused_idx = None;
        // Keep the horizontal response so baseline/indicator share the row bounds.
        let row = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(UNDERLINE_ITEM_GAP, 0.0);
            for (idx, tab_name) in self.tabs.iter().enumerate() {
                let hit = self.paint_item(ui, tab_name, idx == *self.selected);
                tab_rects.push(hit.rect);
                if hit.clicked {
                    clicked_idx = Some(idx);
                }
                if hit.focused {
                    focused_idx = Some(idx);
                }
            }
        });
        apply_selection(self.selected, clicked_idx);
        apply_keyboard_selection(ui, self.selected, focused_idx, self.tabs.len());

        self.paint_baseline(ui, row.response.rect);

        if let Some(target) = tab_rects.get(*self.selected).copied() {
            self.paint_active_underline(ui, track_id, row.response.rect.left(), target);
        }
    }

    fn paint_item(&self, ui: &mut Ui, tab_name: &str, is_active: bool) -> TabHit {
        let measure = TabItemStyle::new(TabKind::Underline, is_active, false, &self.theme);
        let text_width = ui.fonts(|fonts| {
            fonts
                .layout_no_wrap(tab_name.to_owned(), measure.font_id.clone(), measure.text_color)
                .size()
                .x
        });

        let item_width = text_width + UNDERLINE_LABEL_PAD_X;
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(item_width, UNDERLINE_ITEM_HEIGHT), Sense::click());
        let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
        if resp.clicked() {
            ui.memory_mut(|memory| memory.request_focus(resp.id));
        }
        resp.widget_info(|| radio_info(true, is_active, tab_name));

        if resp.hovered() && !is_active {
            ui.painter().rect_filled(
                rect.shrink2(Vec2::new(0.0, UNDERLINE_HOVER_INSET_Y)),
                Rounding::same(UNDERLINE_HOVER_RADIUS),
                self.theme.surface_hover,
            );
        }

        let style = TabItemStyle::new(TabKind::Underline, is_active, resp.hovered(), &self.theme);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            tab_name,
            style.font_id,
            style.text_color,
        );

        TabHit {
            rect,
            clicked: resp.clicked(),
            focused: resp.has_focus(),
        }
    }

    fn paint_baseline(&self, ui: &Ui, row_rect: Rect) {
        let baseline = Rect::from_min_size(
            Pos2::new(row_rect.left(), row_rect.bottom() - UNDERLINE_BASELINE_HEIGHT),
            Vec2::new(row_rect.width(), UNDERLINE_BASELINE_HEIGHT),
        );
        ui.painter()
            .rect_filled(baseline, Rounding::ZERO, self.theme.border_subtle);
    }

    fn paint_active_underline(&self, ui: &Ui, track_id: egui::Id, track_origin_x: f32, target: Rect) {
        let underline = TabTrackerAnimation::animate_underline(ui.ctx(), track_id, track_origin_x, target);
        ui.painter()
            .rect_filled(underline, Rounding::same(1.0), self.theme.accent);
    }
}
