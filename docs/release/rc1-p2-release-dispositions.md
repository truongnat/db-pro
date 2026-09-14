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

---

## E. Performance tooling and release-measurement semantics (#79)

Scope rows are the eight bullets the issue names. Unlike §A–D these are **not** `QA-P2-*` product
findings: the 25 P2 rows carry no perf-tooling, bundle or measurement-semantics row, so the audited
objects are the tooling artifacts themselves (`.skills/perf-audit/`, its reference tables, the
React-era perf reports and PR #13). That absence is stated rather than papered over.

| Row (scope bullet) | Current behaviour (opened for this audit) | Disposition | Rationale |
|---|---|---|---|
| perf-scan/perf-audit `all` semantics and warnings | `all` executed four checks (release build, binary size, `cargo check`, `cargo clippy`) and then printed `Status: PASS — All checks passed`, while the two benchmark sections and the ER/DB runtime sections were deliberately **not** run. The only mention was a prose note printed between the sections and the summary | **`Fix RC1`** — child issue **#240**, fixed in this audit | a quoted `PASS` for `all` reads as "everything was measured"; the readiness/checklist/handoff rows all quote exactly that string. Now every skipped section is counted and **named**, and the status reads `PASS (partial) — 4 executed check(s) passed; not run: Native UI benchmarks, Rust backend benchmarks, ER diagram runtime, DB query performance` |
| fresh-build / source-SHA validation vs stale `dist` heuristics | there is no `dist` heuristic left to be stale: `grep -rn '\bdist\b'` over workflows, scripts and manifests finds no build-output directory, the archived frontend's output lives under `_archive/frontend/`, and the scan builds the release binary with `cargo build --release --locked -p db-pro-native` **in the same run**. When that build fails the scan reports the failure and prints "no binary size reported", so a stale binary is never measured | **`Accept RC1`** — positively verified | the PR #13 wording ("detects stale `dist`") described the React-era script; the native script cannot measure a binary its own build did not produce. Verified by reading the failure path and by the `all` run captured in `providers/47` |
| bundle/dependency count documentation drift | no live release document quotes a JS bundle size or a dependency count: the only such numbers live in `docs/plans/performance-optimization-report-2026-08-13.md` and `docs/plans/performance-baseline-audit-2026-08-13.md`, both carrying a `HISTORICAL (2026-09-11)` banner, and `grep` over `docs/release/*` + `README.md` finds no bundle-size claim. The skill's own statement ("There is no bundle-size metric anymore") was already correct | **`Accept RC1`** | nothing stale to correct in release-facing text; the binary-size budget is the live proxy and is measured, not asserted. The one real drift found was inside the skill's budget tables (next row) |
| `.skills/perf-audit/SKILL.md` stale paths | §2 listed five "operations" as budget rows (`Grid visible-range computation`, `Grid hit-testing`, `Cell formatting / codec round-trip`, `Quick Open index + rank`, `Statement split`) that **no criterion file registers** — `crates/ui/benches/result_grid_benchmarks.rs` registers exactly three ids — and §4 named three backend ids incompletely plus one that is not a benchmark at all ("Execution registry"). `references/perf-budgets.md` repeated that drift and carried a `strip target/release/db-pro-native` row for a packaging step no workflow performs | **`Fix RC1`** — child issue **#240**, fixed in this audit | a budget table that names benchmarks that do not exist cannot be used to verify anything. Both tables now list the exact registered criterion ids (`result_grid_million_rows/project_without_filter_or_sort`, `result_grid_scroll_window/materialize_100_visible_rows`, `result_grid_requested_sizes/build_visual_maps_{1_000,10_000}_rows_50_columns`, `query_rows/select_10k`, `query_json_blob/select_json_metadata_5k`, `serialize_large_text/select_1k_large_text`, `explain_query`, …) and mark every row with no benchmark behind it as such instead of leaving it looking enforced |
| output exit codes / failure semantics | **measured on this host:** `bash perf-scan.sh er` printed `Status: WARN — 1 warning(s), review recommended` and exited **0** (old script, `git show HEAD:…` copy). Any gate that reads only the exit status therefore certified a warned run as green. `cargo clippy` messages were counted into a `warn` that also exited 0 | **`Fix RC1`** — child issue **#240**, fixed in this audit | the contract is now `PASS` 0 / `WARN` **2** / `FAIL` 1, and a run on an uncommitted tree is qualified `[source <sha>+dirty(N): working tree not committed]`. Pinned by `perf-scan.sh --self-test` (14 assertions, ~0 s, no build) which was falsified with three probes before being trusted (1, then 2, then 1 failures; restored → green) |
| build artifact provenance used in release evidence | the scan reported a size from a hardcoded path and nothing else, so a recorded `PASS … 22.6MB` could not be tied to a commit or an artifact. `perf.scan` **is** an `evidence-manifest.json` gate row (`status: pass`, `observed_at_sha 85a7fa3`) and is quoted by the readiness table, the release checklist, the final report and the handoff | **`Fix RC1`** — child issue **#240**, fixed in this audit | every run now prints `Source revision: <sha>[+dirty(N)]` and `Measured artifact: target/release/db-pro-native sha256 <digest> (<size>MB)` in both the header and the summary, and the size line carries the digest prefix. The manifest's historical `perf.scan` row is **left as captured** — it records what the tool printed at `85a7fa3`, and rewriting a past observation would be the opposite of provenance |
| loading fallback / i18n findings tied to perf paths | the "loading fallback" items came from PR #13's React `Suspense` wrappers (`workspace-content.tsx`), which belong to the retired frontend; the i18n item is the live P2 row `QA-P2-06` ("significant Agent/Connection UI strings bypass i18n"), which is a cross-cutting string finding, not a perf-path one | **`Defer post-v0.1`** (fallbacks, subject absent) / **cross-referenced to §F** (i18n) | the native analogue of a Suspense fallback is the introspection/query in-flight state, which §F dispositions under #80 — recorded here as a cross-reference so the row is not counted twice and not silently dropped |
| PR #13 follow-up items that remain relevant | PR #13 (merged 2026-08-13) is the React bundle optimization: main chunk 1,182 → 684 KB, dependencies 87 → 53, 45 files removed. Its metrics are void as release evidence (frontend retired 2026-09-11; both perf reports carry the `HISTORICAL` banner) and its stated follow-ups were "detects stale `dist`" and "returns proper exit codes" for `perf-scan.sh`. A `grep` for a remaining action item in the PR body finds nothing else | **`Accept RC1`** | the two lasting follow-ups are settled by the `Accept RC1` and `Fix RC1` rows above, and the bundle metrics are historical, not claimed |

**Acceptance check for this section:** all eight scope bullets have exactly one disposition; the
defects live in the audit tool itself, are small and safe, and were fixed and self-tested in place; the
three `Fix RC1` rows are carried by one focused child issue (**#240**) with the measured evidence.
Measurement record: `docs/release/evidence/v01-runtime/providers/47-perf-tooling-and-measurement-semantics.md`.
