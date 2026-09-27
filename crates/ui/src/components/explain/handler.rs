use crate::DbProTheme;
use egui::Color32;
use lucide_icons::Icon;

use super::config::{
    EXPLAIN_HOTSPOT_RATIO, EXPLAIN_MIN_BAR_RATIO, EXPLAIN_ROW_SKEW_MAX_RATIO, EXPLAIN_ROW_SKEW_MIN_RATIO,
    EXPLAIN_WARNING_RATIO,
};

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
        let is_bottleneck = node.findings.iter().any(|finding| {
            matches!(
                finding.severity,
                db_pro_core::domain::explain_plan::PlanFindingSeverity::Hotspot
            )
        });
        let findings = node.findings.iter().map(|finding| finding.message.clone()).collect();
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
            cost_estimate: node.total_cost.unwrap_or(0.0) as f32,
            startup_cost: node.startup_cost.map(|cost| cost as f32),
            total_cost: node.total_cost.map(|cost| cost as f32),
            actual_time_ms: actual_time_ms.unwrap_or(0.0),
            actual_startup_ms: node
                .actual_startup_ms
                .filter(|time| time.is_finite())
                .map(|time| aggregate_display_time(time, loop_count)),
            rows_actual: aggregate_display_rows(actual_rows, loop_count),
            rows_planned: planned_rows.map(bounded_row_count),
            actual_loops,
            shared_hit_blocks: node.shared_hit_blocks.map(|blocks| blocks.max(0.0) as usize),
            shared_read_blocks: node.shared_read_blocks.map(|blocks| blocks.max(0.0) as usize),
            is_bottleneck,
            findings,
            children: node.children.iter().map(Self::from_query_plan).collect(),
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
    (per_loop_ms.max(0.0) * loops).min(f32::MAX as f64) as f32
}

fn aggregate_display_rows(per_loop_rows: f64, loops: f64) -> usize {
    bounded_row_count(per_loop_rows.max(0.0) * loops)
}

