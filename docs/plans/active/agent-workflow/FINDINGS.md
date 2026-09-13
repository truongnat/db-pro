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

## P2 — Provider tool-call seam is still pending

The current provider contract returns a draft, not a typed tool call or stream.
The runtime now exposes a typed `ExecuteAgentTool` command/event pair and an
`AgentToolExecutor` that calls the existing schema/query services. The current
provider still returns a draft rather than typed tool calls, so provider-driven
multi-step continuation and compact-panel state remain the next orchestration
slice; the executor is intentionally usable independently of that provider.
