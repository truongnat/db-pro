# Findings

## P1 — AI prediction command was not wired to runtime

- Evidence: `crates/native-app/src/translate.rs` returned `None` for both prediction commands.
- Failure: the UI marked a prediction request pending, but no provider request was sent and no
  response could arrive in the shipped native app.
- Scope: add a typed runtime command/event path and preserve provider credentials inside runtime.

## P1 — Prediction responses were matched to mutable current state

- Evidence: the UI located a pending request by request id but used the current cursor and buffer
  version when constructing the prediction.
- Failure: a delayed response could be shown at a cursor/version different from the request.
- Scope: carry document id, version, and anchor in the event and reject mismatches.

## P2 — Typing dispatched prediction on every edit

- Evidence: `draw_query_editor` sent `RequestSqlPrediction` directly from `response.changed`.
- Failure: fast typing could enqueue one provider call per keystroke.
- Scope: debounce in `QueryDocument` and cancel the superseded request.
