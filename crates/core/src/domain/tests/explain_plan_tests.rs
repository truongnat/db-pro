//! Tests for `../explain_plan.rs` (moved out of the source file).
use super::*;
use serde_json::json;

#[test]
fn parses_estimate_only_postgres_plan() {
    let value = json!([{
        "Plan": {
            "Node Type": "Hash Join",
            "Total Cost": 100.0,
            "Plan Rows": 10.0,
            "Plans": [
                {
                    "Node Type": "Seq Scan",
                    "Relation Name": "orders",
                    "Total Cost": 80.0,
                    "Plan Rows": 5000.0
                },
                {
                    "Node Type": "Hash",
                    "Total Cost": 12.0,
                    "Plan Rows": 10.0,
                    "Plans": [{
                        "Node Type": "Index Scan",
                        "Relation Name": "customers",
                        "Index Name": "customers_pkey",
                        "Total Cost": 8.0,
                        "Plan Rows": 10.0
                    }]
                }
            ]
        }
    }]);
    let plan = parse_postgres_explain_json(&value).expect("parse");
    assert!(!plan.has_runtime_stats);
    assert_eq!(plan.root.node_type, "Hash Join");
    assert_eq!(plan.root.children.len(), 2);
    assert!(plan.findings.iter().any(|f| f.code == "plan.estimate"));
    assert!(plan
        .root
        .children
        .iter()
        .any(|c| c.node_type == "Seq Scan" && !c.findings.is_empty()));
}

#[test]
fn deep_plans_are_truncated_with_a_finding() {
    let mut nested = json!({"Node Type": "Seq Scan"});
    for _ in 0..MAX_EXPLAIN_PLAN_DEPTH {
        nested = json!({"Node Type": "Nested Loop", "Plans": [nested]});
    }
    let plan_json = json!([{"Plan": nested}]);

    let mut plan = parse_postgres_explain_json(&plan_json).expect("parse");
    let mut depth = 1;
    let mut node = &plan.root;
    while let Some(child) = node.children.first() {
        depth += 1;
        node = child;
    }

    assert_eq!(depth, MAX_EXPLAIN_PLAN_DEPTH);
    assert!(plan.findings.iter().any(|finding| finding.code == "plan.truncated"));
    assert!(plan
        .root
        .findings
        .iter()
        .any(|finding| finding.code == "plan.truncated"));

    apply_heuristics(&mut plan);
    assert_eq!(
        plan.findings
            .iter()
            .filter(|finding| finding.code == "plan.truncated")
            .count(),
        1
    );
    assert_eq!(
        plan.root
            .findings
            .iter()
            .filter(|finding| finding.code == "plan.truncated")
            .count(),
        1
    );
}

#[test]
fn wide_plans_are_truncated_at_node_budget() {
    let child_plans = (0..=MAX_EXPLAIN_PLAN_NODES)
        .map(|_| json!({"Node Type": "Seq Scan"}))
        .collect::<Vec<_>>();
    let value = json!({"Plan": {"Node Type": "Append", "Plans": child_plans}});

    let plan = parse_postgres_explain_json(&value).expect("parse");
    let mut count = 0;
    let mut pending = vec![&plan.root];
    while let Some(node) = pending.pop() {
        count += 1;
        pending.extend(node.children.iter());
    }

    assert_eq!(count, MAX_EXPLAIN_PLAN_NODES);
    assert!(plan.findings.iter().any(|finding| finding.code == "plan.truncated"));

    let mut oversized = plan.clone();
    let extra_child = oversized.root.children[0].clone();
    oversized.root.children.push(extra_child);
    apply_heuristics(&mut oversized);
    assert_eq!(oversized.root.children.len() + 1, MAX_EXPLAIN_PLAN_NODES);
    assert!(oversized
        .findings
        .iter()
        .any(|finding| finding.code == "plan.truncated"));
}

#[test]
fn display_total_time_aggregates_root_per_loop_time_when_execution_total_is_missing() {
    let value = json!([{
        "Plan": {
            "Node Type": "Nested Loop",
            "Actual Total Time": 2.5,
            "Actual Loops": 4.0,
            "Total Cost": 20.0
        }
    }]);
    let plan = parse_postgres_explain_json(&value).expect("parse");

    assert_eq!(plan.display_total_ms(), 10.0);
}

