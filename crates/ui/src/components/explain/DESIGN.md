# Explain design

`mod.rs` is the facade for `PlanNode`, `ExplainPlanTree`, metric helpers, and the public `EXPLAIN_*` presentation constants. `handler.rs` builds/adapts the public plan model, bounds imported metrics, aggregates PostgreSQL per-loop measurements, derives row-skew text/icons/flame-bar metrics, and recognizes truncation. `ui.rs` lays out and paints a read-only scrollable plan with accessible metadata for painter-rendered rows and badges. `config.rs` owns tree-specific dimensions and thresholds.

## Data and event flow

The query-output bridge currently supplies PostgreSQL EXPLAIN JSON through the canonical core model. `PlanNode::from_query_plan` converts that model to display data; it is not a general SQL parser. The tree receives a root and caller-provided total metric, then selects cost or runtime labels according to `has_runtime_stats`. Planning time is optional. The viewer has no mutation actions; clicking/expanding is not required to interpret it.

PostgreSQL actual rows and time are averages per loop. Valid positive integer loop counts are aggregated for display; invalid loop counts use a one-pass fallback, and overflowing display aggregates saturate. Cost-only plans retain their estimated values and do not invent runtime measurements. Node flame percentages are per-node comparisons against a total; inclusive node timings can overlap, so percentages need not sum to 100%.

## Bounds and rendering cost

The imported plan adapter and renderer use shared limits of 128 levels and 10,000 nodes. When input exceeds either bound, one truncation finding is shown instead of unbounded descent. Rendering work is proportional to visible plan nodes within that budget; nested node indentation increases with depth. Both-axis scrolling keeps wide/deep trees inspectable, though deep indentation reduces content width.

## Provider and accessibility limits

Current query-output conversion is PostgreSQL EXPLAIN JSON. This contract does not claim SQLite plan normalization; SQLite remains unsupported/pending for this conversion path. Runtime badges, flame bars, hotspot/skew cues and plan rows have accessible text metadata. The plan remains read-only, and caller/application code owns query execution and confirmation for EXPLAIN ANALYZE.
