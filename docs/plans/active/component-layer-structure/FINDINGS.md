# Component Layer Structure — Findings

## Button quality self-review — 2026-10-01

Scope: Button source and core token inputs at baseline SHA `7423aea985fd878f447e229a968eac30cf3868f5`, plus the current uncommitted changes on `main`. This is an implementer self-review with focused tests; native UI, accessibility tree, and rendered-pixel contrast remain unchecked. The defects below were inherited from the baseline Button behavior.

| Requirement | Static result | Evidence and remaining check |
|---|---|---|
| Consistent appearance | Partial | Variants consume `DbProTheme` in `button/handlers/palette.rs`; sizes consume the core contract in `button/handlers/size.rs`. Light/dark and all states still need native visual review. |
| Accessibility | Partial | Filled-button contrast passes the 4.5:1 token check through rest/hover in both themes. Reduce motion reaches Button. Icon-only names are enforced in all builds. Focus appearance, Enter, and the native accessibility tree still need runtime review. |
| Approachable UI/UX | Partial | Loading and disabled block click. `ButtonGroup` now uses a 4-point shared gap, leaving room for adjacent 2-point focus rings. Narrow labels and loading comprehension require runtime review. |
| Dynamic customization | Partial | Theme is supplied to each Button and core tokens own shared dimensions. Variants and size presets are configurable; arbitrary per-instance styling is not exposed. Whether that is needed remains a product decision. |
| Clean code and formatting | Partial | `mod.rs` keeps the public API, UI/handlers are separated, and `config.rs` has no core imports. Focused tests, formatting, UI crate check/Clippy, and clean-code scan pass; the scan reports two warning categories, recorded in `VERIFICATION.md`. |

### Fixed inherited P1 — Filled-button text contrast

At the baseline SHA, `button/handler.rs:95-129` uses white foreground for Default and Destructive text. The declared tokens give white on light accent `#0285ff` **3.62:1**, white on dark accent `#4f8cff` **3.22:1**, and white on dark danger `#ef6b73` **2.99:1**. `DbProTheme::text_on_solid` now chooses black or white from the resolved fill. A focused test checks 101 hover steps for both filled variants in both themes; every measured pair reaches 4.5:1. Rendered-pixel evidence is still pending.

### Fixed inherited P1 — Reduce motion did not reach Button

`settings_appearance_view.rs` already offers `Reduce motion`. The preference now travels through the frame's `DbProTheme`; Button skips hover/press interpolation and holds the loading arc still without requesting spinner repaints. Runtime confirmation remains pending.

### Fixed inherited P2 — Icon-only name depended on debug-only enforcement

The icon-only `access_label` requirement now uses a normal assertion, including a nonblank check, so invalid controls cannot silently render in release. A source scan found no product call site without text or `access_label`; the only match is the intentional panic test. The native accessibility tree still needs review.

Review verdict: **ACCEPT WITH P2** for source changes (P0=0, P1=0; P2=1 for native visual and accessibility-tree evidence). This is implementer self-review, not independent approval. PostgreSQL and SQLite: n/a.

## Input batch (implementation)
- P2 — Input presentation was spread across sibling modules without a component-level presentation/decision boundary. Added `ui.rs` as the presentation entry, `handler.rs` for pure label/counter decisions, and retained existing widget APIs.
- P2 — Textarea's byte-length counter disagreed with user-visible character semantics for Unicode. The counter now uses `chars().count()` through a tested handler function. Runtime visual/accessibility evidence remains pending.

## Baseline
- Historical migration baseline: `2fb54db0` (`main` before the earlier component-layer migration).
- Current continuation baseline: `a2ded17c839724dc95fe8f0ed8e9a8f07a4e50dd` on clean local `main`, matching `origin/main` when work began.
- Current inventory: 38 public components; Button has all three contract documents. The remaining 37 need `DESIGN.md` and `API.md`; `tabs` and `transaction` also need `README.md`. Existing checklist marks cover the earlier structure and do not count as acceptance of these documents.
- Select previously mixed egui rendering with popup keyboard/state decisions, accessible-label construction, and width calculations.
- Most exported components are standalone `.rs` files; complex families already use subdirectories but inconsistent file boundaries.

