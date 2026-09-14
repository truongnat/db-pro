# RC1 P2 findings — release dispositions by module (#75–#80)

**Purpose:** the 25 RC1 P2 findings (`QA-P2-01` … `QA-P2-25`,
`docs/plans/active/rc1-full-product-qa/FINDINGS.md:295-477`) were audited once in
`docs/release/evidence/v01-06/06-rc1-p2-dispositions.md` (#74), which established for each row that its
recorded fix targeted the **archived React frontend** and therefore left a *native carry-over* verdict
(`CARRIED_OVER_UNVERIFIED` ×20, `SATISFIED_IN_NATIVE` ×1, `NOT_APPLICABLE_IN_NATIVE`/`NOT_CARRIED` ×3).

What that audit deliberately did **not** produce is a release disposition per row. These sections do,
one module per audit issue, using exactly one of the three release words:

- **`Fix RC1`** — a real defect in the shipping product with a bounded fix; a focused child issue is
  opened and linked.
- **`Accept RC1`** — ships as-is, with the source that makes it safe (or the residual risk) named.
- **`Defer post-v0.1`** — the intent belongs to a later release; recorded so nothing is silently dropped.

**Method.** Every disposition below was decided from the current tree, not from the React-era report:
each row names the native source that carries (or removes) the finding's mechanism, and every cited
path was opened before this document was committed. A row whose native behaviour cannot be settled
from source alone is dispositioned `Accept RC1` **with the unmeasured part stated**, never as "fixed".

**Status:** sections A–F correspond to #75–#80 in that order; each was added by its own commit so the
dispositions are traceable to the audit issue that produced them.

---

## A. Workspace / Shell / navigation (#75)

Scope rows: `QA-P2-01`, `02`, `03`, `04`, `05`, `13`, `14`, `23`; the shell-specific part of `QA-P2-06`
is cross-referenced to §F. The remaining bullets the issue lists (dirty/close guards, Quick
Open/Command Palette shortcuts, navigation/back, stale selected resources) have **no live P2 row**;
that is recorded here rather than left implicit.

| Row | Native mechanism (opened for this audit) | Disposition | Rationale |
|---|---|---|---|
| `QA-P2-01` pinned tab visual/store order diverge | the native workspace has **no pinned-tab concept**: `grep -rn "pinned" crates/ui/src/workspace_view.rs` returns nothing, and the only `pinned` hits under `crates/` are unrelated (`diagram/tests.rs`, `table_view.rs`, runtime internals) | **`Defer post-v0.1`** | the feature whose order could diverge is not in v0.1; workspace/tab persistence itself is already out of scope (`known-limitations.md` LIM-016). Nothing is dropped silently — the finding's subject is absent by scope, not by regression |
| `QA-P2-02` tab context-menu shortcuts hardcoded `Ctrl` on macOS | `DbProApp::primary_modifier_label()` (`crates/ui/src/app.rs:637`) returns `⌘` on macOS and `Ctrl` elsewhere, and is used by **12** call sites (menus, tooltips, palette entries: `explorer_view.rs:38`, `query_view.rs:46`, `navigation_view.rs:13,537,584`, `table_view.rs:5`, `result_grid_view.rs:1080`, `palette_view.rs:34`) | **`Accept RC1`** | the defect mechanism (a hardcoded `Ctrl` label) cannot exist in the native shell: every label is derived from one platform-aware helper. Source-verifiable without a GUI session |
| `QA-P2-03` topbar reserves macOS traffic-light space on every OS | there is no web topbar to reserve space: `grep -rn "titlebar\|traffic\|window_decorations\|fullsize_content" crates/ui/src crates/native-app/src` returns **no matches**; the window chrome is the OS's own eframe window | **`Accept RC1`** | the mechanism requires a browser/Tauri topbar; the native shell has none |
| `QA-P2-04` agent panel not visibly marked Preview/Coming Soon | `crates/ui/src/agent_view.rs:224` renders `badge(ui, "Preview", …)`, conditioned on the agent being in its `"Offline draft"` state (`:223`) | **`Accept RC1`** | the carried intent is visibly satisfied where the panel is in its unconfigured state. **Condition recorded:** the badge is tied to that state, not to every agent surface — an owner may want it always-on, but no finding row asks for that and none is invented here |
| `QA-P2-05` agent panel uses macOS-only shortcut hint | the agent view carries no shortcut hint at all — its only `hint_text` is the API-key placeholder (`agent_view.rs:163`) — and its Enter-key affordance goes through `primary_modifier_pressed` (`:170`) | **`Accept RC1`** | nothing to mislabel; no second label path exists |
| `QA-P2-13` sidebar search scans/renders everything per keystroke | the explorer filter is a single-line `TextEdit` over the **already-loaded** tree (`explorer_view.rs:246`), and the command palette filters a static in-memory item list (`filtered_palette_items`, `palette_view.rs:179`); there is no database-wide search surface in v0.1 | **`Accept RC1`** | the React finding's blast radius (a schema-wide scan per keystroke) has no native equivalent; the work is bounded by what is already loaded and rendered, which is the same bound as `QA-P2-14` |
| `QA-P2-14` explorer mounts every row for large expanded schemas | `explorer_view.rs:209` uses a plain `egui::ScrollArea::vertical()` — immediate mode, no row virtualization | **`Accept RC1`**, residual risk recorded | v0.1's largest shipped fixture is **250 tables** (`fixtures/smoke/large-er/large_er_fixture.sql`, `grep -c "CREATE TABLE"` = 250), which is a few hundred simple tree rows per frame, and the heavier large-schema path (the ER diagram) was optimised and benched in Gate 4. **Not measured:** no timing exists for the explorer at that size — this is an accepted, stated risk, aligned with the V01-05 evidence gap, not a verified pass |
| `QA-P2-23` dirty replacement uses native `window.confirm` | `window.confirm` is a browser/WebView API; the native dialog path is `rfd` + the UI's own modal (`crates/native-app`, `crates/ui/src/components`) | **`Accept RC1`** | `NOT_APPLICABLE_IN_NATIVE`: the defect mechanism cannot exist in an egui/Rust shell |

**Acceptance check for this section:** all eight live Workspace/Shell rows have exactly one
disposition; no `Fix RC1` was needed, so no child issue was spawned. No code was changed by this audit.

**What this section does *not* claim:** it does not claim the native shell behaves correctly for the
rows marked `Accept RC1` on a machine with a window server — it claims the *mechanism* of each finding
is absent or is centralised in source that can be read. Interactive confirmation remains with
`docs/release/0.1.0-manual-smoke.md` (#91).

---

## B. Data Grid / read-update-delete (#76)

Scope rows: `QA-P2-07`, `08`, `09`, `10`, `11`, `12`. The issue's other scope bullets (staged
update/delete edge cases, partial-success revision behaviour, PK/no-PK boundaries, sorting/filtering
UX correctness, stale query/data after schema reassignment, value display) map onto these rows plus
the Gate 5 work already closed; no additional live P2 row exists for them.

| Row | Native mechanism (opened for this audit) | Disposition | Rationale |
|---|---|---|---|
| `QA-P2-07` columns picker double-toggle on checkbox click | the native visibility state is a set (`grid_hidden_columns`) mutated by an explicit hide action (`result_grid_view.rs:212`) and a "show all" reset (`:216`), not a checkbox bound to a toggle | **`Accept RC1`** | the React defect came from two handlers toggling one checkbox; the native picker has one mutation path per action. **Not pinned by a test** — the four layout tests around `:2472-2526` cover persisted layouts and schema-shape normalisation, not single-toggle semantics; recorded as an untested (not verified) part |
| `QA-P2-08` read-only connections still show editable grid affordances | edit affordances derive from **both** conditions: `can_edit_table_rows()` = `can_mutate_active_connection() && table_has_primary_key()` (`crates/ui/src/table_editor_view.rs:1494`), and #62's `ColumnWritePolicy` gates every entry point (cell edit, insert row, duplicate row, paste, context menu) with four policy + four interaction tests | **`Accept RC1`** — positively verified | this was the highest-priority carry-over in the #74 audit ("safety-adjacent"); it is now checked in source and pinned by tests, including `no_primary_key_table_blocks_safe_row_mutations`. The React-era defect (edit UI on a read-only connection) has no native path: the grid is `editable` only when the connection is writable and the table has a PK (`result_grid_view.rs:73`) |
| `QA-P2-09` grid context menu can render off-screen; no desktop menu behaviour | native menus are egui popups (`crates/ui/src/components/overlay.rs:787` `is_context_menu_triggered`, `:825`), positioned and clamped by egui's own popup layer | **`Accept RC1`** | there is no DOM, no absolutely-positioned React portal and no viewport clipping to escape; the mechanism is absent rather than fixed |
| `QA-P2-10` column/shell resize handles mouse-only | column resize is a drag handle; no keyboard equivalent exists (no `keyboard.*resize` path found) | **`Defer post-v0.1`** | a real UX/accessibility gap, but no v0.1 acceptance item requires keyboard resizing and the v0.1 keyboard requirements cover the core flows (run, save, palette, navigation), not resize. Recorded so it is not silently dropped |
| `QA-P2-11` resize drag cleanup not guaranteed on unmount | there is no component unmount lifecycle in immediate mode: drag state is per-frame input state | **`Accept RC1`** | the leak the finding describes needs a mounted/unmounted component with a retained handler; neither exists in egui |
| `QA-P2-12` large query-result sorting synchronous on main thread | `filtered_sorted_indexes` (`crates/ui/src/result_grid.rs:97`) is called from the draw path (`result_grid_view.rs:76`) **every frame**, uncached, with a comparator that allocates via `cell_text` per comparison (`:113-122`) | **`Fix RC1`** — child issue **#238** | **measured, not suspected**: 200,000 rows × 4 columns → projection 5.35 ms unsorted, **3,414 ms sorted on a text column**, 83.65 ms filtered+sorted (debug, this host). Reachable at `max_rows` up to 100,000 (`crates/core/src/domain/connection.rs:111,116`; default 500), and the grid is deliberately virtualized for large results (`result_grid_view.rs:813`) — so a sorted large result cannot repaint. Responsiveness only: no crash/data loss/credential leak/SQL misclassification, so it stays P2 by the goal-3 §24 rule |

**Acceptance check for this section:** all six live Data Grid rows have exactly one disposition; the
single `Fix RC1` row has a focused child issue (**#238**) with the measurement, the blast radius, the
bounded fix options and its own acceptance list. No speculative fix was implemented in this audit;
the only code touched was a temporary measurement probe, removed before the commit.

---

## C. Connection / session / SSH (#77)

Scope rows: `QA-P2-15`, `16`, `17`, `18`, `19`, `20`, `21`, plus one scope item the issue names
explicitly (SSH controls visible although full SSH qualification is post-v0.1).

| Row | Native mechanism (opened for this audit) | Disposition | Rationale |
|---|---|---|---|
| `QA-P2-15` connection test result goes stale after form edits | the tested draft is remembered (`connection_view.rs:917` `connection_test_draft = Some(draft)`) and the result is accepted only while the current draft still equals it: `events.rs:481-485` sets `connection_test_valid = true` on equality and otherwise keeps it `false` with the message `"Connection changed · test again before saving"`; every edit path also clears it (`connection_view.rs:156,185,303,914`) | **`Accept RC1`** — positively verified | a stale "verified" indicator cannot survive an edit, and the user is told to re-test rather than left guessing |
| `QA-P2-16` Test Connection hides backend error detail | on failure the UI stores the runtime's own message: `events.rs:956` `self.connection_error = message.clone()` and `runtime_message = format!("Connection failed · {message}")`, and the dialog renders that string verbatim in a `Destructive` alert (`connection_view.rs:497-501`) | **`Accept RC1`** — positively verified | the finding's intent (detail suppressed behind a generic failure) is inverted in the native dialog: the backend text is what the user reads |
| `QA-P2-17` SQLite Browse has no user-visible error path | the file picker result is handled in `events.rs:425-441`: a picked path sets the draft and clears the error; a **cancelled** pick sets `connection_error = "File selection was cancelled"` and clears the validity flag | **`Accept RC1`** — verified for the cancelled path | the React defect mechanism (the Tauri dialog plugin's rejection vanishing) is replaced by an explicit handled branch. Note the backend is `rfd`, a different mechanism from the one the original fix targeted — so this is not the same defect repaired, it is the absence of the failure mode |
| `QA-P2-18` `driverChanged` means "ever changed", not "differs from original" | there is no change-tracked driver flag natively: the driver is a plain field of `UiConnectionDraft`, and the domain config is derived per submit (`draft_to_domain`) | **`Accept RC1`** | the React state variable the finding describes does not exist; there is no "ever changed" heuristic to be wrong |
| `QA-P2-19` Duplicate connection silently omits credentials | `open_duplicate_connection` (`connection_view.rs:161`) copies the summary but sets `password: String::new()` (`:171`) | **`Accept RC1`** | secrets stay in the keyring and are never copied into a new draft; the field is empty and visible, and a remote connection cannot be saved without re-entering it. **Recorded, not asserted as ideal:** there is no explicit "password not copied" hint — that is a possible polish item for a later release, not a v0.1 defect since nothing is implied to be present |
| `QA-P2-20` Favorite optimistic update has no rollback | there is **no favourites feature** in the shipped product: `grep -rln favorite crates/ui/src crates/core/src crates/runtime/src` returns nothing | **`Defer post-v0.1`** | the finding's subject is absent by scope (connection folders/tags/favourites are #204, post-v0.1); nothing to roll back |
| `QA-P2-21` SQLite recent subtitle renders meaningless host/port | there is no "recent connections" list in the native shell — the only `recent` references are the saved/recent **queries** palette entry (`palette_view.rs:40`, `query_view.rs:483`) | **`Defer post-v0.1`** | the mechanism requires a recent-connections surface that v0.1 does not ship |
| scope item: SSH control visible though SSH is post-v0.1-qualified | `connection_view.rs:713` renders `Connect via SSH Bastion Tunnel` with no in-UI caveat, while the repository records the capability as unqualified (LIM-006, readiness row, `R009`) | **`Fix RC1`** — child issue **#239** | the issue's own acceptance requires that v0.1 "does not accidentally imply fully qualified SSH support", and the packaged app shows the user none of the documents that say otherwise. The fix is one muted hint, matching the existing `Preview` badge pattern on the agent surface (`agent_view.rs:224`) |

**Acceptance check for this section:** all seven live connection/session rows have exactly one
disposition, and the SSH scope item is dispositioned rather than left to the documents. The single
`Fix RC1` row has a focused child issue (**#239**) with its own acceptance list. No code was changed
by this audit.

---

## D. Query / ER residual findings (#78)

Scope rows: `QA-P2-22`, `23`, `24`, `25`. The remaining scope bullets (statement selection/run-all,
cancellation and stale execution context, Explain/result metadata, saved-query rename atomicity,
query history persistence, error/loading/empty states; ER small/medium regressions, labels/minimap,
persisted manual positions, perf HUD) carry **no live P2 row** of their own after Gate 4 and Gate 5 —
recorded here rather than left implicit. Cancellation semantics were separately re-verified by #129's
audit (`providers/15-cancellation-capability-gating.md`).

| Row | Native mechanism (opened for this audit) | Disposition | Rationale |
|---|---|---|---|
| `QA-P2-22` export enablement tied to SQL text, not result state | the Export affordance is rendered **inside** `if let Some(value) = result` together with the `rows · ms` summary (`query_view.rs:412-421`), and the export dialog is only reachable from there (`:1290-1300`) | **`Accept RC1`** — positively verified | export availability is a function of the result payload, not of editor text: with no result there is no button to press. This is the exact acceptance line the manual-smoke checklist carries (`0.1.0-manual-smoke.md:205`) |
| `QA-P2-23` dirty replacement uses native `window.confirm` | see §A — `NOT_APPLICABLE_IN_NATIVE` | **`Accept RC1`** (dispositioned in §A, not re-counted here) | the mechanism needs a browser API; egui has none. Listed for traceability only |
| `QA-P2-24` ER search auto-picks first substring match | search decides a **view mode**, not a selection: `diagram_search_mode(large_schema, show_all) = large_schema && !show_all` (`diagram_view.rs:168`) opens the focused neighbourhood of *all* matches (the empty-state copy at `:296` says "open a focused neighborhood map"), and editing the query clears an explicit show-all (`diagram_show_all_after_search_edit`, `:172`) | **`Accept RC1`** | there is no "first substring match" to auto-pick: matching filters the graph instead of choosing one node, and the explicit show-all escape hatch is pinned by its own test. The #74 note that disambiguation is untested remains true for *ordering* of matches — recorded, not promoted |
| `QA-P2-25` ER derived state does unnecessary work + LOD runtime gaps | the derived structures are built once per graph build (`crates/ui/src/diagram/model.rs:85-86` builds `node_lookup` and `adjacency` with capacity up front) and a search subset reuses the existing graph rather than rebuilding it (`diagram/tests.rs:1629` `search_subset_reuses_existing_graph`) | **`Accept RC1`** | the memoization intent is addressed and pinned by that test; the other two sub-items (handle IDs stripped per LOD level, Fit View via the React Flow API) are `NOT_APPLICABLE_IN_NATIVE` — React Flow belongs to the archived stack, and the native renderer is `crates/ui/src/diagram/` with its own layout (`layout.rs:9` documents its bounded working set) |

**Acceptance check for this section:** all four Query/ER rows have exactly one disposition and none is
duplicated with Gate 4/5 closure work (Gate 4 handled the large-schema diagram invariants; Gate 5
handled value classes). No `Fix RC1`, so no child issue was spawned; no code was changed.
