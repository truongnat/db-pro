# Query Editor Intelligence Layer

## Goal

Make query execution, explain, schema completion, and AI SQL prediction deterministic per
`QueryDocument`, with stale-response protection and a real async runtime boundary for prediction.

## Scope

- Carry document id, document version, and cursor anchor through prediction requests/events.
- Route prediction through the native translator and async runtime worker.
- Cancel superseded prediction requests and debounce typing-triggered requests.
- Keep execution and explain results associated with the originating document.
- Preserve the existing native Query Workspace UI and completion behavior while adding focused tests.

## Non-goals

- Agent chat UI or a Query Workspace redesign.
- Database schema changes.
- Replacing the existing schema introspection provider.

## Acceptance criteria

- A prediction request contains document context and cannot mutate a newer document version.
- Native runtime executes prediction asynchronously through the configured provider.
- Typing does not issue an AI request until the debounce window expires.
- Switching document connection/schema cancels and invalidates pending prediction state.
- Query and explain completion update only the originating document.
- PostgreSQL and SQLite continue to use their existing execution capabilities.
