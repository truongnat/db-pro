# AgentPrimitives

`AgentPrimitives` is the native egui component family used by agent surfaces and the component gallery. It provides context chips, status badges, tool-call details, execution approval, thinking details, execution plans, and SQL action metadata.

## Public API

- `ContextChip` and `ContextChipKind` render connection, database, schema, table, editor, and file context. With `.removable(true)`, `show` returns the close hit-target `Response`; callers should remove the matching item when it is clicked.
- `StatusBadge` and `StatusBadgeVariant` render semantic status pills.
- `ToolCall` and `ToolCallStatus` render a collapsible tool invocation with input/output previews.
- `ExecutionApproval`, `ExecutionApprovalAction`, and `RiskLevel` render a SQL approval card with typed action outcomes.
- `AgentThinking` renders a collapsible thought stream with optional duration and step count.
- `AgentPlan`, `AgentTaskItem`, and `AgentTaskStatus` render a task checklist and progress bar.
- `AgentSqlActionKind` supplies stable labels and icons for SQL assistance actions.

## Behavior

- Builder methods preserve the legacy borrowing API: `ToolCall` and `AgentThinking` mutate the caller's expansion boolean when their header is clicked or focused and activated with Enter/Space. Their headers expose collapsing-header accessibility metadata.
- Tool-call status, risk labels, thinking titles, task progress counts/ratio, task visuals, and SQL action labels/icons are selected by typed pure handler functions.
- Colors are resolved from `DbProTheme`; component-local geometry remains in `config.rs`.
- `ExecutionApproval::show` returns `Run`, `Preview`, or `Cancel` when the matching button is clicked. High/destructive risks visually make Preview primary and Run destructive; the caller remains responsible for enforcing its confirmation/safety policy. In `table_editor_view`, `Run` is routed through the caller's `submit_ddl()` path.
- `AgentPlan` counts only `Completed` tasks toward progress, including the existing `0.0` ratio for an empty plan.

## Layering

`mod.rs` is the public entry point and keeps the public enums/data model stable. `ui.rs` owns egui allocation, measurement, layout, widget construction, and painting. `handler.rs` owns typed mappings, action decisions, and pure geometry/progress calculations. `config.rs` contains only local primitive geometry and typography values.

The UI flow is intentionally explicit: egui supplies measurements and click signals; handlers convert those signals into typed outcomes; UI modules apply the state change and paint the resulting component. `approval_ui.rs` and `disclosure_ui.rs` keep the independently shaped surfaces small. This keeps behavior testable without moving rendering into the handler.

See [DESIGN.md](DESIGN.md) for event flow and rendering costs and [API.md](API.md)
for builders, state ownership, results, and limits.

## Usage

```rust
use db_pro_ui::components::{
    AgentPlan, AgentTaskItem, AgentTaskStatus, ContextChip, ContextChipKind,
};

ContextChip::new(ContextChipKind::Database, "production", theme).show(ui);

let tasks = vec![
    AgentTaskItem::new("Inspect schema", AgentTaskStatus::Completed),
    AgentTaskItem::new("Prepare migration", AgentTaskStatus::Running),
];
AgentPlan::new("Migration workflow", &tasks, theme).show(ui);
```
