//! Small responsive layout primitives sized from the current `egui::Ui`.
//!
//! Widths are logical points. Invalid/non-finite widths and gaps are treated as zero;
//! minimum widths are clamped to zero and column limits are at least one. A grid narrower
//! than its requested minimum still uses one cell, bounded to the available width.

use egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Vec2};

use super::{config, handler};

/// Bounded fluid content width, including optional horizontal gutters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContainerWidth {
    /// Width inside the gutters.
    pub content_width: f32,
    /// Left/right inset applied inside the available parent width.
    pub gutter: f32,
}

/// Calculate a centered content width without exceeding the parent's available width.
pub fn container_width(available_width: f32, max_width: Option<f32>, gutter: f32) -> ContainerWidth {
    let available = finite_nonnegative(available_width);
    let gutter = handler::bounded_gutter(available, gutter);
    let inner = (available - 2.0 * gutter).max(0.0);
    let content_width = max_width
        .map(finite_nonnegative)
        .map_or(inner, |maximum| inner.min(maximum));
    ContainerWidth {
        content_width,
        gutter: gutter + (inner - content_width) * 0.5,
    }
}

/// Result of sizing an equal-column responsive grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridMetrics {
    pub columns: usize,
    pub cell_width: f32,
    pub gap: f32,
}

/// Choose the largest column count whose cells meet `min_cell_width`.
///
/// At least one column is always returned, including at zero width or when the
/// available width is smaller than the requested minimum. `max_columns` is clamped to one.
pub fn grid_metrics(available_width: f32, min_cell_width: f32, gap: f32, max_columns: usize) -> GridMetrics {
    let available = finite_nonnegative(available_width);
    let minimum = finite_nonnegative(min_cell_width);
    let gap = finite_nonnegative(gap);
    let cap = handler::column_count(max_columns);
    let columns = if minimum == 0.0 {
        cap
    } else {
        (1..=cap)
            .rev()
            .find(|count| available >= *count as f32 * minimum + (*count - 1) as f32 * gap)
            .unwrap_or(1)
    };
    let cell_width = ((available - (columns - 1) as f32 * gap).max(0.0) / columns as f32).min(available);
    GridMetrics {
        columns,
        cell_width,
        gap,
    }
}

/// Fluid, optionally max-width centered content container.
#[derive(Clone, Copy, Debug, Default)]
pub struct Container {
    max_width: Option<f32>,
    gutter: f32,
}

impl Container {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(finite_nonnegative(width));
        self
    }

    pub fn gutter(mut self, gutter: f32) -> Self {
        self.gutter = finite_nonnegative(gutter);
        self
    }

    /// Render content within a child UI whose width never exceeds the parent.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> (Response, R) {
        let available = ui.available_width().max(0.0);
        let metrics = container_width(available, self.max_width, self.gutter);
        let width = metrics.content_width;
        let available_height = ui.available_height().max(0.0);
        let start = ui.next_widget_position();
        let content_rect = egui::Rect::from_min_size(
            egui::pos2(start.x + metrics.gutter, start.y),
            Vec2::new(width, available_height),
        );
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(content_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        let result = add_contents(&mut child);
        let child_rect = child.min_rect();
        let allocated_rect = egui::Rect::from_min_max(
            egui::pos2(start.x, start.y),
            egui::pos2(start.x + available, child_rect.bottom().max(start.y)),
        );
        let response = ui.allocate_rect(allocated_rect, Sense::hover());
        (response, result)
    }
}

/// Equal-width grid whose column count follows the local available width.
#[derive(Clone, Copy, Debug)]
pub struct ResponsiveGrid {
    min_cell_width: f32,
    gap: f32,
    max_columns: usize,
}

impl ResponsiveGrid {
    pub fn new(min_cell_width: f32) -> Self {
        Self {
            min_cell_width: finite_nonnegative(min_cell_width),
            gap: 0.0,
            max_columns: usize::MAX,
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = finite_nonnegative(gap);
        self
    }

    pub fn max_columns(mut self, max_columns: usize) -> Self {
        self.max_columns = max_columns.max(1);
        self
    }

    pub fn metrics(self, available_width: f32) -> GridMetrics {
        grid_metrics(available_width, self.min_cell_width, self.gap, self.max_columns)
    }

    /// Render items row-wise. Cells are bounded to the allocated grid width.
    pub fn show<T, I, R>(self, ui: &mut Ui, items: I, mut add_cell: impl FnMut(&mut Ui, T) -> R) -> Vec<R>
    where
        I: IntoIterator<Item = T>,
    {
        let metrics = self.metrics(ui.available_width());
        let content_spacing = ui.spacing().item_spacing;
        ui.scope(|ui| {
            // The grid owns both axes of spacing; remove the parent row spacing so the
            // vertical gap is exactly the same token as the horizontal gap.
            ui.spacing_mut().item_spacing.y = 0.0;
            let mut results = Vec::new();
            let mut items = items.into_iter();
            let mut has_rendered_row = false;
            loop {
                let row: Vec<T> = items.by_ref().take(metrics.columns).collect();
                if row.is_empty() {
                    break;
                }
                if has_rendered_row {
                    ui.add_space(metrics.gap);
                }
                let row_width = ui.available_width();
                let cells =
                    ui.allocate_ui_with_layout(Vec2::new(row_width, 0.0), Layout::left_to_right(Align::Min), |ui| {
                        ui.spacing_mut().item_spacing.x = metrics.gap;
                        let mut cells = Vec::with_capacity(row.len());
                        for item in row {
                            let cell_resp = ui.allocate_ui_with_layout(
                                Vec2::new(metrics.cell_width, 0.0),
                                Layout::top_down(Align::Min),
                                |cell_ui| {
                                    cell_ui.spacing_mut().item_spacing = content_spacing;
                                    cell_ui.set_max_width(metrics.cell_width);
                                    add_cell(cell_ui, item)
                                },
                            );
                            cells.push(cell_resp.inner);
                        }
                        cells
                    });
                results.extend(cells.inner);
                has_rendered_row = true;
            }
            results
        })
        .inner
    }
}

fn finite_nonnegative(value: f32) -> f32 {
    config::finite_nonnegative(value)
}

#[cfg(test)]
#[path = "tests/ui_tests.rs"]
mod tests;
