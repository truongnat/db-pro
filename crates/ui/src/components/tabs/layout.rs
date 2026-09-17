//! Shared tab-row hit collection helpers.

use egui::{Rect, Ui};

pub(super) struct TabHit {
    pub rect: Rect,
    pub clicked: bool,
}

/// Walk labels left-to-right, record each item rect, and remember a click.
pub(super) fn collect_tab_row(
    ui: &mut Ui,
    tabs: &[&str],
    selected: usize,
    item_gap: f32,
    mut paint_item: impl FnMut(&mut Ui, &str, bool) -> TabHit,
) -> (Vec<Rect>, Option<usize>) {
    let mut tab_rects = Vec::with_capacity(tabs.len());
    let mut clicked_idx = None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(item_gap, 0.0);
        for (idx, tab_name) in tabs.iter().enumerate() {
            let hit = paint_item(ui, tab_name, idx == selected);
            tab_rects.push(hit.rect);
            if hit.clicked {
                clicked_idx = Some(idx);
            }
        }
    });

    (tab_rects, clicked_idx)
}

pub(super) fn apply_selection(selected: &mut usize, clicked_idx: Option<usize>) {
    if let Some(idx) = clicked_idx {
        *selected = idx;
    }
}

pub(super) fn track_id(ui: &Ui, salt: &'static str, first_tab: &str) -> egui::Id {
    ui.id().with(salt).with(first_tab)
}
