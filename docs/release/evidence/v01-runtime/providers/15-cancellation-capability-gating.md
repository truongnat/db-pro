# Query-cancellation capability truthfulness (brief §13)

- Session: `v01-runtime` runtime-evidence run, 2026-09-14
- Repo HEAD when inspected: `8e1d63f` (branch `main`, worktree clean)
- Verdict: **CORRECTLY GATED — no bug found. PostgreSQL is not offered as cancellable.**
- Scope: this run did **not** implement PostgreSQL cancellation, as instructed. Nothing in this
  area was changed.

## 1. The capability values in code

| Provider | Value | Evidence |
|---|---|---|
| PostgreSQL | `cancel = false` | `crates/core/src/domain/capabilities.rs:132`, with the rationale comment at `:129-131` — *"The connector does not yet expose PostgreSQL's wire-level cancellation primitive; do not advertise best-effort task cancellation as provider cancellation."* |
| SQLite | `cancel = true` | `crates/core/src/domain/capabilities.rs:187`, rationale at `:185-186` — *"SQLite cancellation interrupts the active VM and waits for the actor to acknowledge that it is ready for reuse."* |

Both values are pinned by unit tests in the same file:

- `crates/core/src/domain/capabilities.rs:244` — `postgres_capabilities_are_complete` asserts
  `assert!(!caps.query.cancel)`.
- `crates/core/src/domain/capabilities.rs:257` — `sqlite_capabilities_have_expected_gaps` asserts
  `assert!(caps.query.cancel)`.

The values therefore match the brief's model (`postgres.cancel = false`, `sqlite.cancel = true`)
and cannot silently drift without a test failing.

## 2. Is the UI's Stop/cancel action capability-gated?

**Yes — on every path that can reach the shipping UI.** Four independent layers:

| Layer | Location | Behaviour |
|---|---|---|
| Capability lookup | `crates/ui/src/app.rs:927-937` (`query_capabilities`) | Maps the active query driver string to `DatabaseCapabilities`. An unrecognised driver returns `None`, so `is_some_and(...)` is `false` — **fail-closed**: an unknown provider is treated as non-cancellable, never as cancellable. |
| Run/Stop button | `crates/ui/src/query_view.rs:117` | `let cancel_supported = self.query_capabilities().is_some_and(\|c\| c.query.cancel);` |
| Run/Stop button — rendering | `crates/ui/src/query_view.rs:118-129` | While a query is running, the control is a **"Stop"** button only when `cancel_supported`; otherwise it renders a non-actionable **"Running…"** button with the tooltip *"Query running (cancellation is unsupported by this provider)"* (`:122-125`). PostgreSQL therefore never gets a Stop affordance. |
| Run/Stop button — activation | `crates/ui/src/query_view.rs:130-136` | A click only calls `self.cancel_query(request_id)` inside `if cancel_supported`; the `else` branch sets `runtime_message = "Query cancellation is not supported for this provider"` (`:135`). |
| Esc keyboard path | `crates/ui/src/events.rs:1145-1151` | The Esc handler repeats the same `query_capabilities().is_some_and(\|c\| c.query.cancel)` guard at `:1147` before calling `cancel_query` at `:1148`, with the same refusal message at `:1150`. The keyboard shortcut cannot bypass the button's gate. |

A runtime-level backstop exists as well, in case a cancel command ever reaches the backend:
`crates/infrastructure/src/postgres/connector.rs:199-202` implements `cancel` as an explicit
`DbError::Unsupported("PostgreSQL query cancellation is not available for this connector")` — it
does **not** report a best-effort success. That contract is pinned by the test
`postgres_cancel_is_explicitly_unsupported` at `crates/infrastructure/src/postgres/connector.rs:714-722`,
which asserts the error is `DbError::Unsupported` and that PostgreSQL *"must not claim unsupported
cancellation succeeded"*. The runtime surfaces such a failure as an explicit
`query cancellation failed: …` event (`crates/runtime/src/worker.rs:1538-1550`) rather than
pretending it worked.

## 3. Other cancel-shaped controls in the codebase (checked for a bypass)

- `crates/ui/src/components/sql_editor.rs:13-20, 55-65` — `SqlEditorToolbar` can render a
  destructive **"Cancel"** button when constructed with `is_running = true`, emitting
  `SqlEditorAction::CancelQuery`. It is **not** part of the shipping query view
  (`crates/ui/src/query_view.rs:906` builds `SqlEditor`, not `SqlEditorToolbar`). Its only
  consumer is the component gallery
  (`crates/ui/src/component_gallery_view.rs:2185`), which constructs it with
  `SqlEditorToolbar::new(false, true, theme)` — `is_running = false`, so the button is never
  rendered there — and whose action `match` (`:2186-2200`) has **no** `CancelQuery` arm, falling
  through to `_ => {}`. So even if it were rendered, activating it would do nothing.
- The `cancel` locals at `crates/ui/src/query_view.rs:719, 767` are dialog-dismissal booleans
  ("Save Query As" / "Unsaved query"), unrelated to query execution.

No path offers PostgreSQL cancellation, and no path offers a cancellation that silently no-ops
while claiming success.

## 4. Verdict and disposition

**VERIFIED — the capability model and the UI gating agree.** PostgreSQL `cancel = false` and
SQLite `cancel = true` are implemented, tested by unit tests, and enforced in the UI on both the
button and the keyboard path, with a fail-closed default for unknown drivers and an explicit
`Unsupported` error at the connector boundary.

No correctness bug, no P0/P1/P2 finding, no code change. Consistent with the existing release
statement (*"Query cancellation: **`Unsupported`** for PostgreSQL — capability-gated
(`postgres.cancel = false`)"*, `docs/release/0.1.0-readiness.md`, Providers table).

## 5. Not verified in this item

- **No interactive verification.** Whether the "Running…" button and its tooltip *look* right on
  screen was not observed — this run has no window-server access (see `README.md` in this
  directory). The claim above is a **code-and-test-level** verification only.
- A live end-to-end "cancel a running SQLite query in the UI" interaction was not performed, for
  the same reason. SQLite cancellation's automated coverage is unchanged by this run.
