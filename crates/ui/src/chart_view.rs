use bigdecimal::BigDecimal;
use egui::{Align2, Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use lucide_icons::Icon;
use std::collections::{BTreeMap, HashSet};
use std::str::FromStr;

/// Chart visualization types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartType {
    #[default]
    Bar,
    Line,
    Area,
    Scatter,
    Pie,
}

impl std::fmt::Display for ChartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChartType::Bar => write!(f, "Bar"),
            ChartType::Line => write!(f, "Line"),
            ChartType::Area => write!(f, "Area"),
            ChartType::Scatter => write!(f, "Scatter"),
            ChartType::Pie => write!(f, "Pie"),
        }
    }
}

/// Aggregation options for reducing data before charting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartAggregation {
    #[default]
    None,
    Count,
    Sum,
    Average,
    Min,
    Max,
}

impl std::fmt::Display for ChartAggregation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChartAggregation::None => write!(f, "None"),
            ChartAggregation::Count => write!(f, "Count"),
            ChartAggregation::Sum => write!(f, "Sum"),
            ChartAggregation::Average => write!(f, "Average"),
            ChartAggregation::Min => write!(f, "Min"),
            ChartAggregation::Max => write!(f, "Max"),
        }
    }
}

/// Chart state that survives tab switches.
#[derive(Debug, Clone, Default)]
pub struct ChartConfig {
    pub chart_type: ChartType,
    pub x_column: Option<usize>,
    pub y_column: Option<usize>,
    pub series_column: Option<usize>,
    pub aggregation: ChartAggregation,
    pub max_points: usize,
}

impl ChartConfig {
    pub fn new() -> Self {
        Self {
            chart_type: ChartType::Bar,
            x_column: None,
            y_column: None,
            series_column: None,
            aggregation: ChartAggregation::None,
            max_points: 1000,
        }
    }
}

/// A single data point extracted from a result row.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartPoint {
    pub x: f64,
    pub y: f64,
    pub label: String,
    pub series: String,
}

/// Projection outcome — points plus explicit null/skip accounting for the UI.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChartProjection {
    pub points: Vec<ChartPoint>,
    /// Rows skipped because Y was null or non-numeric.
    pub skipped_null_y: usize,
    /// Rows whose X was null/non-numeric and fell back to the row index.
    pub x_fallback_to_index: usize,
}

/// Chart engine: projects query results into chart-ready points.
pub struct ChartEngine;

impl ChartEngine {
    const MAX_PIE_SLICES: usize = 12;

    /// Extract numeric value from a cell. Returns None if not numeric or null.
    pub fn cell_to_numeric(cell: &crate::UiCell) -> Option<f64> {
        match cell {
            crate::UiCell::Null => None,
            crate::UiCell::Boolean(v) => Some(if *v { 1.0 } else { 0.0 }),
            crate::UiCell::Number(v) => {
                // Try parsing as i64 first, then f64, then BigDecimal
                v.parse::<i64>()
                    .map(|i| i as f64)
                    .ok()
                    .or_else(|| v.parse::<f64>().ok())
                    .or_else(|| {
                        v.parse::<BigDecimal>()
                            .ok()
                            .and_then(|d| f64::from_str(&d.to_string()).ok())
                    })
            }
            crate::UiCell::Text(v) => {
                // Try parsing text as a number (for cases where the type info is imprecise)
                v.parse::<i64>()
                    .map(|i| i as f64)
                    .ok()
                    .or_else(|| v.parse::<f64>().ok())
            }
            crate::UiCell::Json(v) => {
                // Try parsing JSON number
                v.parse::<serde_json::Number>().ok().and_then(|n| n.as_f64())
            }
            crate::UiCell::Bytes(_) => None,
        }
    }

    /// Check if a column is numeric-typed (can be Y axis).
    pub fn is_numeric_column(data_type: &str) -> bool {
        let lower = data_type.to_lowercase();
        matches!(
            lower.as_str(),
            "int"
                | "integer"
                | "bigint"
                | "smallint"
                | "tinyint"
                | "decimal"
                | "numeric"
                | "float"
                | "double"
                | "real"
                | "number"
        ) || lower.contains("int")
            || lower.contains("float")
            || lower.contains("double")
            || lower.contains("decimal")
            || lower.contains("numeric")
            || lower.contains("real")
    }

