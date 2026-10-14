# PATCH_globs — glob/loop semantics for check-ui-architecture.sh

Slice owner: PatchGlobs. Plan §3 + §4 (glob/loop/exclusion-regex semantics).
All `rg`/`find` invocations below were run against the post-reorg tree; counts
and clean-sweep results are real outputs, not estimates.

Ownership settled with PatchPaths: they own `explicit_state_modules`
(verbatim repoint), `state_field_leaks`, the event-existence loop, and all
literal path fixes. Edits below marked NOT MINE are retained only as
verification evidence — do NOT apply them.

Two shared arrays are introduced and consumed by my blocks
(`feature_state_modules`, `feature_event_modules`). Keep their definitions in
the positions shown (both precede first use).

Bash-3.2 note (macOS `/bin/bash`): `mapfile`/`readarray` do NOT exist; all
arrays below use `while IFS= read -r … done < <(rg --files …)` which is 3.2-safe.
No `shopt -s globstar` needed — no `**` shell globbing anywhere in my edits.
(`rg --files` also respects `.gitignore`, which is the right behavior.)

---

## Changes

### Edit 1 (L136–308): `explicit_state_modules` — NOT MINE (owned by PatchPaths)

PatchPaths confirmed this block is in their assignment and their patch carries
the full 164-entry repointed list (all paths verified). That is the correct
fix — see the measured analysis below for why the plan's "folder globs"
suggestion does NOT work. **Do not apply any glob rewrite here.**

**Why PLAN §2's glob idea fails (verified):** the plan asked to glob every
`*.rs` in the feature folders. Measured on the clean tree, ~60 feature files
legitimately reference `DbProApp` outside `#[cfg(test)]` (they take
`&mut DbProApp` params): `table/table_view.rs`, `schema/schema_workbench.rs`,
`agent/agent_view.rs`, `connection/lifecycle.rs`, `component_gallery/*.rs`,
`query_workbench/*_view.rs`, `result_grid/*.rs`, `workspace/workspace_view.rs`,
etc. A blanket glob exits 1 on a clean tree. A tighter
`*_state.rs`+`*_surface_view.rs` glob over the feature dirs verifies clean but
yields only **91 files vs the curated 164** — silently dropping
`agent_context`, `explorer_navigation`, `query_save_actions`,
`security_rls`, `synthetic_data`, `workspace_session`, `settings_*_view`
non-surface files, reducers/actions/projections. The verbatim repoint
preserves the gate's coverage exactly; take PatchPaths' version.

### Edit 2 (L310–316): `*_state.rs` DbProApp loop → recursive, shared array

OLD:

```bash
for state_module in "$repo_root"/crates/ui/src/*_state.rs; do
  [[ "$state_module" == "$repo_root/crates/ui/src/app_state.rs" ]] && continue
  if rg -n '^impl DbProApp|\bDbProApp\b' "$state_module"; then
    echo "UI architecture check failed: feature state modules must not depend on DbProApp." >&2
    exit 1
  fi
done
```

NEW:

```bash
# Feature state modules now live one level deep inside feature folders after
# the 2026-10-14 reorg — recurse for *_state.rs instead of the old flat
# src/*_state.rs glob, which silently matched nothing.
feature_state_modules=()
while IFS= read -r module; do
  feature_state_modules+=("$module")
done < <(rg --files "$repo_root/crates/ui/src" -g '*_state.rs')

for state_module in "${feature_state_modules[@]}"; do
  # app/app_state.rs is the composition-root state; it may reference DbProApp.
  [[ "$state_module" == "$repo_root/crates/ui/src/app/app_state.rs" ]] && continue
  if rg -n '^impl DbProApp|\bDbProApp\b' "$state_module"; then
    echo "UI architecture check failed: feature state modules must not depend on DbProApp." >&2
    exit 1
  fi
done
```

Verified: 41 `*_state.rs` files found recursively (old flat glob would have
matched 0 — the check was dead); 40 after the app_state exemption; **zero
`DbProApp` hits** → PASS.

Integration note: PatchPaths also carries the L311 `app_state.rs`→
`app/app_state.rs` repoint; my loop replaces that whole block, so apply
whichever lands first — the exemption path inside the NEW loop is already
the repointed one.


