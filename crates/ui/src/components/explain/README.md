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
- `PlanNode`: Hierarchical tree representation of explain plan operations with costs, timings, row counts, cache hits, and performance findings.
- `FlameBarMetrics`, `flame_bar_metrics`: Metric calculation for relative cost/time allocation bars.
- `calculate_row_skew`: Helper to detect significant planner estimate mismatches vs actual row executions.
- `node_icon_and_color`: Iconography and color resolution based on node properties (bottleneck, index scan, table scan, commit).

## Architecture

- `mod.rs`: Re-exports public structs and functions.
- `config.rs`: Layout metrics, bar dimensions, indent scales, and badge spacing tokens.
- `handler.rs`: Pure calculations for flame metrics, row skew, iconography, and text formatting.
- `ui.rs`: egui rendering and layout passes for trees and node items.