    /// Project result rows into chart points with optional aggregation then downsampling.
    pub fn project(columns: &[crate::UiColumn], rows: &[Vec<crate::UiCell>], config: &ChartConfig) -> ChartProjection {
        let x_idx = config.x_column.unwrap_or(0);
        let y_idx = config.y_column.unwrap_or(1usize.min(columns.len().saturating_sub(1)));
        let series_idx = config.series_column.unwrap_or(columns.len()); // out of bounds = no series

        if x_idx >= columns.len() || y_idx >= columns.len() {
            return ChartProjection::default();
        }

        let mut points = Vec::with_capacity(rows.len().min(config.max_points.max(1)));
        let mut skipped_null_y = 0usize;
        let mut x_fallback_to_index = 0usize;

        for (row_idx, row) in rows.iter().enumerate() {
            let x_cell = &row[x_idx];
            let y_cell = &row[y_idx];
            let series = if series_idx < columns.len() {
                cell_to_label(&row[series_idx])
            } else {
                String::new()
            };

            let x = match Self::cell_to_numeric(x_cell) {
                Some(v) => v,
                None => {
                    if matches!(x_cell, crate::UiCell::Null) {
                        x_fallback_to_index += 1;
                    }
                    row_idx as f64
                }
            };
            let y = match Self::cell_to_numeric(y_cell) {
                Some(v) => v,
                None => {
                    skipped_null_y += 1;
                    continue;
                }
            };

            points.push(ChartPoint {
                x,
                y,
                label: cell_to_label(x_cell),
                series,
            });
        }

        // Aggregate on the full point set first so downsampling never hides groups.
        if config.aggregation != ChartAggregation::None && !points.is_empty() {
            points = aggregate_points(points, config.aggregation);
        }

        if points.len() > config.max_points {
            points = lttb_downsample(points, config.max_points);
        }

        ChartProjection {
            points,
            skipped_null_y,
            x_fallback_to_index,
        }
    }

    /// Project for pie chart: one value per category (grouped by label).
    pub fn project_pie(
        columns: &[crate::UiColumn],
        rows: &[Vec<crate::UiCell>],
        config: &ChartConfig,
    ) -> ChartProjection {
        let x_idx = config.x_column.unwrap_or(0);
        let y_idx = config.y_column.unwrap_or(1usize.min(columns.len().saturating_sub(1)));

        if x_idx >= columns.len() || y_idx >= columns.len() {
            return ChartProjection::default();
        }

        let mut groups: BTreeMap<String, f64> = BTreeMap::new();
        let mut skipped_null_y = 0usize;
        let mut x_fallback_to_index = 0usize;

        for row in rows {
            let label = match &row[x_idx] {
                crate::UiCell::Null => {
                    x_fallback_to_index += 1;
                    "NULL".to_owned()
                }
                other => cell_to_label(other),
            };
            if let Some(val) = Self::cell_to_numeric(&row[y_idx]) {
                *groups.entry(label).or_insert(0.0) += val;
            } else {
                skipped_null_y += 1;
            }
        }

        // Keep top N slices, rest go to "Other"
        let mut entries: Vec<(String, f64)> = groups.into_iter().collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut points: Vec<ChartPoint> = entries
            .iter()
            .take(Self::MAX_PIE_SLICES)
            .map(|(label, val)| ChartPoint {
                x: *val,
                y: *val,
                label: label.clone(),
                series: String::new(),
            })
            .collect();

        if entries.len() > Self::MAX_PIE_SLICES {
            let other_sum: f64 = entries[Self::MAX_PIE_SLICES..].iter().map(|(_, v)| v).sum();
            points.push(ChartPoint {
                x: other_sum,
                y: other_sum,
                label: "Other".into(),
                series: String::new(),
            });
        }

        ChartProjection {
            points,
            skipped_null_y,
            x_fallback_to_index,
        }
    }
}

