# AgentPrimitives API

Types are re-exported from `db_pro_ui::components` and the
`components::agent_primitives` module path.

## Components and results

- `ContextChip::new(kind, label, theme).removable(bool).show(ui) -> Response`
  displays a context label. `kind` is one of Connection, Database, Schema,
  Table, Editor, or File. When removable, the returned response describes the
  close hit target; inspect `clicked()` to remove the associated item in caller
  state. Without removal, it describes the chip.
- `StatusBadge::new(text, variant, theme).show(ui) -> Response` displays text
  using `StatusBadgeVariant::{Active, Running, Success, Warning, Destructive,
  Archived, Draft}`. It has no action result.
- `ToolCall::new(tool_name, status, input_preview, &mut expanded, theme)` accepts
  `ToolCallStatus::{Running, Success, Failed}`. Optional `.duration(text)` and
  `.output_preview(text)` add details; `.show(ui) -> Response` toggles the
  caller-owned expansion state by click or focused Enter/Space.
- `ExecutionApproval::new(title, impact, sql_preview, risk, theme).show(ui)`
  returns `Option<ExecutionApprovalAction>` with `Run`, `Preview`, or `Cancel`.
  `RiskLevel` is Low, Medium, High, or Destructive. Risk changes presentation;
  it does not impose a safety policy or execute the preview.
- `AgentThinking::new(thought, &mut expanded, theme)` supports optional
  `.duration(text)`, `.step_count(count)`, and `.is_active(bool)`; `.show(ui) ->
  Response` toggles the borrowed expansion state by click or focused Enter/Space.
- `AgentTaskItem::new(title, status).with_detail(detail)` builds a task using
  `AgentTaskStatus::{Pending, Running, Completed, Failed, Skipped}`.
  `AgentPlan::new(title, &tasks, theme).show(ui) -> Response` displays all tasks
  and completed progress. Empty plans have a zero progress ratio.
- `AgentSqlActionKind::{Explain, Optimize, FixError, GenerateMigration,
  DescribeSchema, ConvertDialect}` provides `.label()` and `.icon()` metadata.

## State, limits, and safety

Only ToolCall and AgentThinking mutate expansion state, and only in response to
header activation. Context removal, plan/task updates, status updates, and all
provider actions remain caller-owned. `ExecutionApprovalAction::Run` means the
user clicked Run; callers must independently enforce confirmation, validation,
authorization, and database safety before executing SQL. `Preview` and `Cancel`
likewise report intent only.

Task progress counts only `Completed`; Failed and Skipped do not count as
completed. AgentPlan renders every supplied task without virtualization.
Tool/SQL/thought/task strings have no component-defined maximum or truncation
policy. Long values may wrap and enlarge the containing layout; bound previews
in the caller where needed. Runtime keyboard, focus, contrast, and narrow-window
behavior remain subject to native verification.
