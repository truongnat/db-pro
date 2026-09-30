//! Canonical query plan model parsed from provider EXPLAIN output (#215).
//!
//! Humans and Agent tools share this model — do not maintain a second EXPLAIN parser.

use serde_json::Value;

/// Maximum plan-tree nesting before parser, adapter, or renderer truncates it.
pub const MAX_EXPLAIN_PLAN_DEPTH: usize = 128;
/// Maximum total plan nodes parsed from one EXPLAIN document.
pub const MAX_EXPLAIN_PLAN_NODES: usize = 10_000;
/// Shared user-facing notice emitted when either plan-tree limit is reached.
pub const PLAN_TRUNCATION_MESSAGE: &str = "Plan truncated to stay within EXPLAIN safety limits";

/// One node in a provider-normalized execution plan tree.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryPlanNode {
    pub node_type: String,
    pub relation: Option<String>,
    pub alias: Option<String>,
    pub index_name: Option<String>,
    pub startup_cost: Option<f64>,
    pub total_cost: Option<f64>,
    pub plan_rows: Option<f64>,
    pub actual_startup_ms: Option<f64>,
    pub actual_total_ms: Option<f64>,
    pub actual_rows: Option<f64>,
    pub actual_loops: Option<f64>,
    pub shared_hit_blocks: Option<f64>,
    pub shared_read_blocks: Option<f64>,
    pub children: Vec<QueryPlanNode>,
    pub findings: Vec<PlanFinding>,
}

/// Deterministic advisor finding attached to a plan node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanFinding {
    pub code: String,
    pub message: String,
    pub severity: PlanFindingSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanFindingSeverity {
    Info,
    Warning,
    Hotspot,
}

/// Parsed explain document with estimate-vs-runtime metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryPlan {
    pub root: QueryPlanNode,
    /// True when Actual Total Time (or equivalent) is present — EXPLAIN ANALYZE.
    pub has_runtime_stats: bool,
    pub planning_time_ms: Option<f64>,
    pub execution_time_ms: Option<f64>,
    pub findings: Vec<PlanFinding>,
}

impl QueryPlan {
    /// Total time preference: execution time → root actual across loops → root cost proxy.
    pub fn display_total_ms(&self) -> f64 {
        let loops = self
            .root
            .actual_loops
            .filter(|count| count.is_finite() && *count > 0.0 && count.fract() == 0.0 && *count < usize::MAX as f64)
            .unwrap_or(1.0);

        self.execution_time_ms
            .filter(|time| time.is_finite())
            .map(|time| time.max(0.0))
            .or_else(|| {
                self.root
                    .actual_total_ms
                    .filter(|time| time.is_finite())
                    .map(|time_per_loop| (time_per_loop.max(0.0) * loops).min(f64::MAX))
            })
            .or_else(|| {
                self.root
                    .total_cost
                    .filter(|cost| cost.is_finite())
                    .map(|cost| cost.max(0.0))
            })
            .unwrap_or(0.0)
    }
}

/// Parse PostgreSQL `EXPLAIN (FORMAT JSON)` / `EXPLAIN (ANALYZE, FORMAT JSON)` output.
pub fn parse_postgres_explain_json(value: &Value) -> Option<QueryPlan> {
    let plan_obj = extract_postgres_plan_object(value)?;
    let root_value = plan_obj.get("Plan")?;
    let mut truncated = false;
    let mut nodes_parsed = 0;
    let root = parse_postgres_node(root_value, 0, &mut nodes_parsed, &mut truncated)?;
    let has_runtime_stats = root.actual_total_ms.is_some() || plan_obj.get("Execution Time").is_some();
    let planning_time_ms = json_f64(plan_obj.get("Planning Time"));
    let execution_time_ms = json_f64(plan_obj.get("Execution Time"));
    let mut plan = QueryPlan {
        root,
        has_runtime_stats,
        planning_time_ms,
        execution_time_ms,
        findings: Vec::new(),
    };
    apply_heuristics(&mut plan);
    if truncated {
        let finding = plan_truncation_finding();
        plan.root.findings.push(finding.clone());
        plan.findings.push(finding);
    }
    Some(plan)
}

