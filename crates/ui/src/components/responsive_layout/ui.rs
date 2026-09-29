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
        let mut results = Vec::new();
        let mut items = items.into_iter();
        loop {
            let row: Vec<T> = items.by_ref().take(metrics.columns).collect();
            if row.is_empty() {
                break;
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
                                cell_ui.set_max_width(metrics.cell_width);
                                add_cell(cell_ui, item)
                            },
                        );
                        cells.push(cell_resp.inner);
                    }
                    cells
                });
            results.extend(cells.inner);
        }
        results
    }
}

fn finite_nonnegative(value: f32) -> f32 {
    config::finite_nonnegative(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_is_bounded_and_centered() {
        assert_eq!(
            container_width(900.0, Some(600.0), 20.0),
            ContainerWidth {
                content_width: 600.0,
                gutter: 150.0
            }
        );
        assert_eq!(
            container_width(30.0, Some(600.0), 20.0),
            ContainerWidth {
                content_width: 0.0,
                gutter: 15.0
            }
        );
    }

    #[test]
    fn grid_respects_minimum_gap_and_column_cap() {
        assert_eq!(
            grid_metrics(500.0, 100.0, 20.0, 3),
            GridMetrics {
                columns: 3,
                cell_width: 153.33333,
                gap: 20.0
            }
        );
        assert_eq!(grid_metrics(219.0, 100.0, 20.0, 4).columns, 1);
        assert_eq!(grid_metrics(0.0, 100.0, 20.0, 4).cell_width, 0.0);
        assert_eq!(grid_metrics(900.0, 0.0, 0.0, 0).columns, 1);
    }

    #[test]
    fn invalid_inputs_are_safe() {
        let result = grid_metrics(f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 2);
        assert_eq!(
            result,
            GridMetrics {
                columns: 2,
                cell_width: 0.0,
                gap: 0.0
            }
        );
    }

    #[test]
    fn container_consumes_intrinsic_height_before_following_sibling() {
        let context = egui::Context::default();
        let mut child_bounds = Vec::new();
        let mut sibling_bounds = None;
        let _ = context.run(egui::RawInput::default(), |context| {
            egui::CentralPanel::default().show(context, |ui| {
                Container::new().max_width(100.0).show(ui, |content| {
                    content.allocate_exact_size(Vec2::new(40.0, 30.0), Sense::hover());
                    child_bounds.push(content.min_rect());
                });
                sibling_bounds = Some(ui.allocate_exact_size(Vec2::new(20.0, 10.0), Sense::hover()).0);
            });
        });
        let content = child_bounds[0];
        let sibling = sibling_bounds.unwrap();
        assert!(sibling.top() >= content.bottom());
        assert!(content.width() <= 100.0);
    }

    #[test]
    fn container_sequential_siblings_and_narrow_bounds() {
        let context = egui::Context::default();
        let mut first_child = None;
        let mut second_child = None;
        let mut parent_resp_1 = None;
        let mut parent_resp_2 = None;
        let _ = context.run(egui::RawInput::default(), |context| {
            egui::CentralPanel::default().show(context, |ui| {
                let (r1, _) = Container::new().max_width(50.0).gutter(10.0).show(ui, |content| {
                    content.allocate_exact_size(Vec2::new(30.0, 40.0), Sense::hover());
                    first_child = Some(content.min_rect());
                });
                parent_resp_1 = Some(r1);

                let (r2, _) = Container::new().max_width(80.0).show(ui, |content| {
                    content.allocate_exact_size(Vec2::new(20.0, 25.0), Sense::hover());
                    second_child = Some(content.min_rect());
                });
                parent_resp_2 = Some(r2);
            });
        });

        let c1 = first_child.unwrap();
        let c2 = second_child.unwrap();
        let p1 = parent_resp_1.unwrap().rect;
        let p2 = parent_resp_2.unwrap().rect;

        assert!(
            p2.top() >= p1.bottom(),
            "Sequential container must stack below previous"
        );
        assert!(c2.top() >= c1.bottom(), "Child 2 must start below child 1");
        assert!(c1.width() <= 50.0);
        assert!(c2.width() <= 80.0);
    }

    #[test]
    fn grid_cells_render_with_bounded_width_and_intrinsic_row_height() {
        let context = egui::Context::default();
        let mut bounds = Vec::new();
        let _ = context.run(egui::RawInput::default(), |context| {
            egui::CentralPanel::default().show(context, |ui| {
                ResponsiveGrid::new(80.0)
                    .gap(8.0)
                    .max_columns(2)
                    .show(ui, [30.0, 50.0, 20.0], |cell, height| {
                        cell.allocate_exact_size(Vec2::new(40.0, height), Sense::hover());
                        bounds.push(cell.min_rect());
                    });
            });
        });
        assert_eq!(bounds.len(), 3);
        assert!(bounds.iter().all(|rect| rect.width() <= 80.0));
        // First row contains bounds[0] and bounds[1], second row contains bounds[2]
        assert!(bounds[0].left() < bounds[1].left());
        assert_eq!(bounds[0].top(), bounds[1].top());
        assert!(bounds[2].top() > bounds[0].top());
    }
}
