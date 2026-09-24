# Findings — Explorer First-Tab Visual Redesign

## Baseline review

- `docs/design/sidebar-explorer/sidebar-explorer-baseline.md` is a source inventory, explicitly not runtime evidence.
- `docs/design/sidebar-explorer/sidebar-explorer-feature-research.md` lists product capability gaps F1–F11 and a capability roadmap. It is not a visual redesign spec; implementing those recommendations would expand the user's visual-polish scope.
- Both Explorer documents pin claims to `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`. The inspected current Explorer source files have no changes after that baseline in `HEAD`, so the inventory remains current for this slice.
- The appropriate visual source is `docs/10-egui-native-migration-plan.md` §1.1, together with `DbProTheme`; use the actual theme tokens rather than proposed colors in the historical migration brief.

## Findings

### F-1 — Research roadmap and visual redesign are different scopes

**Severity:** P2 (scope/UX)  
**Evidence:** the feature report's Phase 1–3 proposes object filters, favorites, quick docs, compare, DDL workflows, provider folders, and dependency navigation. These are capabilities, not visual treatment.  
**Decision:** excluded from this feature. Use research only to preserve existing behavior and identify important row states.

### F-2 — Visual design references and current token implementation differ

**Severity:** P2 (design consistency)  
**Evidence:** migration-plan §1.1 describes dark-first graphite with lavender/indigo accent; `crates/ui/src/theme.rs` currently defines light as default, warm-minimalist surfaces, and blue accents.  
**Decision:** do not introduce a competing palette in Explorer. Treat the current `DbProTheme` as canonical for implementation; capture current visual behavior before proposing token-level changes.


### F-3 — Connecting connection is collapsed by default

**Severity:** P2 (state discoverability)  
**Evidence:** the pre-change 1280×800 Explorer capture shows a connecting root row collapsed with only a small status dot. Its `Connecting…` hint is rendered only when the row is open (`explorer_connection_node_view.rs`), while `ConnectionRowContext::draw` defaulted open only for connected/failed states (`explorer_connection_row_view.rs`).  
**Decision:** include `is_connecting` in the row's default-open condition. This exposes the existing progress hint; it adds no action, changes no reducer/provider path, and preserves a user-collapsed row through egui's persistent state.

### F-4 — Active connection had no selected-row treatment

**Severity:** P2 (visual hierarchy)  
**Evidence:** the connection row passed `is_selected: false` for every lifecycle state, even though the connected row represents the active connection.  
**Decision:** render the connected connection as selected, using the existing selected-row token treatment. The current runtime fixture could not reach connected state, so this visual branch remains source-verified only.

## Self-review

- **P0:** 0. **P1:** 0. **P2:** 4 (F-1–F-4; F-1/F-2 are scoped dispositions).
- No action variants, reducer mappings, database commands, provider capabilities, or persistence formats changed.
- The `Connecting…` row opens only by default when no persisted disclosure state exists; users can still collapse it.
- The connected-row selected treatment derives from the existing `is_connected` state.
- Runtime review remains incomplete: no connected PostgreSQL schema tree or Explorer error-state capture; the host cannot reach the required 900/1080 logical heights.
- Independent review has not run; the feature remains `RUNTIME_VERIFY`, not lifecycle-completed.

## Provider impact

| Provider | Impact | Runtime requirement |
|---|---|---|
| PostgreSQL | None; presentation-only change | N/A |
| SQLite | None; presentation-only change | N/A |