/// Convert a cell to a label for display.
fn cell_to_label(cell: &crate::UiCell) -> String {
    match cell {
        crate::UiCell::Null => "NULL".into(),
        crate::UiCell::Boolean(v) => v.to_string(),
        crate::UiCell::Number(v) => v.clone(),
        crate::UiCell::Text(v) => v.clone(),
        crate::UiCell::Json(v) => {
            if v.len() > 20 {
                format!("{}…", &v[..20])
            } else {
                v.clone()
            }
        }
        crate::UiCell::Bytes(v) => format!("<{} bytes>", v.len()),
    }
}

/// Largest-Triangle-Three-Buckets downsampling for line/area/scatter.
fn lttb_downsample(points: Vec<ChartPoint>, target: usize) -> Vec<ChartPoint> {
    if points.len() <= target {
        return points;
    }
    if target == 0 {
        return Vec::new();
    }
    if target == 1 {
        return vec![points[0].clone()];
    }
    if target == 2 {
        return vec![points[0].clone(), points[points.len() - 1].clone()];
    }

    let mut sampled = Vec::with_capacity(target);
    sampled.push(points[0].clone());

    let bucket_size = (points.len() - 2) as f64 / (target - 2) as f64;
    let mut a = 0usize;

    for i in 1..target - 1 {
        let next_a = ((i as f64 + 1.0) * bucket_size).floor() as usize + 1;
        let next_a = next_a.min(points.len() - 1);

        // Calculate point average for next bucket
        let avg_x = points[a..next_a].iter().map(|p| p.x).sum::<f64>() / (next_a - a).max(1) as f64;
        let avg_y = points[a..next_a].iter().map(|p| p.y).sum::<f64>() / (next_a - a).max(1) as f64;

        let end = ((i as f64 + 2.0) * bucket_size).floor() as usize + 1;
        let end = end.min(points.len());

        let mut max_area = f64::NEG_INFINITY;
        let mut max_idx = a;

        for j in (a + 1)..end {
            let area = ((points[a].x - avg_x) * (points[j].y - points[a].y)
                - (points[a].x - points[j].x) * (avg_y - points[a].y))
                .abs();
            if area > max_area {
                max_area = area;
                max_idx = j;
            }
        }

        sampled.push(points[max_idx].clone());
        a = max_idx;
    }

    sampled.push(points[points.len() - 1].clone());
    sampled
}

/// Aggregate points by (series, label) using the specified function.
fn aggregate_points(points: Vec<ChartPoint>, agg: ChartAggregation) -> Vec<ChartPoint> {
    let mut groups: BTreeMap<(String, String), (f64, usize, f64, f64)> = BTreeMap::new();
    // key = (series, label); value = (sum, count, min, max)

    for p in &points {
        let entry =
            groups
                .entry((p.series.clone(), p.label.clone()))
                .or_insert((0.0, 0, f64::INFINITY, f64::NEG_INFINITY));
        entry.0 += p.y;
        entry.1 += 1;
        entry.2 = entry.2.min(p.y);
        entry.3 = entry.3.max(p.y);
    }

    groups
        .into_iter()
        .enumerate()
        .map(|(idx, ((series, label), (sum, count, min, max)))| {
            let y = match agg {
                ChartAggregation::None => sum,
                ChartAggregation::Count => count as f64,
                ChartAggregation::Sum => sum,
                ChartAggregation::Average => {
                    if count > 0 {
                        sum / count as f64
                    } else {
                        0.0
                    }
                }
                ChartAggregation::Min => min,
                ChartAggregation::Max => max,
            };
            ChartPoint {
                x: idx as f64,
                y,
                label,
                series,
            }
        })
        .collect()
}

/// Chart renderer using egui's built-in primitives.
pub struct ChartRenderer;

impl ChartRenderer {
    const MARGIN_LEFT: f64 = 60.0;
    const MARGIN_RIGHT: f64 = 20.0;
    const MARGIN_TOP: f64 = 30.0;
    const MARGIN_BOTTOM: f64 = 40.0;
    const PALETTE: [Color32; 6] = [
        Color32::from_rgb(66, 133, 244), // Blue
        Color32::from_rgb(219, 68, 55),  // Red
        Color32::from_rgb(244, 180, 0),  // Yellow
        Color32::from_rgb(15, 157, 88),  // Green
        Color32::from_rgb(171, 71, 188), // Purple
        Color32::from_rgb(0, 172, 193),  // Cyan
    ];