## Revised contract — Batch 1
- At baseline `a2ded17c839724dc95fe8f0ed8e9a8f07a4e50dd`, AspectRatio, Badge, Card, Code, DevTools, and Separator already had the intended UI/handler/config module shape and README; this batch added the missing design/API contracts without manufacturing more layers.
- Removed Badge's private `BADGE_RADIUS` mirror and Separator's private shared-spacing mirrors; rendering now reads `RADIUS_BADGE` and `SPACE_SM` directly from shared tokens.
- Added Gallery coverage for AspectRatio clipping/ratio and both separator orientations. Static scoped review found no P0/P1; required native viewport/accessibility evidence is still P2 and remains open.

## Decisions
- `mod.rs` is the public component entry/exporter.
- Every public component is in scope and must have the named layers plus a usage README.
- UI owns egui rendering/layout and applies egui side effects; handler owns decision logic and calculations called by UI.
- Config owns component-specific defaults; shared design tokens/theme stay centralized.
- Migrate in batches to keep behavior and API stable; empty handler/config files are not accepted, so implementation must move meaningful responsibilities into them.

## Findings
- P2 — Select mixed UI and logic. Its keyboard navigation, trigger/accessibility labels, widths, option selection, dismissal, load-more decisions and dropdown geometry now live in typed handler functions.
- P2 — AspectRatio mixed ratio validation and size calculation into its egui allocation method. The pure decisions now live in typed handler functions with regression tests; UI still owns allocation, clipping and child rendering.
- AgentComposer UI review (source-only at HEAD `81238b4df2b173e6faa15180eddecddbbc36bdf2` plus worktree): P1 blank keyboard submission bypassed prompt validation; fixed by shared handler decision. Prompt label and wrapped toolbar added. P2 native narrow-width/accessibility evidence remains pending.
- AgentPrimitives UI review (source-only at HEAD `81238b4df2b173e6faa15180eddecddbbc36bdf2` plus worktree): initial P1 removable-chip hit target, destructive approval hierarchy and long-title overlap were fixed; disclosure headers gained keyboard/focus/accessibility semantics and unconditional repaint removed. Follow-up reports P0=0/P1=0, with P2 runtime interaction/layout evidence pending.
- Alert UI review (source-only at HEAD `81238b4df2b173e6faa15180eddecddbbc36bdf2` plus worktree): initial P1 destructive-by-default/backdrop dismissal, narrow-width sizing and modal identity/focus risks were addressed by opt-in destructive mode, backdrop policy, Escape, Cancel focus request, id salt and width-aware layout. Follow-up reports P0=0/P1=0; P2 focus trap/restoration and runtime verification remain open.
- Calendar review (source-only at HEAD `81238b4df2b173e6faa15180eddecddbbc36bdf2` plus worktree): P1 month/day normalization, lost view navigation state, and disabled popup mutation were fixed; current-local-date default, selection close and Escape close were added. Follow-up review reports P0=0/P1=0, with P2 egui interaction/accessibility and popup placement evidence pending.
- Database source review at implementation SHA `5ff99485b3205dc58369714e2c76ea31c9e51bb5`: initial P2 long-text overflow and repeated Connect while Connecting were fixed with shrink-aware labels/tooltips and disabled pending action; Error now labels recovery Retry while preserving the Connect action. Follow-up ACCEPT WITH P2 (P0=0/P1=0): extremely narrow cards may still be narrower than fixed status/SSL affordances; runtime viewports remain unverified.
- No provider/database backend impact; this is native UI architecture work.

## Toggle and ScrollArea batch
- P2 — Toggle previously combined size presets, width/appearance decisions, grouped corner geometry, and selection transitions with egui allocation and painting. The refactor moves those typed decisions and caller-owned click mutations into `toggle/handler.rs`, keeps egui galley measurement and painting in `toggle/ui.rs`, and preserves the caller-facing builders and re-exports.
- P2 — ScrollArea previously encoded the egui axis tuple directly in its presentation layer. Axis ordering is now a tested handler decision; scrollbar visual mutation remains a handler-owned save/restore lifecycle and the clip-margin value is named in local config.
- Source-only review at baseline SHA `6cedac0a0ff0492133d101f8272c0b801fd2e28f` plus the uncommitted batch: no P0/P1 found. Automated crate checks are blocked by pre-existing missing `form/{config,handler,ui}.rs` and `input/{handler,ui}.rs` modules outside this scope. Native runtime evidence remains pending.

