//! Tests for `../handler.rs` (moved out of the source file).
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
fn query_plan_adapter_sanitizes_costs_and_buffer_counts() {
    let mut canonical = query_node(Vec::new(), Vec::new());
    canonical.startup_cost = Some(-2.0);
    canonical.total_cost = Some(f64::MAX);
    canonical.shared_hit_blocks = Some(f64::INFINITY);
    canonical.shared_read_blocks = Some(-1.0);

    let adapted = PlanNode::from_query_plan(&canonical);

    assert_eq!(adapted.cost_estimate, f32::MAX);
    assert_eq!(adapted.startup_cost, Some(0.0));
    assert_eq!(adapted.total_cost, Some(f32::MAX));
    assert_eq!(adapted.shared_hit_blocks, Some(0));
    assert_eq!(adapted.shared_read_blocks, Some(0));
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
fn query_plan_adapter_truncates_manually_built_deep_trees() {
    let mut canonical = query_node(Vec::new(), Vec::new());
    for _ in 0..db_pro_core::domain::explain_plan::MAX_EXPLAIN_PLAN_DEPTH {
        canonical = query_node(vec![canonical], Vec::new());
    }

    let adapted = PlanNode::from_query_plan(&canonical);
    let mut depth = 1;
    let mut node = &adapted;
    while let Some(child) = node.children.first() {
        depth += 1;
        node = child;
    }

    assert_eq!(depth, db_pro_core::domain::explain_plan::MAX_EXPLAIN_PLAN_DEPTH);
    assert!(node.findings.iter().any(|finding| finding.contains("truncated")));
}

#[test]
fn query_plan_adapter_truncates_wide_trees_at_shared_node_budget() {
    let child = query_node(Vec::new(), Vec::new());
    let canonical = query_node(
        vec![child; db_pro_core::domain::explain_plan::MAX_EXPLAIN_PLAN_NODES],
        vec![db_pro_core::domain::explain_plan::PlanFinding {
            code: "plan.truncated".to_owned(),
            message: db_pro_core::domain::explain_plan::PLAN_TRUNCATION_MESSAGE.to_owned(),
            severity: db_pro_core::domain::explain_plan::PlanFindingSeverity::Warning,
        }],
    );

    let adapted = PlanNode::from_query_plan(&canonical);

    assert_eq!(
        adapted.children.len() + 1,
        db_pro_core::domain::explain_plan::MAX_EXPLAIN_PLAN_NODES
    );
    assert_eq!(
        adapted
            .findings
            .iter()
            .filter(|finding| finding.as_str() == db_pro_core::domain::explain_plan::PLAN_TRUNCATION_MESSAGE)
            .count(),
        1
    );
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
