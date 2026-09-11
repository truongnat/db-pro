# Native AI Agent

## Status

`COMPLETED`

## Goal

Turn the native Agent preview into a database-aware copilot for the Rust/egui
workspace. The first slice must work offline and produce inspectable SQL drafts
without executing mutations.

## Scope

- Add a pure Rust template responder with connection/schema context.
- Add native agent conversation state and a functional composer.
- Render assistant text, SQL drafts and write-risk warnings.
- Insert a proposed SQL draft into the active query document.
- Keep the agent visually inside the database workspace, not as a detached chat.

## Out of scope

- Network LLM calls or provider/API-key storage.
- Automatic query execution or mutation approval.
- MCP/tool execution and autonomous database changes.

## Architecture

`egui composer -> AgentMessage -> pure template responder -> AgentMessage`

The responder receives only non-secret context: driver, connection label,
table names and column names. SQL drafts are data for the query editor; they are
never executed by the Agent action itself. A later provider slice can replace
the responder behind the same message contract in the runtime layer.

## Acceptance criteria

- Empty state offers useful database-aware starter prompts.
- Cmd/Ctrl+Enter and Send produce a user message plus an assistant response.
- Responses can include SQL and Insert places it in the active query tab.
- Mutating drafts are visibly marked as requiring confirmation and never run.
- Pure responder tests cover empty schema, table selection, read-only SQL and mutation warning.