    /// Draw the chart onto the given UI area.
    pub fn draw(ui: &mut egui::Ui, points: &[ChartPoint], config: &ChartConfig, theme: &crate::theme::DbProTheme) {
        if points.is_empty() {
            ui.centered_and_justified(|ui| {
                crate::components::empty_state(
                    ui,
                    Icon::BarChart3,
                    "No data to chart",
                    "Select numeric columns and adjust chart settings.",
                    *theme,
                );
            });
            return;
        }

        let rect = ui.available_rect_before_wrap();

        // Draw title / axis labels area
        let chart_rect = Rect::from_min_size(
            Pos2::new(
                rect.min.x + Self::MARGIN_LEFT as f32,
                rect.min.y + Self::MARGIN_TOP as f32,
            ),
            Vec2::new(
                (rect.width() - Self::MARGIN_LEFT as f32 - Self::MARGIN_RIGHT as f32).max(10.0),
                (rect.height() - Self::MARGIN_TOP as f32 - Self::MARGIN_BOTTOM as f32).max(10.0),
            ),
        );

        // Compute data bounds
        let (x_min, _x_max, y_min, y_max) = Self::data_bounds(points);

        // Draw axes
        let painter = ui.painter();
        let axis_stroke = Stroke::new(1.0_f32, theme.text_secondary.gamma_multiply(0.5));

        // Y axis
        painter.line_segment([chart_rect.left_bottom(), chart_rect.left_top()], axis_stroke);
        // X axis
        painter.line_segment([chart_rect.left_bottom(), chart_rect.right_bottom()], axis_stroke);

        // Y-axis gridlines + labels
        let y_range = (y_max - y_min).abs().max(1e-10);
        let y_ticks = 5usize;
        for i in 0..=y_ticks {
            let frac = i as f64 / y_ticks as f64;
            let y_val = y_min + y_range * frac;
            let py = chart_rect.bottom() - (frac as f32 * chart_rect.height());
            painter.line_segment(
                [Pos2::new(chart_rect.left(), py), Pos2::new(chart_rect.right(), py)],
                Stroke::new(0.5_f32, theme.text_muted.gamma_multiply(0.3)),
            );
            painter.text(
                Pos2::new(chart_rect.left() - 5.0, py),
                Align2::RIGHT_CENTER,
                format_value(y_val),
                egui::FontId::proportional(10.0),
                theme.text_secondary,
            );
        }

        // X-axis labels (categorical — show at most 8)
        let unique_labels: Vec<String> = points
            .iter()
            .map(|p| p.label.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let x_ticks = unique_labels.len().min(8);
        #[allow(clippy::manual_checked_ops)]
        if x_ticks > 0 {
            let step = unique_labels.len() / x_ticks;
            for i in 0..x_ticks {
                let idx = i * step;
                let px = rect.left() + (idx as f32 / points.len().max(1) as f32) * rect.width();
                painter.text(
                    Pos2::new(px, chart_rect.bottom() + 5.0),
                    Align2::CENTER_TOP,
                    &unique_labels[idx],
                    egui::FontId::proportional(9.0),
                    theme.text_secondary,
                );
            }
        }

        // Draw chart based on type
        match config.chart_type {
            ChartType::Bar => Self::draw_bars(painter, chart_rect, points, x_min, y_min, y_range),
            ChartType::Line => Self::draw_line(painter, chart_rect, points, x_min, y_min, y_range, false),
            ChartType::Area => Self::draw_line(painter, chart_rect, points, x_min, y_min, y_range, true),
            ChartType::Scatter => Self::draw_scatter(painter, chart_rect, points, x_min, y_min, y_range),
            ChartType::Pie => Self::draw_pie(painter, chart_rect, points, theme),
        }

        // Draw count
        painter.text(
            Pos2::new(chart_rect.right(), chart_rect.top() - 15.0),
            Align2::RIGHT_CENTER,
            format!("{} points", points.len()),
            egui::FontId::proportional(10.0),
            theme.text_muted,
        );

        ui.allocate_rect(rect, egui::Sense::hover());
    }

    fn data_bounds(points: &[ChartPoint]) -> (f64, f64, f64, f64) {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;

        for p in points {
            x_min = x_min.min(p.x);
            x_max = x_max.max(p.x);
            y_min = y_min.min(p.y);
            y_max = y_max.max(p.y);
        }

        // Ensure non-zero range
        if (x_max - x_min).abs() < 1e-10 {
            x_min -= 1.0;
            x_max += 1.0;
        }
        if (y_max - y_min).abs() < 1e-10 {
            y_min -= 1.0;
            y_max += 1.0;
        }

        (x_min, x_max, y_min, y_max)
    }

    fn map_x(x: f64, x_min: f64, x_range: f64, rect: Rect) -> f32 {
        rect.left() + ((x - x_min) / x_range) as f32 * rect.width()
    }

    fn map_y(y: f64, y_min: f64, y_range: f64, rect: Rect) -> f32 {
        rect.bottom() - ((y - y_min) / y_range) as f32 * rect.height()
    }

    fn series_color(series_name: &str, series_order: &[String]) -> Color32 {
        if series_name.is_empty() {
            return Self::PALETTE[0];
        }
        let idx = series_order.iter().position(|name| name == series_name).unwrap_or(0);
        Self::PALETTE[idx % Self::PALETTE.len()]
    }

    fn series_order(points: &[ChartPoint]) -> Vec<String> {
        let mut order = Vec::new();
        for point in points {
            if !point.series.is_empty() && !order.iter().any(|existing| existing == &point.series) {
                order.push(point.series.clone());
            }
        }
        order
    }

    fn draw_bars(painter: &egui::Painter, rect: Rect, points: &[ChartPoint], _x_min: f64, y_min: f64, y_range: f64) {
        let bar_width = (rect.width() / points.len().max(1) as f32 * 0.7).max(2.0);
        let series_order = Self::series_order(points);

        for (i, point) in points.iter().enumerate() {
            let color = Self::series_color(&point.series, &series_order);
            let px = rect.left() + (i as f32 / points.len().max(1) as f32) * rect.width();
            let py = Self::map_y(point.y, y_min, y_range, rect);
            let base_y = Self::map_y(y_min.max(0.0), y_min, y_range, rect);

            let bar_rect = Rect::from_min_max(
                Pos2::new(px - bar_width / 2.0, py.min(base_y)),
                Pos2::new(px + bar_width / 2.0, py.max(base_y)),
            );

            painter.rect_filled(bar_rect, Rounding::ZERO, color.gamma_multiply(0.8));
        }
    }

    fn draw_line(
        painter: &egui::Painter,
        rect: Rect,
        points: &[ChartPoint],
        x_min: f64,
        y_min: f64,
        y_range: f64,
        fill_area: bool,
    ) {
        if points.len() < 2 {
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Need at least 2 points",
                egui::FontId::proportional(12.0),
                Color32::GRAY,
            );
            return;
        }

        let x_range = (points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) - x_min)
            .abs()
            .max(1.0);
        let series_order = Self::series_order(points);
        let series_keys: Vec<String> = if series_order.is_empty() {
            vec![String::new()]
        } else {
            series_order.clone()
        };

