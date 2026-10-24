# UAT Findings — 2026-10-23/24

Seven UAT agents drove the live `db-pro-native` debug build via orca
computer-use against "Sample E-Commerce (SQLite)" and "Xe Lạc Hồng (PostgreSQL)"
@127.0.0.1:5433 — serialized (one GUI agent at a time) because orca exposes a
single selectable instance. 48 scenario checks → 46 PASS, 1 PARTIAL, 1 BLOCKED
(Copy Cell Value click eaten by Chrome PiP — inconclusive).

Severity follows AGENTS.md: P0 / P1 / P2 only. The "P3" labels below are the
UAT agents' own wording for the lowest tier — treat them as **minor P2**.

## Defects (dedup across agents)

| ID | Sev | Defect | Evidence |
|---|---|---|---|
| UAT-1 | P2 | Popover outside-click **click-through**: dismissing the explorer filter-options popover by clicking out also activates the row underneath → unintended table tab open. `egui::Popup` forwards outside clicks. | UatNavigator D1, reproduced twice |
| UAT-2 | P2 | egui ID collision: "⚠ Second use of widget ID 367E" debug overlay persists over the table editor toolbar on every table surface. **Debug build only.** | UatTable — `/tmp/uat-table/01_orders_data.png` |
| UAT-3 | P2 | Floating context menus absent from the accessibility tree (render fine visually). VoiceOver operability NOT verified — this is an AX-omission, not proven VoiceOver failure. | UatConnections + UatNavigator |
| UAT-4 | P2 | Row-limit truncation invisible: limit 100 on a 256-row result reports "Query completed · 100 rows" with no "of N / truncated" indicator. | UatQuery check 7 |
| UAT-5 | P2 | Tab strip does not auto-scroll to a newly activated tab; active tab's × is clipped at the strip edge — a newly opened tab can be invisible and unclosable by mouse. | UatTools D1+D2 |
| UAT-6 | P3 | Status bar shows the globally active connection, not the active document's bound connection (`shell_chrome_view.rs:draw_statusbar` uses `active_connection()`; `query_status_bar_surface_view.rs` uses the doc binding). Two agents confirmed independently. | UatQuery P3 + UatCopilotSettings |
| UAT-7 | P3 | Switching active connection disconnects the other ("Disconnected — click to connect"); tab headers keep stale connection badges. Possibly by-design single-session; flagged. | UatPgTools |
| UAT-8 | P3 | PG query context picker labels schema `main` instead of `public`. | UatQuery |
| UAT-9 | P3 | Autocomplete popup can stay floating after Run; Cmd+A intermittently no-ops while popup open (typed text appends). Correlated with popup-open; may be tooling-contingent. | UatQuery |
| UAT-10 | P3 | Empty Port field validates as range error "Use a port number between 1 and 65535" instead of "Port is required". | UatConnections |
| UAT-11 | P3 | Validation toast text duplicated in a11y tree (inline node + container node) — double announcement risk. | UatConnections |
| UAT-12 | P3 | SAVED QUERIES lists 9 entries all named "Query 1" — indistinguishable. | UatTools D3 |
| UAT-13 | P3 | Settings mode hides all chrome with no Back/Close affordance inside the surface (only rail switch exits). | UatCopilotSettings |
| UAT-14 | P3 | Navigator list virtualization: rows beyond fold unreachable via AX scroll (Quick Open works). | UatTable |
| UAT-15 | note | "Drop table" gallery AlertDialog — tooltip of × close overlaps header text (user-reported earlier; cosmetic). | Owner screenshot |

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
