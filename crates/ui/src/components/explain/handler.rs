use egui::Color32;
use lucide_icons::Icon;

use super::config::{
    EXPLAIN_HOTSPOT_RATIO, EXPLAIN_MIN_BAR_RATIO, EXPLAIN_ROW_SKEW_MAX_RATIO, EXPLAIN_ROW_SKEW_MIN_RATIO,
    EXPLAIN_WARNING_RATIO,
};
use crate::DbProTheme;

#[derive(Debug, Clone)]
pub struct PlanNode {
    pub node_type: String,
    pub relation: Option<String>,
    pub index_name: Option<String>,
    pub cost_estimate: f32,
    pub startup_cost: Option<f32>,
    pub total_cost: Option<f32>,
    /// Display duration; imported per-loop values are aggregated across observed loops.
    pub actual_time_ms: f32,
    /// Display startup duration using the same loop aggregation as total duration.
    pub actual_startup_ms: Option<f32>,
    /// Display row count; imported per-loop values are aggregated across observed loops.
    pub rows_actual: usize,
    /// Planned rows normalized to the observed loop count for comparable skew ratios.
    pub rows_planned: Option<usize>,
    /// Number of observed executions represented by the aggregated runtime metrics.
    pub actual_loops: Option<usize>,
    pub shared_hit_blocks: Option<usize>,
    pub shared_read_blocks: Option<usize>,
    pub is_bottleneck: bool,
    pub findings: Vec<String>,
    pub children: Vec<PlanNode>,
}

