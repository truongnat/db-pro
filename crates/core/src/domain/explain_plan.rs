//! Canonical query plan model parsed from provider EXPLAIN output (#215).
//!
//! Humans and Agent tools share this model — do not maintain a second EXPLAIN parser.

use serde_json::Value;

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
    /// Total time preference: execution time → root actual → root cost proxy.
    pub fn display_total_ms(&self) -> f64 {
        self.execution_time_ms
            .or(self.root.actual_total_ms)
            .or(self.root.total_cost)
            .unwrap_or(0.0)
    }
}

/// Parse PostgreSQL `EXPLAIN (FORMAT JSON)` / `EXPLAIN (ANALYZE, FORMAT JSON)` output.
pub fn parse_postgres_explain_json(value: &Value) -> Option<QueryPlan> {
    let plan_obj = extract_postgres_plan_object(value)?;
    let root_value = plan_obj.get("Plan")?;
    let root = parse_postgres_node(root_value)?;
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

fn parse_postgres_node(value: &Value) -> Option<QueryPlanNode> {
    let obj = value.as_object()?;
    let node_type = obj
        .get("Node Type")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_owned();
    let children = obj
        .get("Plans")
        .and_then(|v| v.as_array())
        .map(|items| items.iter().filter_map(parse_postgres_node).collect::<Vec<_>>())
        .unwrap_or_default();
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
    let root_cost = plan.root.total_cost.unwrap_or(0.0).max(0.0);
    let root_time = plan.root.actual_total_ms.unwrap_or(0.0).max(0.0);
    apply_node_heuristics(&mut plan.root, root_cost, root_time, plan.has_runtime_stats);
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

fn apply_node_heuristics(node: &mut QueryPlanNode, root_cost: f64, root_time: f64, has_runtime: bool) {
    node.findings.clear();
    let lower = node.node_type.to_ascii_lowercase();

    if lower.contains("seq scan") {
        let rows = node.actual_rows.or(node.plan_rows).unwrap_or(0.0);
        if rows >= 1_000.0 || node.total_cost.unwrap_or(0.0) > root_cost * 0.25 {
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
            node.actual_total_ms.unwrap_or(0.0) / root_time
        } else if root_cost > 0.0 {
            node.total_cost.unwrap_or(0.0) / root_cost
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
        if let (Some(planned), Some(actual)) = (node.plan_rows, node.actual_rows) {
            if planned > 0.0 && actual > 0.0 {
                let ratio = (actual / planned).max(planned / actual);
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
    }

    let cost_share = if root_cost > 0.0 {
        node.total_cost.unwrap_or(0.0) / root_cost
    } else {
        0.0
    };
    let time_share = if has_runtime && root_time > 0.0 {
        node.actual_total_ms.unwrap_or(0.0) / root_time
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

    for child in &mut node.children {
        apply_node_heuristics(child, root_cost, root_time, has_runtime);
    }
}

fn collect_findings(node: &QueryPlanNode) -> Vec<PlanFinding> {
    let mut out = node.findings.clone();
    for child in &node.children {
        out.extend(collect_findings(child));
    }
    out
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
