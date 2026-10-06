# Table Indexes Responsive Layout — Findings

## P2 — Status column is outside the visible Indexes viewport

- Evidence: pre-fix native fixture capture `evidence/indexes-light-1280x800-before.png`; the visible header row ends at Predicate and Status is absent.
- Source: baseline `crates/ui/src/table_indexes_surface_view.rs` at commit `710ba002612c6d71ef2605f99f2a49743b51c3a4` declared fixed widths totalling 1,020pt, plus the shared table's 80pt minimum flex width for Status. The reproduced 928pt viewport cannot contain the 1,100pt request.
- Reproduction: `cargo test --locked -p db-pro-ui index_columns_fit_the_standard_table_detail_viewport -- --nocapture` failed before the fix with `index columns request 1100 pt from a 928 pt viewport` (exit 101).
- Fix: compute all six widths from the visible viewport; use dynamic cell truncation and hover disclosure. Keep a 640pt minimum content width for genuinely narrow viewports, where horizontal scrolling is expected.
- Verification: the same targeted test passed after the fix; `evidence/indexes-light-1280x800-after.png` shows all six columns and the complete `Enforces uniqueness` status.
- Disposition: addressed in working-tree blob `46f85b3a119c511633dc9a31bc81b74882e05fa6`.

## P2 — Singular count uses plural grammar

- Evidence: baseline header formatted `Total: 1 indexes`.
- Fix: render `index` for one and `indexes` otherwise.
- Disposition: addressed in working-tree blob `46f85b3a119c511633dc9a31bc81b74882e05fa6`; visible in the after capture.

## Other paths reviewed

- Empty or filtered results use the existing empty state; metadata absence uses the loading state.
- Clicking an index name emits `SelectIndex`, stores the selected name, and renders the detail dialog from the same metadata record.
- These paths were source-reviewed but not exercised in the runtime capture. No provider call or mutation occurs in this presentation path.

## Independent review — 2026-10-04

### P2 (inherited) — Index detail action cannot be triggered by the name label

- Evidence: current Indexes screenshot `evidence/indexes-light-1280x800-after.png`, region `Indexes table / Index Name cell`; the name is visually plain text. In working-tree blob `46f85b3a119c511633dc9a31bc81b74882e05fa6`, `table_indexes_surface_view.rs:158-162` checks `response.clicked()` but creates the widget with `Label::new(...).truncate()` and never assigns click sense. egui 0.29.1 `Label::new` defaults to hover-only sense unless `.sense(Sense::click())` is set ([upstream source](https://github.com/emilk/egui/blob/0.29.1/crates/egui/src/widgets/label.rs)). The same click handling existed at baseline SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4` (`table_indexes_surface_view.rs:165-170`), so this is inherited, not introduced by the responsive-width change.
- Failure scenario: clicking the only index name cannot emit `SelectIndex`; repository search shows `TableMetadataView` is the only path assigning `table_index_detail`, so the detail dialog is unreachable through normal UI interaction.
- Recommendation: give the index-name control click and keyboard/accessibility semantics and a clear hover affordance, or remove the misleading click path if details are not intended.
- Status: fixed in current working-tree blob `600e950c84b7128ceee37cba2a938de6d971df94`: the label has click sense, pointer affordance, and button accessibility metadata; `clicking_index_name_opens_its_details` verifies pointer activation. Follow-up visual review also confirmed icon/name spacing at 1280×800. Dialog title text is now hidden while its close button remains. Keyboard and detail-dialog native runtime behavior remain unverified.

### Review coverage gap — wider viewports and other states remain unverified

- Evidence: only the loaded, light-theme 1280×800 logical capture is available. 1440×900 was previously stopped after a stall; 1920×1080, constrained width, empty/loading, and detail-dialog captures are absent.
- The captured state shows all six columns, including Status, within the viewport; no visual overflow is visible at this size. This does not establish the remaining required viewport/state combinations.
- Disposition: keep `RUNTIME_VERIFY`; do not claim the full UI acceptance gate passed.

## Metadata header and modal removal — 2026-10-06
- P2 legacy input had 32px content height plus vertical margins and inherited a different font from the surrounding section/total labels. Indexes and Structure now use one header renderer with the shared 30px compact input token and 13px UI-label font for title/input/total.
- Removed index and column modal renderers, click actions, adapters and their unused TableState selection fields. Names are passive metadata; long names/values retain full content on hover, with index definition available there.
- Baseline `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted changes; no provider command or persisted-data changes.

Foreign Keys now also uses the same metadata filter header. Only its header changed; existing navigation/query behavior is retained. Deterministic Foreign Keys capture route includes a fixture relationship for visual verification.

## Constraints header follow-up — 2026-10-06
- P2: Constraints used the default accent selectable label rather than the parent `tab_button`, and a separate input renderer/font. Type cells inherited tight horizontal spacing.
- Scope: reuse the shared metadata header with a trailing slot, right-align existing category tabs in their original order, reuse parent tab styling, and set Type icon/badge spacing to SPACE_SM. Preserve category/search predicates and provider metadata.

## Dependencies follow-up — 2026-10-06
- P2: separate oversized search header, accent category controls and cramped direction icon/badge gap. Shared metadata header and parent tab component replace the inconsistent renderers. Category filters remain right aligned in their original visible order; total remains visible below on the right to avoid crowding at 1280.
