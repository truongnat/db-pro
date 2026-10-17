# Findings — #346

Confirmed the guard exists at schema_workbench_actions.rs:59-63. Failed plans set `preview_error` and clear `preview_sql` in `plan_workbench_action`, so the dialog cannot open for failed/empty plans.