## Unresolved
- Inventory each public component's meaningful handler/config responsibilities during its batch; do not duplicate shared settings or fabricate placeholder logic.
- Explain review at source SHA `fc1c4f815887c25696bbad13fd72bc9d6b1ae17d`: ACCEPT, introduced/inherited P0/P1/P2 all 0. Previous per-loop correctness and deep/wide traversal findings were fixed in commits `64096e67`, `49791a98`, and `fc1c4f81`; truncation findings persist across repeated heuristic application and render once.
- Explain provider scope: current query-output parser is PostgreSQL EXPLAIN JSON; SQLite EXPLAIN normalization remains unsupported/pending and was not changed. Required runtime screenshots/accessibility evidence and workspace-wide gates remain open plan-level items.
- Feedback was already organized into the five required layers before this batch; the inventory omission was stale. Source follow-up at `7a119f968ca2c4c14f21ded5fb2e9059de525583` sanitizes progress inputs, adds ProgressIndicator accessibility values/labels, and guards beam-edge overflow. Initial review P2s were fixed; final source review ACCEPT with introduced/inherited P0/P1/P2 all 0. Runtime screenshots/accessibility tree evidence remains pending.

## Common layer continuation on `main`
- P2 — `common_utils.rs` was a mixed shared-support bucket containing layout geometry and display formatting. The implementation is now split into named `common/layout.rs` and `common/format.rs` modules, with explicit root exports and a compatibility facade so existing callers do not break.
- Scope is UI-only; no PostgreSQL/SQLite/provider behavior changed. Automated source checks pass; native runtime screenshots/accessibility traversal remain pending at the initiative level.

## Revised contract — Batch 2
- `chrome`, `feedback`, `logs`, `nav`, `responsive_layout`, and `scroll_area` already had meaningful UI/handler boundaries; this batch completed the design/API contract without adding placeholder modules.
- P1 — Skeleton pulse/shimmer, indeterminate Progress beam, and Spinner continued animated repaint while `DbProTheme::reduce_motion` was true. They now use the shared animation preference decision; indeterminate progress and Spinner retain a static visible state. A handler test locks the reduced-motion decision and a geometry test locks the static beam segment.
- Removed repeated shared-token mirrors from Chrome and Feedback configs and Logs' renderer. Logs/Feedback previously publicly re-exported some of these mirror constants; `API.md` documents their removal and replacement tokens. No in-repository callers used those aliases.
- Static review found no remaining P0/P1 in scope. Native screenshots, real keyboard focus, accessibility-tree traversal, and narrow-window inspection remain P2 plan-level gates.

## Revised contract — Batch 3
- Input's existing `ui.rs` already groups Text, Search, Password, and Textarea as private presentation modules; Form already exposes compatibility facades. No physical move or additional abstraction was needed.
- Removed `INPUT_ROUNDING`, `FIELD_INNER_MARGIN_X`, `FIELD_INNER_MARGIN_Y`, and `INPUT_ICON_GAP`; widgets now use shared `RADIUS_XS`, `SPACE_SM`, and `SPACE_XS`. The old alias-equality test was deleted because it only locked duplicated values.
- Preserved the `input::layout`, `form::field`, `form::rules`, and `form::state` public paths. In-repo callsites use component exports; external users of the four removed Input config aliases must migrate to shared tokens.
- Targeted source/test review found no P0/P1. P2: native narrow-width, keyboard/focus, and accessibility-tree runtime checks remain open at initiative level.

## Revised contract — Batch 4
- Select, Selection, RadioGroup, and Toggle already had useful UI/handler boundaries; added the missing design/API contracts without introducing extra layers. Their config files contain component-owned values; no duplicate shared-token aliases were found in Select or Tabs config to remove.
- Moved SegmentedTabs and UnderlineTabs into `tabs/ui/`, kept their public exports, and moved click/keyboard selection policy to `tabs/handler.rs`. The UI now passes egui key/click/focus signals into the handler. Tab indicator motion snaps directly to its target when `DbProTheme::reduce_motion` is enabled.
- Added a RadioGroup sample to Component Gallery. Source review and focused tests found no P0/P1. P2: native keyboard/focus, accessibility-tree, theme contrast and viewport evidence remain outstanding for the initiative.

## Revised contract — Batch 5
- P2 — Calendar custom-painted month arrows, day cells, and trigger had no explicit keyboard activation or accessibility metadata. They now expose accessible labels/selected state, draw focus rings, consume Enter/Space when focused, restore trigger focus after popup dismissal/selection, and mark DatePicker responses changed when its selection/open state changes.
- P2 — Shared disclosure hover/open/body transitions ignored `DbProTheme::reduce_motion`. Accordion and Collapsible now render those transitions immediately and bypass animated body clipping under the preference; Calendar hover feedback is also immediate.
- Calendar's existing date normalization, stable viewed-month state, Escape/outside dismissal, post-selection close, and screen-clamped popup behavior remain. HoverCard's sanitized timer, measured-height collision placement, Escape suppression, and bounded scroll content were already covered; this batch documents their contracts without changing those paths.
- Source review found no P0/P1. P2 remains for native keyboard/focus, accessibility-tree, visual contrast, and viewport evidence; DatePicker intentionally has no arrow-key traversal across date cells.