### Edit 3a (L318–329): three `*_state.rs` UiCommand `if`s → shared glob loop

OLD:

```bash
if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/table_state.rs"; then
  echo "UI architecture check failed: TableState must not construct runtime protocol commands." >&2
  exit 1
fi
if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/table_data_query_state.rs"; then
  echo "UI architecture check failed: TableDataQueryState must not construct runtime protocol commands." >&2
  exit 1
fi
if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/query_library_state.rs"; then
  echo "UI architecture check failed: QueryLibraryState must not construct runtime protocol commands." >&2
  exit 1
fi
```

NEW:

```bash
# UiCommand is runtime protocol: feature state modules consume events, never
# construct commands. app/app_state.rs is exempt — the composition root owns
# command dispatch.
for state_module in "${feature_state_modules[@]}"; do
  [[ "$state_module" == "$repo_root/crates/ui/src/app/app_state.rs" ]] && continue
  if rg -n '\bUiCommand\b' "$state_module"; then
    echo "UI architecture check failed: feature state modules must not construct runtime protocol commands." >&2
    exit 1
  fi
done
```

The four intervening non-state checks (schema_workbench, explorer_navigation,
query_execution_actions, query_save_actions — L330–345) and
`connection/lifecycle.rs` (L361–364) stay; PatchPaths repoints those paths.

Integration note: PatchPaths' patch repoints these same three checks (they
were in their assignment). If their repoints land, apply my NEW loop at the
Edit-3b position instead and DELETE the three repointed `if`s — the glob
subsumes them exactly (verified: all 40 non-exempt state files are UiCommand-
clean, and the glob covers strictly more files).

### Edit 3b (L346–360): management-state UiCommand `for` list → delete

All nine entries are `*_state.rs` files now covered by the Edit-3a glob
(`audit|fdw|replication|event_trigger|pg_settings → tools/*_state.rs`,
`monitoring_state.rs`, `security_state.rs`, `overlay_state.rs`,
`schema_compare_state.rs` — all verified present under `tools/`,
`monitoring/`, `security/`, `shell/`, `schema/`).

OLD:

```bash
for state_module in \
  "$repo_root/crates/ui/src/audit_state.rs" \
  "$repo_root/crates/ui/src/fdw_state.rs" \
  "$repo_root/crates/ui/src/replication_state.rs" \
  "$repo_root/crates/ui/src/event_trigger_state.rs" \
  "$repo_root/crates/ui/src/pg_settings_state.rs" \
  "$repo_root/crates/ui/src/monitoring_state.rs" \
  "$repo_root/crates/ui/src/security_state.rs" \
  "$repo_root/crates/ui/src/overlay_state.rs" \
  "$repo_root/crates/ui/src/schema_compare_state.rs"; do
  if rg -n '\bUiCommand\b' "$state_module"; then
    echo "UI architecture check failed: management read-only state must not construct runtime protocol commands." >&2
    exit 1
  fi
done
```