#[test]
fn display_total_time_ignores_invalid_loop_counts() {
    let value = json!([{
        "Plan": {
            "Node Type": "Nested Loop",
            "Actual Total Time": 2.5,
            "Actual Loops": 0.0
        }
    }]);
    let plan = parse_postgres_explain_json(&value).expect("parse");

    assert_eq!(plan.display_total_ms(), 2.5);
}

#[test]
fn heuristics_ignore_nonfinite_provider_metrics() {
    let mut plan = QueryPlan {
        root: QueryPlanNode {
            node_type: "Hash Join".to_owned(),
            relation: None,
            alias: None,
            index_name: None,
            startup_cost: Some(-1.0),
            total_cost: Some(f64::NAN),
            plan_rows: Some(f64::NAN),
            actual_startup_ms: None,
            actual_total_ms: Some(f64::INFINITY),
            actual_rows: Some(f64::NAN),
            actual_loops: Some(f64::NAN),
            shared_hit_blocks: None,
            shared_read_blocks: None,
            children: Vec::new(),
            findings: Vec::new(),
        },
        has_runtime_stats: true,
        planning_time_ms: None,
        execution_time_ms: None,
        findings: Vec::new(),
    };

    apply_heuristics(&mut plan);

    assert!(plan.root.findings.is_empty());
    assert_eq!(plan.display_total_ms(), 0.0);
}

#[test]
fn parses_analyze_plan_and_flags_misestimate() {
    let value = json!([{
        "Execution Time": 12.5,
        "Planning Time": 0.4,
        "Plan": {
            "Node Type": "Seq Scan",
            "Relation Name": "big",
            "Total Cost": 50.0,
            "Plan Rows": 10.0,
            "Actual Total Time": 12.0,
            "Actual Rows": 2000.0,
            "Actual Loops": 1.0,
            "Shared Hit Blocks": 40.0
        }
    }]);
    let plan = parse_postgres_explain_json(&value).expect("parse");
    assert!(plan.has_runtime_stats);
    assert_eq!(plan.execution_time_ms, Some(12.5));
    assert!(plan.findings.iter().any(|f| f.code == "plan.runtime"));
    assert!(plan.root.findings.iter().any(|f| f.code == "plan.row-misestimate"));
}

#[test]
fn parses_sqlite_explain_query_plan_rows_into_a_tree() {
    let value = serde_json::json!([
        { "id": 2, "parent": 0, "detail": "SCAN employees" },
        { "id": 5, "parent": 2, "detail": "SEARCH orders USING INDEX idx_orders_user (user_id=?)" }
    ]);

    let plan = parse_sqlite_explain_json(&value).expect("parse");

    assert_eq!(plan.root.node_type, "SCAN");
    assert_eq!(plan.root.relation.as_deref(), Some("employees"));
    assert_eq!(plan.root.children.len(), 1);
    assert_eq!(plan.root.children[0].node_type, "SEARCH");
    assert_eq!(plan.root.children[0].relation.as_deref(), Some("orders"));
    assert_eq!(plan.root.children[0].index_name.as_deref(), Some("idx_orders_user"));
    assert!(!plan.has_runtime_stats);
}

#[test]
fn sqlite_explain_groups_multiple_roots_under_a_plan_node() {
    let value = serde_json::json!([
        { "id": 1, "parent": 0, "detail": "SCAN a" },
        { "id": 4, "parent": 0, "detail": "SCAN b" }
    ]);

    let plan = parse_sqlite_explain_json(&value).expect("parse");

    assert_eq!(plan.root.node_type, "Plan");
    assert_eq!(plan.root.children.len(), 2);
}

#[test]
fn sqlite_explain_rejects_non_row_arrays() {
    assert!(parse_sqlite_explain_json(&serde_json::json!([])).is_none());
    assert!(parse_sqlite_explain_json(&serde_json::json!([{ "Plan": {} }])).is_none());
}
