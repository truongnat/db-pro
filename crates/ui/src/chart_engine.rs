use bigdecimal::BigDecimal;
use std::collections::BTreeMap;
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
        crate::UiCell::Json(v) => crate::components::truncate_ellipsis(v, 20),
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
