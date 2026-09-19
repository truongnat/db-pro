# Native Core Architecture — Findings

## F1 — `DbProApp` owns multiple feature states

Evidence: `crates/ui/src/app.rs` contains shell, query, grid, schema,
connection, agent, monitoring, security, workspace and task state in one
struct. Views are split into files, but still mutate the same aggregate.

Impact: feature changes have a wide compile and regression surface; state
transitions cannot be tested independently from the application composition
root.

Severity: P1 architectural correctness/maintainability risk.

## F2 — Connection dialog state was embedded in the composition root

Evidence: the dialog draft, validation result, diagnostics and modal lifecycle
were individual `DbProApp` fields used by connection views and unrelated shell
code.

Fix in progress: these fields now live in `ConnectionDialogState`, with explicit
open/edit/duplicate/new/test/close transitions.

Severity: P1 boundary leak.

## F3 — Runtime event dispatch is centralized before feature migration

Evidence: `events.rs` drains all `UiEvent` values and mutates `DbProApp`
directly. This is a useful current seam, but it must dispatch into feature
reducers as aggregates are extracted.

Severity: P1 follow-up.

## F4 — Connection lifecycle was coupled to every consumer

Evidence: active connection identity, pending request identity, failure
markers and request flags were individual `DbProApp` fields read by explorer,
query, navigation, task, schema and event code.

Fix in `13dedb61`: those lifecycle values now have one owner,
`ConnectionLifecycleState`, with explicit clearing behavior for pending
requests and connection errors. Fix in `d9ff6b00`: the saved-connection read
model now has one owner, `ConnectionCatalogState`, including replacement and
lookup behavior.

Severity: P1 boundary leak, partially resolved.

## F5 — Shell layout and navigation state was coupled to the composition root

Evidence: activity selection, workspace tab, panel visibility/geometry,
welcome lifecycle and pending navigation were individual `DbProApp` fields
consumed by the shell, sidebar, query, explorer and workspace actions.

Fix in `a483000a`: those values now have one owner, `WorkspaceShellState`;
panel geometry is clamped through state setters and shell tests cover defaults
and boundary clamping.

Severity: P1 boundary leak, resolved for the shell slice.

## F6 — Query document lifecycle was coupled to the composition root

Evidence: open documents, active document selection, selected text, save/close
request maps and Save As/dirty-close state were individual `DbProApp` fields
read by query, agent, workspace and runtime event code.

Fix in `0bde3a5f`: those values now have one owner,
`QuerySessionState`. Query output/result state intentionally remains separate
for the next query slice.

Severity: P1 boundary leak, resolved for document lifecycle.
