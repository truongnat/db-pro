# Explain Plan Component

Execution plan and performance diagnostics tree component for database queries (`EXPLAIN` and `EXPLAIN ANALYZE`).

## Usage

```rust
let root_node = PlanNode::new("Seq Scan", 120.5, 4.2, 1000)
    .relation("users")
    .bottleneck(true);

ExplainPlanTree::new(&root_node, 15.4, theme)
    .has_runtime_stats(true)
    .planning_time(Some(0.85))
    .show(ui);
```

## Public API

- `ExplainPlanTree`: Builder-style egui widget for rendering query plan hierarchy with cost/execution flame bars and hotspots.
- `PlanNode`: Hierarchical tree model for costs, timings, row counts, cache hits, findings, and conversion from the canonical query-plan model.
- `FlameBarMetrics`, `flame_bar_metrics`: Metric calculation for relative cost/time allocation bars.
- `calculate_row_skew`: Helper to detect significant planner estimate mismatches vs actual row executions.
- `node_icon_and_color`: Iconography and color resolution based on node properties (bottleneck, index scan, table scan, commit).

## Behavior

- PostgreSQL `EXPLAIN ANALYZE` actual time and row counts are averages per loop. `PlanNode::from_query_plan` aggregates actual and planned rows plus actual time across valid positive integer loop counts, and the UI displays that count beside the totals. Invalid counts use a one-pass fallback; oversized aggregates saturate the legacy display types.
- Parsing, adaptation, and rendering enforce shared depth/node budgets (`MAX_EXPLAIN_PLAN_DEPTH`, `MAX_EXPLAIN_PLAN_NODES`) and show one shared truncation warning instead of traversing unbounded input.
- The current query-output bridge consumes PostgreSQL EXPLAIN JSON; SQLite plan normalization is not added by this component migration.
- Cost-only plans keep estimated cost/rows and do not invent runtime measurements. Row-skew labels use a lower-bound marker when a very small ratio would otherwise round upward misleadingly.
- Flame bars compare each node's metric with the plan total; node times are inclusive, so percentages are per-node comparisons and are not mutually exclusive or expected to sum to 100%.
- Painter-rendered EXPLAIN ANALYZE, hotspot, skew, and flame-bar cues expose matching egui accessibility labels; the tree is read-only.

## Architecture

- `mod.rs`: Re-exports public structs and functions.
- `config.rs`: Commented local layout metrics, bar/badge dimensions, and behavior thresholds; shared spacing, typography, and colors remain in tokens/theme.
- `handler.rs`: Plan-node model/builders and query-plan adaptation, plus pure flame metrics, row-skew, visibility, icon, and text decisions.
- `ui.rs`: egui rendering and layout passes for trees and node items.
