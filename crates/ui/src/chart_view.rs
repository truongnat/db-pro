#[path = "chart_engine.rs"]
mod chart_engine;

pub use chart_engine::{ChartAggregation, ChartConfig, ChartEngine, ChartPoint, ChartProjection, ChartType};

use egui::{Align2, Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use lucide_icons::Icon;
use std::collections::HashSet;

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
        // allow: `x_ticks > 0` above guards against division by zero; checked_div would make
        // the step calculation harder to read. #[allow(unknown_lints)] below is a compatibility marker.
        #[allow(unknown_lints)]
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
                if let (Some(first), Some(last)) = (points_pos.first(), points_pos.last()) {
                    let mut fill_points = points_pos.clone();
                    fill_points.push(Pos2::new(last.x, rect.bottom()));
                    fill_points.push(Pos2::new(first.x, rect.bottom()));
                    painter.add(egui::Shape::convex_polygon(
                        fill_points,
                        color.gamma_multiply(0.15),
                        Stroke::NONE,
                    ));
                }
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