        for series_name in &series_keys {
            let series_points: Vec<&ChartPoint> = if series_name.is_empty() {
                points.iter().collect()
            } else {
                points.iter().filter(|p| &p.series == series_name).collect()
            };
            if series_points.len() < 2 {
                continue;
            }
            let color = Self::series_color(series_name, &series_order);
            let points_pos: Vec<Pos2> = series_points
                .iter()
                .map(|p| {
                    Pos2::new(
                        Self::map_x(p.x, x_min, x_range, rect),
                        Self::map_y(p.y, y_min, y_range, rect),
                    )
                })
                .collect();

            if fill_area {
                let mut fill_points = points_pos.clone();
                fill_points.push(Pos2::new(points_pos.last().unwrap().x, rect.bottom()));
                fill_points.push(Pos2::new(points_pos.first().unwrap().x, rect.bottom()));
                painter.add(egui::Shape::convex_polygon(
                    fill_points,
                    color.gamma_multiply(0.15),
                    Stroke::NONE,
                ));
            }

            painter.add(egui::Shape::line(points_pos.clone(), Stroke::new(2.0_f32, color)));
            for pos in &points_pos {
                painter.circle_filled(*pos, 3.0, color);
            }
        }
    }

    fn draw_scatter(painter: &egui::Painter, rect: Rect, points: &[ChartPoint], x_min: f64, y_min: f64, y_range: f64) {
        let x_range = (points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) - x_min)
            .abs()
            .max(1.0);
        let series_order = Self::series_order(points);

        for point in points {
            let color = Self::series_color(&point.series, &series_order);
            let px = Self::map_x(point.x, x_min, x_range, rect);
            let py = Self::map_y(point.y, y_min, y_range, rect);
            painter.circle_filled(Pos2::new(px, py), 4.0, color.gamma_multiply(0.7));
        }
    }

    fn draw_pie(painter: &egui::Painter, rect: Rect, points: &[ChartPoint], theme: &crate::theme::DbProTheme) {
        let total: f64 = points.iter().map(|p| p.y).sum();
        if total <= 0.0 {
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "No positive values",
                egui::FontId::proportional(12.0),
                Color32::GRAY,
            );
            return;
        }

        let center = rect.center();
        let radius = rect.width().min(rect.height()) / 2.0 * 0.8;
        let mut start_angle = 0.0_f64;

        for (i, point) in points.iter().enumerate() {
            let frac = point.y / total;
            let end_angle = start_angle + frac * std::f64::consts::TAU;

            let color = Self::PALETTE[i % Self::PALETTE.len()];

            // Draw slice as a polygon approximation
            let num_segments = 24;
            let angle_step = (end_angle - start_angle) / num_segments as f64;
            let mut polygon = vec![center];
            for s in 0..=num_segments {
                let a = start_angle + s as f64 * angle_step;
                polygon.push(Pos2::new(
                    center.x + (a.cos() * radius as f64) as f32,
                    center.y + (a.sin() * radius as f64) as f32,
                ));
            }
            polygon.push(center);

            painter.add(egui::Shape::convex_polygon(
                polygon,
                color,
                Stroke::new(1.0_f32, theme.surface_app),
            ));

            // Label
            let mid_angle = (start_angle + end_angle) / 2.0;
            let label_radius = radius * 0.7;
            let label_pos = Pos2::new(
                center.x + (mid_angle.cos() * label_radius as f64) as f32,
                center.y + (mid_angle.sin() * label_radius as f64) as f32,
            );
            painter.text(
                label_pos,
                Align2::CENTER_CENTER,
                format!("{}: {:.0}", point.label, point.y),
                egui::FontId::proportional(10.0),
                Color32::WHITE,
            );

            start_angle = end_angle;
        }

        // Legend
        let legend_x = rect.right() + 10.0;
        for (i, point) in points.iter().enumerate() {
            let color = Self::PALETTE[i % Self::PALETTE.len()];
            let ly = rect.min.y + i as f32 * 18.0;
            painter.rect_filled(
                Rect::from_min_size(Pos2::new(legend_x, ly), Vec2::new(12.0, 12.0)),
                Rounding::ZERO,
                color,
            );
            painter.text(
                Pos2::new(legend_x + 16.0, ly + 6.0),
                Align2::LEFT_CENTER,
                &point.label,
                egui::FontId::proportional(10.0),
                theme.text_primary,
            );
        }
    }
}

