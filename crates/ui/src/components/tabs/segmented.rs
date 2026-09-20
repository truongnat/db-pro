//! Pill-style segmented tab track.

use super::config::{
    SEGMENTED_ITEM_GAP, SEGMENTED_ITEM_HEIGHT, SEGMENTED_ITEM_MIN_WIDTH, SEGMENTED_LABEL_PAD_X, SEGMENTED_PILL_RADIUS,
    SEGMENTED_TRACK_PAD, SEGMENTED_TRACK_RADIUS,
};
use super::layout::{apply_keyboard_selection, apply_selection, collect_tab_row, track_id, TabHit};
use super::style::{TabItemStyle, TabKind};
use super::track::TabTrackerAnimation;
use crate::components::interact::radio_info;
use crate::DbProTheme;
use egui::{Align2, CursorIcon, Frame, Margin, Rect, Rounding, Sense, Stroke, Ui, Vec2};

pub struct SegmentedTabs<'a> {
    selected: &'a mut usize,
    tabs: &'a [&'a str],
    theme: DbProTheme,
    focusable: bool,
}

impl<'a> SegmentedTabs<'a> {
    pub fn new(selected: &'a mut usize, tabs: &'a [&'a str], theme: DbProTheme) -> Self {
        Self {
            selected,
            tabs,
            theme,
            focusable: true,
        }
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let first_tab = self.tabs.first().copied().unwrap_or("");
        Frame {
            fill: self.theme.surface_hover,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            inner_margin: Margin::same(SEGMENTED_TRACK_PAD),
            rounding: Rounding::same(SEGMENTED_TRACK_RADIUS),
            ..Default::default()
        }
        .show(ui, |ui| {
            let track_id = track_id(ui, "segmented_tabs", first_tab);
            let track_origin_x = ui.max_rect().left();
            let pill_shape_idx = ui.painter().add(egui::Shape::Noop);

            let (tab_rects, clicked_idx, focused_idx) = collect_tab_row(
                ui,
                self.tabs,
                *self.selected,
                SEGMENTED_ITEM_GAP,
                |ui, name, is_active| self.paint_item(ui, name, is_active),
            );
            apply_selection(self.selected, clicked_idx);
            apply_keyboard_selection(ui, self.selected, focused_idx, self.tabs.len());

            if let Some(target) = tab_rects.get(*self.selected).copied() {
                let pill = TabTrackerAnimation::animate_pill(ui.ctx(), track_id, track_origin_x, target);
                self.paint_active_pill(ui, pill, pill_shape_idx);
            }
        });
    }

    fn paint_item(&self, ui: &mut Ui, tab_name: &str, is_active: bool) -> TabHit {
        let measure = TabItemStyle::new(TabKind::Segmented, is_active, false, &self.theme);
        let text_width = ui.fonts(|fonts| {
            fonts
                .layout_no_wrap(tab_name.to_owned(), measure.font_id.clone(), measure.text_color)
                .size()
                .x
        });

        let item_width = (text_width + SEGMENTED_LABEL_PAD_X).max(SEGMENTED_ITEM_MIN_WIDTH);
        let sense = Sense {
            click: true,
            drag: false,
            focusable: self.focusable,
        };
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(item_width, SEGMENTED_ITEM_HEIGHT), sense);
        let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
        if self.focusable && resp.clicked() {
            ui.memory_mut(|memory| memory.request_focus(resp.id));
        }
        resp.widget_info(|| radio_info(true, is_active, tab_name));

        let style = TabItemStyle::new(TabKind::Segmented, is_active, resp.hovered(), &self.theme);
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

    fn paint_active_pill(&self, ui: &Ui, pill: Rect, shape_idx: egui::layers::ShapeIdx) {
        let pill_shapes = vec![
            egui::Shape::rect_filled(pill, Rounding::same(SEGMENTED_PILL_RADIUS), self.theme.surface_elevated),
            egui::Shape::rect_stroke(
                pill,
                Rounding::same(SEGMENTED_PILL_RADIUS),
                Stroke::new(1.0, self.theme.border_subtle),
            ),
        ];
        ui.painter().set(shape_idx, egui::Shape::Vec(pill_shapes));
    }
}
