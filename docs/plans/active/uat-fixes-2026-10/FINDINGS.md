# UAT Findings — 2026-10-23/24

Seven UAT agents drove the live `db-pro-native` debug build via orca
computer-use against "Sample E-Commerce (SQLite)" and "Xe Lạc Hồng (PostgreSQL)"
@127.0.0.1:5433 — intended one-GUI-agent-at-a-time (orca exposes a single
selectable instance); one overlap occurred: UatNavigator had a state-dump batch
in flight while UatConnections was driving, so its first observations may be
contaminated (they were discarded and re-taken after the hold). 48 scenario
checks → 46 PASS, 1 PARTIAL, 1 BLOCKED (Copy Cell Value click eaten by Chrome
PiP — inconclusive).

Severity follows AGENTS.md: P0 / P1 / P2 only. What the agents called "P3" is
recorded here as **minor P2** (polish end of the same tier).

## Defects (dedup across agents)

| ID | Sev | Defect | Evidence |
|---|---|---|---|
| UAT-1 | P2 | Popover outside-click **click-through**: dismissing the explorer filter-options popover by clicking out also activates the row underneath → unintended table tab open. `egui::Popup` forwards outside clicks. | UatNavigator D1, reproduced twice |
| UAT-2 | P2 | egui ID collision: "⚠ Second use of widget ID 367E" debug overlay persists over the table editor toolbar on every table surface. **Debug build only.** | UatTable — `/tmp/uat-table/01_orders_data.png` |
| UAT-3 | P2 | Floating context menus absent from the accessibility tree (render fine visually). VoiceOver operability NOT verified — this is an AX-omission, not proven VoiceOver failure. | UatConnections + UatNavigator |
| UAT-4 | P2 | Applied query row cap is not disclosed: toolbar limit 100 on a large result reports "Query completed · 100 rows" with no indication that the query was capped. Provider fetches return at most `max_rows` without probing another row, so exact-N cannot prove truncation; disclose the applied cap, not a truncation claim. | UatQuery check 7; provider trace in VERIFICATION.md |
| UAT-5 | P2 | Tab strip does not auto-scroll to a newly activated tab; active tab's × is clipped at the strip edge — a newly opened tab can be invisible and unclosable by mouse. | UatTools D1+D2 |
| UAT-6 | minor P2 | Status bar shows the globally active connection, not the active document's bound connection (`shell_chrome_view.rs:draw_statusbar` uses `active_connection()`; `query_status_bar_surface_view.rs` uses the doc binding). Two agents confirmed independently. | UatQuery P3 + UatCopilotSettings |
| UAT-7 | minor P2 | Switching active connection disconnects the other ("Disconnected — click to connect"); tab headers keep stale connection badges. Possibly by-design single-session; flagged. | UatPgTools |
| UAT-8 | minor P2 | PG query context picker labels schema `main` instead of `public`. | UatQuery |
| UAT-9 | minor P2 | Autocomplete popup can stay floating after Run; Cmd+A intermittently no-ops while popup open (typed text appends). Correlated with popup-open; may be tooling-contingent. | UatQuery |
| UAT-10 | minor P2 | Empty Port field validates as range error "Use a port number between 1 and 65535" instead of "Port is required". | UatConnections |
| UAT-11 | minor P2 | Validation toast text duplicated in a11y tree (inline node + container node) — double announcement risk. | UatConnections |
| UAT-12 | minor P2 | SAVED QUERIES lists 9 entries all named "Query 1" — indistinguishable. | UatTools D3 |
| UAT-13 | minor P2 | Settings mode hides all chrome with no Back/Close affordance inside the surface (only rail switch exits). | UatCopilotSettings |
| UAT-14 | minor P2 | Navigator list virtualization: rows beyond fold unreachable via AX scroll (Quick Open works). | UatTable |

## Not defects (verified correct)
- Read-only badge/disabled edit on SQLite: connection has `readonly=true` → `can_mutate=false`. By design.
- Copilot "AI provider not configured" + "Configure an API key" toast: correct no-key path, no hang.
- `weird"name` table loads; DDL escapes to `"weird""name"`. Unicode `Ünïcödé Üsers` fine.
- Views correctly omit table-only tabs; PG `users` DDL emits schema-qualified quoting + partial-index `WHERE`.
- Connection switching disconnect semantics (UAT-7) may be intended — needs product decision.

## NOT TESTED
- Hub tool actions (Compare diff, Workbench Apply DDL, Security role ops, Transfer execution, Task scheduling) — smoke-open only, no disposable data.
- Mutations/transactions on PG (writable conn confirmed by enabled Add Row; not clicked — real DB).
- Backup/Restore execution (needs pg_dump), real AI provider, VoiceOver itself.
- Capture heights >838pt (macOS visible-area clamp).