NEW: (delete — superseded by Edit 3a's loop)

Verified: zero `\bUiCommand\b` hits across all 40 exempted state files → PASS.
Also verified `app/app_state.rs` contains no `UiCommand` today, so the
exemption is belt-and-suspenders, matching the documented composition-root
rule.

### Edit 4 (L366–387): reducer list → `*_events.rs` glob minus `runtime/`

OLD:

```bash
for reducer in \
  "$repo_root/crates/ui/src/agent_events.rs" \
  "$repo_root/crates/ui/src/connection_events.rs" \
  "$repo_root/crates/ui/src/ddl_events.rs" \
  "$repo_root/crates/ui/src/file_picker_events.rs" \
  "$repo_root/crates/ui/src/management_events.rs" \
  "$repo_root/crates/ui/src/schema_events.rs" \
  "$repo_root/crates/ui/src/table_events.rs" \
  "$repo_root/crates/ui/src/query_library_events.rs" \
  "$repo_root/crates/ui/src/query_prediction_events.rs" \
  "$repo_root/crates/ui/src/query_save_events.rs" \
  "$repo_root/crates/ui/src/query_history_events.rs" \
  "$repo_root/crates/ui/src/query_queue_events.rs" \
  "$repo_root/crates/ui/src/query_execution_events.rs" \
  "$repo_root/crates/ui/src/query_result_events.rs" \
  "$repo_root/crates/ui/src/query_multi_result_events.rs" \
  "$repo_root/crates/ui/src/query_failure_events.rs"; do
  if rg -n '^impl DbProApp|\bDbProApp\b' "$reducer"; then
    echo "UI architecture check failed: feature event reducers must depend on explicit state, not DbProApp." >&2
    exit 1
  fi
done
```

NEW:

```bash
# Feature event reducers are *_events.rs inside feature folders. src/runtime/
# holds the dispatcher plumbing (operation_events.rs) — it is not a feature
# reducer and is excluded. `rg --files` output is relative to the cwd, so pipe
# through `rg -v` to drop the runtime dir.
feature_event_modules=()
while IFS= read -r module; do
  feature_event_modules+=("$module")
done < <(rg --files "$repo_root/crates/ui/src" -g '*_events.rs' | rg -v '/runtime/')

for reducer in "${feature_event_modules[@]}"; do
  if rg -n '^impl DbProApp|\bDbProApp\b' "$reducer"; then
    echo "UI architecture check failed: feature event reducers must depend on explicit state, not DbProApp." >&2
    exit 1
  fi
done
```

Verified: glob finds 17 `*_events.rs`; minus `runtime/operation_events.rs`
(the only `*_events.rs` under `runtime/` — `runtime/events.rs` and
`runtime/event_router.rs` do not match `*_events.rs`) = **16 feature
reducers**, exactly the old list's membership, at new paths
(`agent/agent_events.rs`, `connection/connection_events.rs`,
`files/file_picker_events.rs`, `table/{ddl,table}_events.rs`,
`schema/schema_events.rs`, `tools/management_events.rs`, 9×
`query_workbench/query_*_events.rs`). Zero `DbProApp` hits → PASS.

### Edits 5–6: `state_field_leaks` (L393–398) + event-existence loop (L413–418) — NOT MINE (owned by PatchPaths)

PatchPaths confirmed both blocks were explicitly in their assignment; their
patch carries verified replacements:

- `state_field_leaks`: `rg --glob '*_state.rs'` over `crates/ui/src` (recursive
  by default — no globstar needed) + repointed explicit files
  (`schema/schema_workbench.rs`, `connection/{state,lifecycle,catalog}.rs`).
  Verified equivalent to my `feature_state_modules[@]` version: `rg` treats
  explicit file args as paths, `--glob` applies only to directory recursion —
  so the literal files still get scanned, and `app/app_state.rs` stays
  included via the glob (it has zero `pub` fields; verified clean).
- Event-existence loop: 19 explicit repointed paths (runtime plumbing +
  feature reducers incl. `query_failure_events.rs`). Stricter than my
  glob-non-empty version (a single deleted reducer still fails) — prefer it.

Do NOT apply my earlier versions of these blocks; integrator takes
PATCH_paths.md's. If integration ever drops their `rg --glob` fix, my
equivalent (shared `feature_state_modules` array + repointed literals) is in
git history of this file — but default to theirs.

### Edit 7 (L79, L86, L93): `task_bridge` exemption `app.rs` → `app/mod.rs`

Three identical OLD snippets — one per check:

```bash
| rg -v '/app\.rs:' || true)
```

NEW (each):

```bash
| rg -v '/app/mod\.rs:' || true)
```

Applies to `direct_sends` (L79), `direct_best_effort_sends` (L86),
`direct_request_allocations` (L93). Verified on the new tree: every
`task_bridge.send(`/`send_best_effort(`/`next_request_id(` call site is inside
`app/mod.rs` — after the new filter all three pipelines are empty → PASS.
Exemption kept narrow (`app/mod.rs` only, not `src/app/`) so sibling app
helpers like `app/command_dispatch.rs` stay gated.

Integration note: PATCH_paths.md carries the IDENTICAL change for these
three lines (confirmed in coordination). Apply once — whichever patch lands
first wins; if the second patch is applied after, its OLD text will no
longer match (harmless — skip it).

### Edit 8 (L420–428): raw-widgets exclusion — add `*/tests/` dirs

OLD comment + filter:

```bash
# RULES.md: view code must use crates/ui/src/components primitives, never raw
# egui widgets. Exemptions: components/ (the wrappers themselves), dev_tools/,
# *_tests.rs / test modules, and lines carrying an explicit `// ui-rules:` comment.
raw_widgets=$(rg -n \
  'ui\.checkbox|ui\.radio\b|ui\.toggle_value|ui\.button\(|ui\.selectable_label|ui\.text_edit_|ui\.menu_button|egui::ComboBox|egui::Slider|egui::RadioButton|egui::TextEdit|egui::ScrollArea' \
  "$repo_root/crates/ui/src" --glob '*.rs' \
  | rg -v '/components/|/dev_tools/|_tests\.rs:|app_tests\.rs:|component_gallery_rendering\.rs:' \
```

NEW:

```bash
# RULES.md: view code must use crates/ui/src/components primitives, never raw
# egui widgets. Exemptions: components/ (the wrappers themselves), dev_tools/,
# *_tests.rs files, tests/ directories, test modules, and lines carrying an
# explicit `// ui-rules:` comment.
raw_widgets=$(rg -n \
  'ui\.checkbox|ui\.radio\b|ui\.toggle_value|ui\.button\(|ui\.selectable_label|ui\.text_edit_|ui\.menu_button|egui::ComboBox|egui::Slider|egui::RadioButton|egui::TextEdit|egui::ScrollArea' \
  "$repo_root/crates/ui/src" --glob '*.rs' \
  | rg -v '/components/|/dev_tools/|/tests/|_tests\.rs:|app_tests\.rs:|component_gallery_rendering\.rs:' \
```

`/tests/` is placed before the filename patterns (rg alternation order is
cosmetic here; kept first for readability). `tests/` dirs verified present:
`app/tests/`, `editor/tests/`, `query_workbench/tests/`, `result_grid/tests/`,
`settings/tests/`, `table/tests/`. `app_tests\.rs:` kept per plan §4 (belt:
`app/tests/app_tests.rs` is already hit by `/tests/`). `dev_tools/` and
`component_gallery/component_gallery_rendering.rs` both verified still
existing.

### Edit 9 (L388–391): `database_operations|DatabaseOperationsState` — NO CHANGE NEEDED

```bash
if rg -n 'database_operations|DatabaseOperationsState' "$repo_root/crates/ui/src" --glob '*.rs'; then
```

Already scans the whole tree recursively — recursion depth is irrelevant to
`rg`. Verified against the new layout: **zero matches** (the state file was
replaced by `schema/database_management_state.rs`, which contains neither
string). Works as-is post-reorg. No edit — documenting so integrator doesn't
touch it.

## Verification

Run locally (all executed, clean tree):

- `rg --files crates/ui/src -g '*_state.rs'` → 41 files (recursion works).
- `rg --files <22 feature dirs> -g '*_state.rs' -g '*_surface_view.rs' -g '!*_tests.rs' -g '!**/tests/**' -g '!**/mod.rs'` → 91 files, zero `DbProApp` awk hits.
- `*_state.rs` minus `app/app_state.rs`: zero `DbProApp`, zero `UiCommand`.
- `*_events.rs` minus `runtime/`: 16 reducers, zero `DbProApp`.
- `task_bridge.send/send_best_effort/next_request_id` outside `app/mod.rs`: empty.
- `pub`/`pub(crate)` fields across all 41 state files + 4 literals: empty.
- `database_operations|DatabaseOperationsState` over `src`: zero matches.

Parent should run after merging all sibling patches:

- `bash scripts/check-ui-architecture.sh` → expect `UI architecture boundary: PASS`.
- Adversarial: `echo 'fn f(){let _=DbProApp;}' >> crates/ui/src/table/table_view.rs`
  → Edit-2/3a loops won't catch it (it's not `*_state.rs`), but that's
  identical to old behavior — `table_view.rs` was never on the old allowlist.
- Known residual: bash 3.2 + `set -u` errors on `"${arr[@]}"` if an array is
  ever empty (e.g. all `*_state.rs` deleted). Acceptable — the gate fails
  loudly; if undesired, guard with `[[ ${#arr[@]} -gt 0 ]]`.
