# Findings

## P2 — Existing Agent prototype is UI/runtime-shaped

The repository already contains an offline draft provider and a remote
`RunAgent` command, but its context/message types are duplicated between UI and
runtime and it returns prose/drafts rather than typed actions. The first slice
adds a core-owned contract without removing the working compatibility path.

## P2 — Runtime evidence is pending

This foundation does not claim provider or native UI runtime evidence. The
existing provider key and viewport evidence gap from Query Editor remains
separate; Agent Workflow needs its own PostgreSQL/SQLite and native checks.

## P2 — Existing safety classifier is intentionally reused

The agent permission decision maps `StatementSafety` from the core policy
classifier. It does not introduce a UI classifier. Unknown/incomplete SQL is
confirmation-gated rather than silently treated as read-only.

## P2 — Multi-step typed provider tool orchestration verified

The runtime parses Responses API function calls into typed `AgentToolCall` values,
continues with structured tool outputs, and runs a bounded multi-step `AgentRunOrchestrator`.
Confirmation pauses are resumed without creating synthetic run IDs, and `GetCurrentQuery`
is the explicit recovery path for stale document versions. The native Agent panel renders
tool activity, streaming text, and diff previews with approval controls.

## P2 — Agent query results remain ephemeral by design

Agent query execution returns bounded tool summaries (`sample_rows` up to 20, total count)
to the provider and does not replace the visible `QueryDocument` result workspace.
Surfacing an agent result in the UI requires user action via "Open sample in Results",
ensuring background reasoning never unexpectedly alters the active user workspace.

## P2 — Independent database cancellation semantics

- **PostgreSQL**: When an agent query is cancelled via Stop or tab closure, cancellation issues `query_api.cancel(&connection_id)` and aborts the async task. In PostgreSQL, this triggers backend socket disconnect/cancel and frees the connection cleanly.
- **SQLite**: SQLite queries run synchronously on a dedicated connection worker. Cancellation aborts the runtime command receiver and awaits VM step recovery. The UI immediately transitions to `Cancelled` without hanging.

