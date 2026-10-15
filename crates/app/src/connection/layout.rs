/// Column widths for one form row: `weights[i]/sum(weights)` of the space left
/// after `gap * (n-1)` inter-cell spacing. Relative weights only — `[2, 1]` is
/// a 2:1 split. Returns one width per weight.
pub fn row_widths(available_width: f32, gap: f32, weights: &[f32]) -> Vec<f32> {
    let count = weights.len().max(1);
    let total = (available_width - gap * (count.saturating_sub(1)) as f32).max(0.0);
    let weight_sum: f32 = weights.iter().sum();
    if weight_sum <= 0.0 {
        return vec![total / count as f32; count];
    }
    weights.iter().map(|w| total * w / weight_sum).collect()
}

/// Widths for the SQLite database-file row (input + trailing Browse button).
pub fn sqlite_file_row(available_width: f32, gap: f32) -> (f32, f32) {
    let button_w = 110.0;
    let input_w = (available_width - button_w - gap).max(120.0);
    (input_w, button_w)
}


/// Render `cells` side-by-side, each pinned to its share of the row.
///
/// Cells take `Box<dyn FnOnce(&mut Ui)>` so callers can pass one-off component
/// closures without tripping the borrow checker on `self` fields.
#[allow(clippy::type_complexity)]
pub fn form_row(ui: &mut egui::Ui, widths: &[f32], cells: Vec<Box<dyn FnOnce(&mut egui::Ui) + '_>>, gap: f32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (width, cell) in widths.iter().copied().zip(cells) {
            // allocate_ui pins the cell to exactly `width` — ui.scope would leave
            // the child's max_rect at the remaining row width.
            ui.allocate_ui_with_layout(
                egui::vec2(width, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| cell(ui),
            );
        }
    });
}