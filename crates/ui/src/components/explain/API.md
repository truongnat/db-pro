# Explain API

`PlanNode`, `ExplainPlanTree`, `FlameBarMetrics`, and public helpers are re-exported from `components::explain` and `components`.

## Plan model and tree

`PlanNode::new(node_type, cost, time_ms, rows)` creates a display node; chain `relation(name)`, `index_name(name)`, `bottleneck(bool)`, and `with_child(node)`. Public fields also carry startup/total cost, actual/planned rows and times, loop count, shared cache blocks, findings, and children. `PlanNode::from_query_plan(&QueryPlanNode)` adapts the canonical model with bounded depth/node traversal and PostgreSQL per-loop aggregation.

`ExplainPlanTree::new(&root, total_time_ms, theme).planning_time(Option<f32>).has_runtime_stats(bool).show(ui) -> Response` renders a read-only tree. `has_runtime_stats(false)` displays cost estimates; true displays actual execution metrics and the EXPLAIN ANALYZE badge. Supply the matching total metric (cost or execution time) and `planning_time` only when available.

## Helpers

- `flame_bar_metrics(node_value, total_value, is_bottleneck, &theme) -> FlameBarMetrics` returns a bounded percentage and semantic status color.
- `calculate_row_skew(planned, actual) -> Option<String>` returns a label for substantial estimate mismatch.
- `node_icon_and_color(is_bottleneck, has_relation, has_index, &theme)` selects node icon/color.
- `node_stat_text(has_runtime_stats, actual_time_ms, cost_estimate, percentage, rows)` formats display values.
- `EXPLAIN_*` constants are re-exported for integrations that align custom displays with this widget.

Imported and rendered plans are bounded to 128 levels and 10,000 nodes; excess input is represented by a truncation finding. The query-output bridge currently consumes PostgreSQL EXPLAIN JSON. This API does not provide SQLite plan normalization or execute queries. EXPLAIN ANALYZE can execute the statement, so the caller remains responsible for the application's confirmation policy.