/// Parse a pretty-printed JSON string produced by the runtime bridge.
pub fn parse_postgres_explain_str(raw: &str) -> Option<QueryPlan> {
    let value: Value = serde_json::from_str(raw).ok()?;
    parse_postgres_explain_json(&value)
}

fn extract_postgres_plan_object(value: &Value) -> Option<&Value> {
    match value {
        Value::Array(items) => items.first(),
        Value::Object(_) => Some(value),
        _ => None,
    }
}

fn parse_postgres_node(
    value: &Value,
    depth: usize,
    nodes_parsed: &mut usize,
    truncated: &mut bool,
) -> Option<QueryPlanNode> {
    if *nodes_parsed >= MAX_EXPLAIN_PLAN_NODES {
        *truncated = true;
        return None;
    }
    *nodes_parsed += 1;

    let obj = value.as_object()?;
    let node_type = obj
        .get("Node Type")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_owned();
    let child_plans = obj.get("Plans").and_then(|v| v.as_array());
    let mut children = Vec::new();
    if let Some(child_plans) = child_plans {
        if depth + 1 >= MAX_EXPLAIN_PLAN_DEPTH && !child_plans.is_empty() {
            *truncated = true;
        } else {
            for child in child_plans {
                if *nodes_parsed >= MAX_EXPLAIN_PLAN_NODES {
                    *truncated = true;
                    break;
                }
                if let Some(child) = parse_postgres_node(child, depth + 1, nodes_parsed, truncated) {
                    children.push(child);
                }
            }
        }
    }
    Some(QueryPlanNode {
        node_type,
        relation: json_string(obj.get("Relation Name")),
        alias: json_string(obj.get("Alias")),
        index_name: json_string(obj.get("Index Name")),
        startup_cost: json_f64(obj.get("Startup Cost")),
        total_cost: json_f64(obj.get("Total Cost")),
        plan_rows: json_f64(obj.get("Plan Rows")),
        actual_startup_ms: json_f64(obj.get("Actual Startup Time")),
        actual_total_ms: json_f64(obj.get("Actual Total Time")),
        actual_rows: json_f64(obj.get("Actual Rows")),
        actual_loops: json_f64(obj.get("Actual Loops")),
        shared_hit_blocks: json_f64(obj.get("Shared Hit Blocks")),
        shared_read_blocks: json_f64(obj.get("Shared Read Blocks")),
        children,
        findings: Vec::new(),
    })
}

fn json_string(value: Option<&Value>) -> Option<String> {
    value.and_then(|v| v.as_str()).map(str::to_owned)
}

fn json_f64(value: Option<&Value>) -> Option<f64> {
    value.and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
}

/// Deterministic heuristics: seq scans, misestimates, cost/time hotspots, sorts/hashes.
pub fn apply_heuristics(plan: &mut QueryPlan) {
    let previously_truncated = plan.findings.iter().any(|finding| finding.code == "plan.truncated")
        || plan
            .root
            .findings
            .iter()
            .any(|finding| finding.code == "plan.truncated");
    let root_cost = nonnegative_finite(plan.root.total_cost);
    let root_time = nonnegative_finite(plan.root.actual_total_ms);
    let mut truncated = false;
    let mut nodes_seen = 0;
    apply_node_heuristics(
        &mut plan.root,
        root_cost,
        root_time,
        plan.has_runtime_stats,
        0,
        &mut nodes_seen,
        &mut truncated,
    );
    if truncated || previously_truncated {
        plan.root.findings.push(plan_truncation_finding());
    }
    plan.findings = collect_findings(&plan.root);
    if plan.has_runtime_stats {
        plan.findings.insert(
            0,
            PlanFinding {
                code: "plan.runtime".into(),
                message: "Plan includes runtime stats from EXPLAIN ANALYZE (query was executed)".into(),
                severity: PlanFindingSeverity::Info,
            },
        );
    } else {
        plan.findings.insert(
            0,
            PlanFinding {
                code: "plan.estimate".into(),
                message: "Estimate-only EXPLAIN — costs are planner estimates, not measured runtime".into(),
                severity: PlanFindingSeverity::Info,
            },
        );
    }
}

fn nonnegative_finite(value: Option<f64>) -> f64 {
    value.filter(|value| value.is_finite()).unwrap_or(0.0).max(0.0)
}

