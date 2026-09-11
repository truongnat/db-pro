# Findings

## Initial audit — 2026-09-11

| ID | Severity | Evidence | Decision |
|---|---|---|---|
| NID-001 | P1 | `crates/ui/src/app.rs` has fixed activity/sidebar/workspace orchestration; no persisted panel widths or bottom output surface. | Build the workbench state first because every later surface depends on its layout and focus model. |
| NID-002 | P1 | `crates/ui/src/navigation_view.rs` renders bounded object groups but not a complete connection/database/schema hierarchy or context actions. | Add compact hierarchy/context menus without changing domain services. |
| NID-003 | P1 | `crates/ui/src/query_view.rs` has one result surface and no explain/messages output state. | Add an output-tab state model and route explain through the existing runtime API. |
| NID-004 | P1 | `crates/ui/src/table_editor_view.rs` dispatches update/delete immediately from grid actions. | Introduce local staged changes, then reuse existing typed mutation commands for apply. |
| NID-005 | P2 | `crates/ui/src/palette_view.rs` advertises Ctrl+P while `goal-1.md` requires Cmd/Ctrl+K. | Make K canonical and retain P only as a compatibility shortcut. |
| NID-006 | P1 | `crates/ui/src/app.rs::agent_context` only includes connection/driver/tables/columns. | Extend context with selected object, SQL, result/error summaries; keep destructive actions confirmation-gated. |

The earlier native foundation, provider adapters and runtime facades remain reusable and are intentionally not rewritten.

## Progress — 2026-09-11

| Area | Evidence now present | Remaining verification |
|---|---|---|
| P0 shell | Persistent sidebar/Agent/output dimensions, resizable panels, native modifier helper, bottom output panel | Native Xvfb walkthrough pass |
| P1 explorer | Database → schema nesting, bounded groups, connection/table/view/function context menus | PostgreSQL 977-table introspection and SQLite table interaction pass |
| P2 query | Results/Messages/Explain/History tabs; typed Explain command through runtime; compact More menu | PostgreSQL query/Explain and SQLite query walkthrough pass |
| P3 grid | Update/delete/insert stage locally; dirty rows; sequential Apply and Discard; reload guarded while dirty | Automated state/identity tests pass; provider-specific mutation commands remain capability-gated |
| P4 object workspace | Structure/Data/Indexes/Foreign Keys/Constraints/Dependencies/DDL tabs | PostgreSQL `sql_diff` and SQLite `notes` Structure/Data workspaces pass |
| P5 palette | Cmd/Ctrl+K, dynamic table Quick Open, Explain/Export actions | Native keyboard routing regression and runtime shell pass |
| P6 Agent | Context includes schema/table/columns/SQL/result/plan/error and contextual Ask Agent actions | Offline provider response and safe draft flow pass |

### Follow-up — 2026-09-11

- The explorer no longer hard-codes `public`/`main` as its only schema. `IntrospectResult.schemas` now crosses the runtime/native boundary, the selected schema is stateful, and table/view/trigger/function lists are filtered to it.
- Switching connections clears the previous schema snapshot before the new introspection request, so loading/error states cannot display objects from the wrong connection.
- Agent context and Quick Open now reuse the active-schema table/column scope instead of leaking objects from other schemas.
- PostgreSQL routines are now presented as one compact Functions / Procedures group with routine type visible; the existing provider adapter already returns both kinds.
- The native process exposes an X11 window when launched with `WINIT_UNIX_BACKEND=x11`; the default Wayland launch is not discoverable by the current desktop automation path.
- Future provider-specific surfaces not exposed by this vertical slice remain explicitly out of scope (user/session/lock/sequence/type management). Exposed Explain, backup/restore and routine paths now consult core capabilities.
- Global shortcuts now defer to focused native text inputs; Cmd/Ctrl+K remains global while panel/query shortcuts no longer steal Ctrl+B/F/Enter from form fields.
- Connection mutations now immediately enqueue an Explorer refresh, and pending runtime work requests bounded repainting so async connection/schema state reaches the native UI without relying on a blinking text caret.
- Native walkthrough evidence covers PostgreSQL and SQLite connection/schema/query/result/object flows, Agent offline draft, Explain and SQL error recovery; the plan completion gate is satisfied.
