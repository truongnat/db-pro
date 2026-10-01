# AgentPrimitives design

`mod.rs` is the public entry point for the family enums, task data, and
re-exports. `ui.rs` paints context chips, status badges, and tool-call cards.
`approval_ui.rs` paints the execution approval card. `disclosure_ui.rs` paints
thinking and plan disclosures and exposes SQL action labels/icons.
`handler.rs` owns typed visual/action mappings and pure progress/geometry
calculations. `config.rs` contains component-local dimensions and type sizes.

## Event flow

`ContextChip` returns an egui `Response`; when removable, its close target has a
label of `Remove context: {label}` and the caller decides whether to remove the
associated context item. `StatusBadge` is display-only and publishes its text
as a label.

`ToolCall` and `AgentThinking` receive a mutable expansion flag from the caller.
Their headers respond to pointer clicks and focused Enter/Space activation,
update that flag, and expose collapsing-header accessibility metadata. The UI
then paints the input/output or thought body only while expanded. `AgentPlan`
derives the completed count and progress ratio from its borrowed task slice;
only `Completed` contributes to progress. `AgentSqlActionKind` maps each typed
kind to a stable label and icon without initiating a SQL operation.

`ExecutionApproval` lays out the title, impact, SQL preview, and Run/Preview/
Cancel buttons. A button click becomes an `ExecutionApprovalAction` returned to
the caller. Risk level changes the visual emphasis, with Preview primary and
Run destructive for high/destructive risk. The component itself does not run
SQL or enforce authorization, confirmation, or database safety rules. For
example, `table_editor_view` handles `Run` by calling its own `submit_ddl()`.

## Rendering cost and constraints

Each visible family member paints its supplied text and controls during the
current egui frame. Collapsed ToolCall and AgentThinking bodies are not painted;
AgentPlan paints every task on every visible frame and does not virtualize large
task lists. Previews and task details are borrowed caller-provided text without
a component maximum length, so callers should bound exceptionally large values.
The shared `DbProTheme` supplies semantic colors; local geometry stays in
`config.rs`.