## Revised contract — Batch 6
- Dialog already had meaningful `ui`, `frame`, `sheet`, `layout`, `handler`, and shared `modal_guard` responsibilities. Overlay already had distinct UI, tooltip, toast, dismissal handler, and local config; no extra module split was justified.
- Added `DESIGN.md` and `API.md` for Dialog and Overlay, detailing compatibility exports, open-state ownership, show return values, topmost Escape/backdrop rules, focus anchor behavior, pointer dismissal, menu/toast results and timers, layout constraints, and keyboard/accessibility limitations.
- Source review found no new P0/P1. Existing modal tests cover topmost routing, Escape precedence, clicks inside/outside the card, focus fallback/preservation, and compatibility exports. Popover/dropdown do not handle Escape or focus restoration; tooltip is hover-only; toast API has no live-region semantics. These are documented P2 runtime/accessibility limits, not silently claimed as covered.
- No PostgreSQL/SQLite/provider impact. Native keyboard/focus and viewport screenshots remain pending at initiative level.

## Revised contract — Batch 7
- Alert, Command, and Transaction already had purposeful UI/handler/config boundaries. Added design/API contracts for all three and the missing Transaction README. Command's `BORDER_WIDTH` mirrored `tokens::STROKE_THIN`; removed the alias and used the shared token directly in its renderer.
- Verified destructive AlertDialog ignores backdrop clicks but supports explicit cancel/Escape; TransactionBar emits caller-owned actions, and DestructiveOperationDialog requires exact case-sensitive keyword matching after trimming and disables confirmation while invalid. Gallery examples already expose Alert, Command, TransactionBar, and destructive confirmation.
- Source review found no P0/P1. P2 limits documented: AlertDialog lacks a full focus trap/restore and a busy state; command navigation/dispatch remains caller-owned; transaction surfaces have no busy argument, so callers must suppress repeat actions while work is pending.
- No database/provider behavior changed; these are presentation and action-intent components only.

## Revised contract — Batch 8
- Database, Diff, Explain, and SqlEditor already had meaningful UI/handler/config boundaries. Added DESIGN/API contracts, correcting public behavior and limits from implementation and callers; updated SqlEditor README to reflect that Ask AI remains available while a query runs.
- SqlEditor config duplicated common `ButtonSize::Sm`, `SPACE_XXS`, and `STROKE_THIN` values. Removed those aliases and referenced shared tokens/button size in the UI; retained only toolbar-specific margin and corner composition.
- Explain conversion remains PostgreSQL EXPLAIN JSON through the query-output bridge. Core/shared parser and renderer budgets are 128 levels and 10,000 nodes; SQLite plan normalization is not claimed or added. Existing tests cover deep/wide truncation, loop aggregation, large metrics, and runtime labels.
- Database long labels truncate with hover values and status-specific actions; Diff uses horizontal scroll and renders all supplied rows (no virtualization); SqlEditor wraps its fixed action set. These contracts and large-content ceilings are documented. Source review found no P0/P1.
- No database behavior changed. P2 viewport/keyboard/accessibility runtime evidence remains pending.

## Revised contract — Batch 9
- Table and Tree already had purposeful layers and Gallery coverage. Added DESIGN/API contracts that state actual callback ownership, virtualized painting versus O(total rows) visibility checks, tree reveal ownership, and large-content limits.
- Workspace's `config.rs` contained geometry math and the latency threshold decision. Moved those pure decisions into `handler.rs`, keeping measurements/activity destination definitions in config and preserving current geometry/order/threshold with focused tests.
- ActivityBar and DatabaseTreeNode previously lacked explicit accessible names/focus indication; they now expose button metadata and a `DbProTheme::border_focus` outline. ConnectionIndicator now exposes name/driver/health/latency as an accessible label. StatusBarItem's existing tooltip field now participates in hover and each item publishes its label.
- Workspace status labels can still overlap when caller-supplied left/right content exceeds narrow available width. Table checks every row index each frame despite painting only visible rows; Tree has no built-in node budget/virtualization. These P2 ceilings are documented.
- Source review found no P0/P1. No database/provider behavior changed.
