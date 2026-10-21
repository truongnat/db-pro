## Analysis
The nonminimal boolean `!pending.as_ref().is_some_and(...)` was introduced in a recent commit. Replacing it with `pending.as_ref().is_none_or(...)` satisfies Clippy while preserving identical logic semantics.