fn bounded_ratio(numerator: f64, denominator: f64) -> f64 {
    let maximum = f64::MAX / 100.0;
    (numerator / denominator).min(maximum)
}

fn apply_node_heuristics(
    node: &mut QueryPlanNode,
    root_cost: f64,
    root_time: f64,
    has_runtime: bool,
    depth: usize,
    nodes_seen: &mut usize,
    truncated: &mut bool,
) {
    *nodes_seen += 1;
    node.findings.clear();
    let lower = node.node_type.to_ascii_lowercase();

    if lower.contains("seq scan") {
        let rows = nonnegative_finite(node.actual_rows).max(nonnegative_finite(node.plan_rows));
        if rows >= 1_000.0 || nonnegative_finite(node.total_cost) > root_cost * 0.25 {
            node.findings.push(PlanFinding {
                code: "plan.seq-scan".into(),
                message: format!(
                    "Sequential scan on {} may dominate cost — consider an index if filters are selective",
                    node.relation.as_deref().unwrap_or("relation")
                ),
                severity: PlanFindingSeverity::Warning,
            });
        }
    }

    if lower.contains("sort") || lower.contains("hash") {
        let share = if has_runtime && root_time > 0.0 {
            bounded_ratio(nonnegative_finite(node.actual_total_ms), root_time)
        } else if root_cost > 0.0 {
            bounded_ratio(nonnegative_finite(node.total_cost), root_cost)
        } else {
            0.0
        };
        if share >= 0.35 {
            node.findings.push(PlanFinding {
                code: "plan.sort-hash-hotspot".into(),
                message: format!(
                    "{} accounts for ~{:.0}% of plan cost/time",
                    node.node_type,
                    share * 100.0
                ),
                severity: PlanFindingSeverity::Hotspot,
            });
        }
    }

    if has_runtime {
        let planned = nonnegative_finite(node.plan_rows);
        let actual = nonnegative_finite(node.actual_rows);
        if planned > 0.0 && actual > 0.0 {
            let ratio = bounded_ratio(actual, planned).max(bounded_ratio(planned, actual));
            if ratio >= 10.0 {
                node.findings.push(PlanFinding {
                    code: "plan.row-misestimate".into(),
                    message: format!(
                        "Row estimate misestimate: planned {planned:.0} vs actual {actual:.0} (×{ratio:.1})"
                    ),
                    severity: PlanFindingSeverity::Warning,
                });
            }
        }
    }

    let cost_share = if root_cost > 0.0 {
        bounded_ratio(nonnegative_finite(node.total_cost), root_cost)
    } else {
        0.0
    };
    let time_share = if has_runtime && root_time > 0.0 {
        bounded_ratio(nonnegative_finite(node.actual_total_ms), root_time)
    } else {
        0.0
    };
    if (cost_share >= 0.5 || time_share >= 0.5) && !node.findings.iter().any(|f| f.code == "plan.sort-hash-hotspot") {
        node.findings.push(PlanFinding {
            code: "plan.expensive-node".into(),
            message: format!("{} is an expensive node in this plan", node.node_type),
            severity: PlanFindingSeverity::Hotspot,
        });
    }

    if depth + 1 >= MAX_EXPLAIN_PLAN_DEPTH && !node.children.is_empty() {
        node.children.clear();
        *truncated = true;
        return;
    }

    let mut processed_children = 0;
    while processed_children < node.children.len() && *nodes_seen < MAX_EXPLAIN_PLAN_NODES {
        apply_node_heuristics(
            &mut node.children[processed_children],
            root_cost,
            root_time,
            has_runtime,
            depth + 1,
            nodes_seen,
            truncated,
        );
        processed_children += 1;
    }
    if processed_children < node.children.len() {
        node.children.truncate(processed_children);
        *truncated = true;
    }
}

fn collect_findings(root: &QueryPlanNode) -> Vec<PlanFinding> {
    let mut findings = Vec::new();
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        findings.extend(node.findings.iter().cloned());
        pending.extend(node.children.iter().rev());
    }
    findings
}

fn plan_truncation_finding() -> PlanFinding {
    PlanFinding {
        code: "plan.truncated".to_owned(),
        message: PLAN_TRUNCATION_MESSAGE.to_owned(),
        severity: PlanFindingSeverity::Warning,
    }
}

#[cfg(test)]
mod tests {
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
}