// Legacy `usize` fields saturate explicitly instead of wrapping/truncating huge aggregates.
fn bounded_row_count(rows: f64) -> usize {
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
mod tests {
    use super::*;

    fn query_node(
        children: Vec<db_pro_core::domain::explain_plan::QueryPlanNode>,
        findings: Vec<db_pro_core::domain::explain_plan::PlanFinding>,
    ) -> db_pro_core::domain::explain_plan::QueryPlanNode {
        db_pro_core::domain::explain_plan::QueryPlanNode {
            node_type: "Seq Scan".to_owned(),
            relation: Some("users".to_owned()),
            alias: None,
            index_name: Some("users_idx".to_owned()),
            startup_cost: Some(0.1),
            total_cost: Some(30.0),
            plan_rows: Some(20.0),
            actual_startup_ms: Some(0.2),
            actual_total_ms: Some(1.25),
            actual_rows: Some(10.0),
            actual_loops: Some(1.0),
            shared_hit_blocks: Some(3.0),
            shared_read_blocks: Some(1.0),
            children,
            findings,
        }
    }

    #[test]
    fn test_flame_bar_metrics() {
        let theme = DbProTheme::light();
        let m_hotspot = flame_bar_metrics(50.0, 100.0, false, &theme);
        assert_eq!(m_hotspot.percentage, 0.5);
        assert_eq!(m_hotspot.color, theme.danger);

        let m_warn = flame_bar_metrics(30.0, 100.0, false, &theme);
        assert_eq!(m_warn.percentage, 0.3);
        assert_eq!(m_warn.color, theme.warning);

        let m_ok = flame_bar_metrics(10.0, 100.0, false, &theme);
        assert_eq!(m_ok.percentage, 0.1);
        assert_eq!(m_ok.color, theme.success);

        let m_forced_bottleneck = flame_bar_metrics(5.0, 100.0, true, &theme);
        assert_eq!(m_forced_bottleneck.color, theme.danger);
    }

    #[test]
    fn test_flame_bar_metrics_handles_non_finite_and_non_positive_values() {
        let theme = DbProTheme::light();

        for (node_val, total_val) in [
            (f32::NAN, 100.0),
            (f32::INFINITY, 100.0),
            (50.0, f32::NAN),
            (50.0, f32::INFINITY),
            (-50.0, 100.0),
            (50.0, 0.0),
            (50.0, -100.0),
        ] {
            let metrics = flame_bar_metrics(node_val, total_val, false, &theme);
            assert_eq!(metrics.percentage, 0.0, "inputs: {node_val:?}, {total_val:?}");
            assert!(metrics.percentage.is_finite(), "percentage must remain finite");
        }
    }

    #[test]
    fn query_plan_adapter_aggregates_per_loop_metrics_and_preserves_metadata() {
        use db_pro_core::domain::explain_plan::{PlanFinding, PlanFindingSeverity};

        let child = query_node(Vec::new(), Vec::new());
        let hotspot = PlanFinding {
            code: "HOTSPOT".to_owned(),
            message: "High cost".to_owned(),
            severity: PlanFindingSeverity::Hotspot,
        };
        let mut canonical = query_node(vec![child], vec![hotspot]);
        canonical.actual_loops = Some(4.0);

        let adapted = PlanNode::from_query_plan(&canonical);

        assert_eq!(adapted.node_type, "Seq Scan");
        assert_eq!(adapted.relation.as_deref(), Some("users"));
        assert_eq!(adapted.actual_time_ms, 5.0);
        assert_eq!(adapted.actual_startup_ms, Some(0.8));
        assert_eq!(adapted.rows_actual, 40);
        assert_eq!(adapted.rows_planned, Some(80));
        assert_eq!(adapted.actual_loops, Some(4));
        assert!(adapted.is_bottleneck);
        assert_eq!(adapted.findings, vec!["High cost".to_owned()]);
        assert_eq!(adapted.children.len(), 1);
        assert_eq!(adapted.children[0].index_name.as_deref(), Some("users_idx"));
    }

    #[test]
    fn query_plan_adapter_saturates_extreme_time_and_row_aggregates() {
        let mut canonical = query_node(Vec::new(), Vec::new());
        canonical.actual_total_ms = Some(f64::MAX);
        canonical.actual_startup_ms = Some(f64::MAX);
        canonical.actual_loops = Some(2.0);
        let bounded_time = PlanNode::from_query_plan(&canonical);
        assert_eq!(bounded_time.actual_time_ms, f32::MAX);
        assert_eq!(bounded_time.actual_startup_ms, Some(f32::MAX));

        canonical = query_node(Vec::new(), Vec::new());
        canonical.actual_loops = Some((usize::MAX / 4) as f64);
        let bounded_rows = PlanNode::from_query_plan(&canonical);
        assert_eq!(bounded_rows.rows_actual, usize::MAX);
        assert_eq!(bounded_rows.rows_planned, Some(usize::MAX));
    }

    #[test]
    fn query_plan_adapter_ignores_invalid_loop_counts() {
        for invalid_loops in [-2.0, 0.0, 1.5, f64::INFINITY] {
            let mut canonical = query_node(Vec::new(), Vec::new());
            canonical.actual_loops = Some(invalid_loops);

            let adapted = PlanNode::from_query_plan(&canonical);

            assert_eq!(adapted.actual_loops, None);
            assert_eq!(adapted.actual_time_ms, 1.25);
            assert_eq!(adapted.rows_actual, 10);
            assert_eq!(adapted.rows_planned, Some(20));
        }
    }

    #[test]
    fn query_plan_adapter_distinguishes_missing_and_zero_runtime_time() {
        let mut canonical = query_node(Vec::new(), Vec::new());
        canonical.actual_total_ms = Some(0.0);
        let zero_runtime = PlanNode::from_query_plan(&canonical);
        assert_eq!(zero_runtime.actual_time_ms, 0.0);

        canonical.actual_total_ms = None;
        let cost_only = PlanNode::from_query_plan(&canonical);
        assert_eq!(cost_only.actual_time_ms, 30.0);
    }

    #[test]
    fn plan_node_builders_keep_hierarchy_and_relation_metadata() {
        let child = PlanNode::new("Index Scan", 12.0, 1.5, 8)
            .relation("users")
            .index_name("users_id_idx")
            .bottleneck(true);
        let root = PlanNode::new("Nested Loop", 20.0, 2.0, 8).with_child(child);

        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].relation.as_deref(), Some("users"));
        assert_eq!(root.children[0].index_name.as_deref(), Some("users_id_idx"));
        assert!(root.children[0].is_bottleneck);
    }

    #[test]
    fn flame_bar_visibility_uses_its_readability_threshold() {
        assert!(!should_render_flame_bar(EXPLAIN_MIN_BAR_RATIO));
        assert!(should_render_flame_bar(EXPLAIN_MIN_BAR_RATIO + f32::EPSILON));
    }

    #[test]
    fn node_stats_match_estimate_and_runtime_modes() {
        assert_eq!(node_stat_text(true, 1.234, 10.0, 0.5, 8), "1.23ms (50%) · 8 rows");
        assert_eq!(node_stat_text(false, 1.234, 10.0, 0.5, 8), "cost 10.0 (50%) · 8 rows");
        assert_eq!(
            node_runtime_stat_text(5.0, 0.5, 40, Some(4)),
            "5.00ms (50%) · 40 rows · 4 loops"
        );
        assert_eq!(
            node_runtime_stat_text(5.0, 0.5, 10, Some(1)),
            "5.00ms (50%) · 10 rows · 1 loop"
        );
    }

    #[test]
    fn test_calculate_row_skew() {
        assert_eq!(calculate_row_skew(Some(10), 200), Some("20x rows skew".to_string()));
        assert_eq!(calculate_row_skew(Some(100), 5), Some("<0.1x rows skew".to_string()));
        assert_eq!(calculate_row_skew(Some(100), 100), None);
        assert_eq!(calculate_row_skew(None, 100), None);
    }

    #[test]
    fn test_node_icon_and_color() {
        let theme = DbProTheme::light();
        let (icon, color) = node_icon_and_color(true, false, false, &theme);
        assert_eq!(char::from(icon), char::from(Icon::Flame));
        assert_eq!(color, theme.danger);

        let (icon2, color2) = node_icon_and_color(false, false, true, &theme);
        assert_eq!(char::from(icon2), char::from(Icon::KeyRound));
        assert_eq!(color2, theme.accent);

        let (icon3, color3) = node_icon_and_color(false, true, false, &theme);
        assert_eq!(char::from(icon3), char::from(Icon::Table));
        assert_eq!(color3, theme.text_secondary);
    }
}