fn format_value(v: f64) -> String {
    if v.abs() >= 1e6 {
        format!("{:.2e}", v)
    } else if v.fract() == 0.0 && v.abs() < 1e6 {
        format!("{:.0}", v)
    } else {
        format!("{:.2}", v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiCell, UiColumn};

    #[test]
    fn is_numeric_column_recognizes_common_types() {
        assert!(ChartEngine::is_numeric_column("INTEGER"));
        assert!(ChartEngine::is_numeric_column("numeric(20,4)"));
        assert!(!ChartEngine::is_numeric_column("text"));
        assert!(!ChartEngine::is_numeric_column("jsonb"));
    }

    #[test]
    fn project_downsamples_to_max_points() {
        let columns = vec![
            UiColumn {
                name: "x".into(),
                data_type: "integer".into(),
                nullable: false,
            },
            UiColumn {
                name: "y".into(),
                data_type: "integer".into(),
                nullable: false,
            },
        ];
        let rows: Vec<Vec<UiCell>> = (0..50)
            .map(|i| vec![UiCell::Number(i.to_string()), UiCell::Number((i * 2).to_string())])
            .collect();
        let mut config = ChartConfig::new();
        config.x_column = Some(0);
        config.y_column = Some(1);
        config.max_points = 10;
        let points = ChartEngine::project(&columns, &rows, &config);
        assert!(
            points.points.len() <= 10,
            "expected downsample, got {}",
            points.points.len()
        );
    }

    #[test]
    fn project_aggregates_before_downsample_and_reports_null_skips() {
        let columns = vec![
            UiColumn {
                name: "category".into(),
                data_type: "text".into(),
                nullable: true,
            },
            UiColumn {
                name: "value".into(),
                data_type: "integer".into(),
                nullable: true,
            },
            UiColumn {
                name: "series".into(),
                data_type: "text".into(),
                nullable: false,
            },
        ];
        let rows = vec![
            vec![
                UiCell::Text("a".into()),
                UiCell::Number("1".into()),
                UiCell::Text("s1".into()),
            ],
            vec![
                UiCell::Text("a".into()),
                UiCell::Number("3".into()),
                UiCell::Text("s1".into()),
            ],
            vec![UiCell::Null, UiCell::Number("9".into()), UiCell::Text("s2".into())],
            vec![UiCell::Text("b".into()), UiCell::Null, UiCell::Text("s2".into())],
        ];
        let mut config = ChartConfig::new();
        config.x_column = Some(0);
        config.y_column = Some(1);
        config.series_column = Some(2);
        config.aggregation = ChartAggregation::Sum;
        config.max_points = 1;
        let projection = ChartEngine::project(&columns, &rows, &config);
        assert_eq!(projection.skipped_null_y, 1);
        assert_eq!(projection.x_fallback_to_index, 1);
        assert_eq!(projection.points.len(), 1, "downsample after aggregate");
        assert!(!projection.points[0].series.is_empty());
    }

    #[test]
    fn project_pie_reports_null_skips() {
        let columns = vec![
            UiColumn {
                name: "label".into(),
                data_type: "text".into(),
                nullable: true,
            },
            UiColumn {
                name: "value".into(),
                data_type: "integer".into(),
                nullable: true,
            },
        ];
        let rows = vec![
            vec![UiCell::Text("a".into()), UiCell::Number("2".into())],
            vec![UiCell::Null, UiCell::Number("3".into())],
            vec![UiCell::Text("b".into()), UiCell::Null],
        ];
        let mut config = ChartConfig::new();
        config.chart_type = ChartType::Pie;
        config.x_column = Some(0);
        config.y_column = Some(1);
        let projection = ChartEngine::project_pie(&columns, &rows, &config);
        assert_eq!(projection.skipped_null_y, 1);
        assert_eq!(projection.x_fallback_to_index, 1);
        assert!(projection.points.iter().any(|p| p.label == "NULL"));
    }

    #[test]
    fn project_handles_ten_thousand_rows_within_budget() {
        let columns = vec![
            UiColumn {
                name: "x".into(),
                data_type: "integer".into(),
                nullable: false,
            },
            UiColumn {
                name: "y".into(),
                data_type: "decimal".into(),
                nullable: false,
            },
        ];
        let rows: Vec<Vec<UiCell>> = (0..10_000)
            .map(|i| vec![UiCell::Number(i.to_string()), UiCell::Number(format!("{}.5", i % 1000))])
            .collect();
        let mut config = ChartConfig::new();
        config.x_column = Some(0);
        config.y_column = Some(1);
        config.max_points = 1_000;
        let started = std::time::Instant::now();
        let points = ChartEngine::project(&columns, &rows, &config);
        let elapsed = started.elapsed();
        assert_eq!(points.points.len(), 1_000);
        assert!(
            elapsed.as_millis() < 250,
            "10k-row project took {:?}, expected <250ms",
            elapsed
        );
    }
}