impl PlanNode {
    pub fn new(node_type: impl Into<String>, cost: f32, time_ms: f32, rows: usize) -> Self {
        Self {
            node_type: node_type.into(),
            relation: None,
            index_name: None,
            cost_estimate: cost,
            startup_cost: None,
            total_cost: Some(cost),
            actual_time_ms: time_ms,
            actual_startup_ms: None,
            rows_actual: rows,
            rows_planned: None,
            actual_loops: None,
            shared_hit_blocks: None,
            shared_read_blocks: None,
            is_bottleneck: false,
            findings: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Adapts PostgreSQL plan metrics, which report actual rows and time as per-loop averages.
    pub fn from_query_plan(node: &db_pro_core::domain::explain_plan::QueryPlanNode) -> Self {
        let mut nodes_seen = 0;
        Self::from_query_plan_at(node, 0, &mut nodes_seen)
    }

    fn from_query_plan_at(
        node: &db_pro_core::domain::explain_plan::QueryPlanNode,
        depth: usize,
        nodes_seen: &mut usize,
    ) -> Self {
        *nodes_seen += 1;
        let depth_limited =
            depth + 1 >= db_pro_core::domain::explain_plan::MAX_EXPLAIN_PLAN_DEPTH && !node.children.is_empty();
        let mut children = Vec::new();
        let mut truncated = depth_limited;
        if !depth_limited {
            for child in &node.children {
                if *nodes_seen >= db_pro_core::domain::explain_plan::MAX_EXPLAIN_PLAN_NODES {
                    truncated = true;
                    break;
                }
                children.push(Self::from_query_plan_at(child, depth + 1, nodes_seen));
            }
        }
        let is_bottleneck = node.findings.iter().any(|finding| {
            matches!(
                finding.severity,
                db_pro_core::domain::explain_plan::PlanFindingSeverity::Hotspot
            )
        });
        let mut findings: Vec<String> = node.findings.iter().map(|finding| finding.message.clone()).collect();
        let truncation_already_reported = node.findings.iter().any(|finding| finding.code == "plan.truncated");
        if truncated && !truncation_already_reported {
            findings.push(db_pro_core::domain::explain_plan::PLAN_TRUNCATION_MESSAGE.to_owned());
        }
        let actual_loops = normalized_actual_loops(node.actual_loops);
        let loop_count = actual_loops.map(|loops| loops as f64).unwrap_or(1.0);
        let actual_time_ms = node
            .actual_total_ms
            .filter(|time| time.is_finite())
            .map(|time| aggregate_display_time(time, loop_count));
        let actual_rows = node
            .actual_rows
            .filter(|rows| rows.is_finite())
            .or_else(|| node.plan_rows.filter(|rows| rows.is_finite()))
            .unwrap_or(0.0)
            .max(0.0);
        let planned_rows = node
            .plan_rows
            .filter(|rows| rows.is_finite())
            .map(|rows| rows.max(0.0) * loop_count);
        let mut ui_node = Self {
            node_type: node.node_type.clone(),
            relation: node.relation.clone(),
            index_name: node.index_name.clone(),
            cost_estimate: node.total_cost.map(bounded_display_f32).unwrap_or(0.0),
            startup_cost: node.startup_cost.map(bounded_display_f32),
            total_cost: node.total_cost.map(bounded_display_f32),
            actual_time_ms: actual_time_ms.unwrap_or(0.0),
            actual_startup_ms: node
                .actual_startup_ms
                .filter(|time| time.is_finite())
                .map(|time| aggregate_display_time(time, loop_count)),
            rows_actual: aggregate_display_rows(actual_rows, loop_count),
            rows_planned: planned_rows.map(bounded_row_count),
            actual_loops,
            shared_hit_blocks: node.shared_hit_blocks.map(bounded_row_count),
            shared_read_blocks: node.shared_read_blocks.map(bounded_row_count),
            is_bottleneck,
            findings,
            children,
        };
        if actual_time_ms.is_none() {
            // Cost-only plans lack runtime duration; retain the existing cost proxy for display callers.
            ui_node.actual_time_ms = ui_node.cost_estimate;
        }
        ui_node
    }

    pub fn relation(mut self, relation: impl Into<String>) -> Self {
        self.relation = Some(relation.into());
        self
    }

    pub fn index_name(mut self, index_name: impl Into<String>) -> Self {
        self.index_name = Some(index_name.into());
        self
    }

    pub fn bottleneck(mut self, is_bottleneck: bool) -> Self {
        self.is_bottleneck = is_bottleneck;
        self
    }

    pub fn with_child(mut self, child: PlanNode) -> Self {
        self.children.push(child);
        self
    }
}

// PostgreSQL loop counts must be finite positive integers; invalid values use one-pass fallback behavior.
fn normalized_actual_loops(loops: Option<f64>) -> Option<usize> {
    let loops = loops
        .filter(|count| count.is_finite() && *count > 0.0 && count.fract() == 0.0 && *count < usize::MAX as f64)?;
    Some(loops as usize)
}

fn aggregate_display_time(per_loop_ms: f64, loops: f64) -> f32 {
    let total_ms = per_loop_ms.max(0.0) * loops;
    if total_ms.is_infinite() {
        f32::MAX
    } else {
        bounded_display_f32(total_ms)
    }
}

fn bounded_display_f32(value: f64) -> f32 {
    if !value.is_finite() || value <= 0.0 {
        return 0.0;
    }

    // The bound prevents overflow in the legacy f32 PlanNode representation.
    value.min(f32::MAX as f64) as f32
}

fn aggregate_display_rows(per_loop_rows: f64, loops: f64) -> usize {
    let total_rows = per_loop_rows.max(0.0) * loops;
    if total_rows.is_infinite() {
        usize::MAX
    } else {
        bounded_row_count(total_rows)
    }
}

// Legacy `usize` fields saturate explicitly instead of wrapping/truncating huge aggregates.
fn bounded_row_count(rows: f64) -> usize {
    if !rows.is_finite() || rows <= 0.0 {
        return 0;
    }
    if rows >= usize::MAX as f64 {
        return usize::MAX;
    }

    // The bound prevents overflow; the legacy PlanNode API stores row counts as integers.
    rows as usize
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlameBarMetrics {
    pub percentage: f32,
    pub color: Color32,
}

pub fn flame_bar_metrics(node_val: f32, total_val: f32, is_bottleneck: bool, theme: &DbProTheme) -> FlameBarMetrics {
    let percentage = if node_val.is_finite() && total_val.is_finite() && total_val > 0.0 {
        (node_val / total_val).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let color = if percentage > EXPLAIN_HOTSPOT_RATIO || is_bottleneck {
        theme.danger
    } else if percentage > EXPLAIN_WARNING_RATIO {
        theme.warning
    } else {
        theme.success
    };

    FlameBarMetrics { percentage, color }
}

pub fn node_icon_and_color(
    is_bottleneck: bool,
    has_relation: bool,
    has_index: bool,
    theme: &DbProTheme,
) -> (Icon, Color32) {
    if is_bottleneck {
        (Icon::Flame, theme.danger)
    } else if has_index {
        (Icon::KeyRound, theme.accent)
    } else if has_relation {
        (Icon::Table, theme.text_secondary)
    } else {
        (Icon::GitCommitVertical, theme.text_secondary)
    }
}

pub(super) fn should_render_flame_bar(percentage: f32) -> bool {
    percentage > EXPLAIN_MIN_BAR_RATIO
}

pub fn calculate_row_skew(planned: Option<usize>, actual: usize) -> Option<String> {
    if let Some(planned_rows) = planned {
        if planned_rows > 0 && actual > 0 {
            let ratio = (actual as f64) / (planned_rows as f64);
            if !(EXPLAIN_ROW_SKEW_MIN_RATIO..=EXPLAIN_ROW_SKEW_MAX_RATIO).contains(&ratio) {
                return Some(if ratio < EXPLAIN_ROW_SKEW_MIN_RATIO {
                    format!("<{:.1}x rows skew", EXPLAIN_ROW_SKEW_MIN_RATIO)
                } else if ratio > 1.0 {
                    format!("{:.0}x rows skew", ratio)
                } else {
                    format!("{:.1}x rows skew", ratio)
                });
            }
        }
    }
    None
}

/// Formats display-ready row/time values; use `node_runtime_stat_text` to include loop count.
pub fn node_stat_text(
    has_runtime_stats: bool,
    actual_time_ms: f32,
    cost_estimate: f32,
    pct: f32,
    rows: usize,
) -> String {
    if has_runtime_stats {
        actual_node_stat_text(actual_time_ms, pct, rows)
    } else {
        format!("cost {:.1} ({:.0}%) · {} rows", cost_estimate, pct * 100.0, rows)
    }
}

pub(super) fn node_runtime_stat_text(actual_time_ms: f32, pct: f32, rows: usize, loops: Option<usize>) -> String {
    let stats = actual_node_stat_text(actual_time_ms, pct, rows);
    let Some(loops) = loops else {
        return stats;
    };

    let loop_label = if loops == 1 { "loop" } else { "loops" };
    format!("{stats} · {loops} {loop_label}")
}

fn actual_node_stat_text(actual_time_ms: f32, pct: f32, rows: usize) -> String {
    format!("{:.2}ms ({:.0}%) · {} rows", actual_time_ms, pct * 100.0, rows)
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
