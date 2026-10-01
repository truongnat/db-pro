# Component UI Layer Structure — Checklist

## Revised documentation and quality batches
- [x] Batch 1: AspectRatio, Badge, Card, Code, DevTools, Separator; document API/design, remove shared-token aliases, add AspectRatio/Separator Gallery samples.
- [x] Batch 2: Chrome, Feedback, Logs, Nav, ResponsiveLayout, ScrollArea.
- [ ] Batch 3: Input and Form; preserve facade module paths; remove four Input token aliases.
- [ ] Batch 4: Select, Selection, RadioGroup, Toggle, Tabs; preserve tab public types and add RadioGroup Gallery sample.
- [ ] Batch 5: Accordion, Calendar, Collapsible, HoverCard.
- [ ] Batch 6: Dialog and Overlay.
- [ ] Batch 7: Alert, Command, Transaction; remove Command shared-stroke alias and add Transaction README.
- [ ] Batch 8: Database, Diff, Explain, SqlEditor.
- [ ] Batch 9: Table, Tree, Workspace.
- [ ] Batch 10: AgentComposer and AgentPrimitives.
- [ ] Run the component and workspace gates recorded in `PLAN.md`; do not close the plan while native runtime evidence remains pending.

## Button architecture update (2026-10-01)
- [x] Record the revised component contract in `PLAN.md` and `components/README.md`.
- [x] Split Button UI and handler layers into focused submodules while keeping `mod.rs` exports.
- [x] Remove core token imports and aliases from Button `config.rs`.
- [x] Add `DESIGN.md` for UI and logic rationale and `API.md` for detailed use.
- [x] Record the existing English flow-comment rule and apply it to Button's UI/handler boundaries.
- [x] Record the five component quality requirements and perform a source-only Button self-review.
- [x] Resolve filled-button text contrast across rest/hover in light and dark modes.
- [x] Honor `Reduce motion` for Button hover/press/loading.
- [ ] Verify meaningful names, role/state, Tab/Enter/Space, focus ring, and target sizes in the native UI/accessibility tree.
- [ ] Inspect Button variants and narrow/loading/disabled states at required viewports.
- [x] Run focused Button tests, UI crate check, formatting, and diff check; record exact outcomes.
- [ ] Run workspace-wide gates, native release build, and runtime verification after owner code review.

## HoverCard UI Product Review batch (IMPLEMENTING)
- [x] Sanitize public and defensive delay inputs with finite semantics.
- [x] Schedule repaint only for pending timer deadlines; use measured card height for collision placement.
- [x] Implement Escape dismissal, focus surrender, and inactive-trigger reopening suppression.
- [x] Add handler coverage for delays, Escape transition, and measured-height placement.
- [ ] Collect native UI screenshots/runtime evidence; pending.

## Architecture
- [x] Confirm `mod.rs` is the component's public entry/exporter.
- [x] Define UI as egui presentation/layout and handler as behavior/decisions/calculations.
- [x] Define local-config vs shared-token boundary.
- [x] Require a usage `README.md` in every component folder.
- [x] Add detailed English comments for non-obvious Rust component logic and UI flow, tracing inputs/events through handlers/state/outcomes to rendering; explain intent and rationale without merely restating code.

## Migration inventory
- [x] Select: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`.
- [x] Accordion: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`.
- [x] AgentComposer; [x] AgentPrimitives; [x] Alert; [x] Badge; [x] Calendar.
- [x] AgentComposer preserves its public builder/action API and legacy caller-facing module path.
- [x] AgentComposer separates egui layout/painting from typed handler decisions and local badge geometry.
- [x] AgentComposer adds focused handler tests and a usage README.
- [x] AgentComposer UI Product Review v3 completed; fixed blank keyboard submission, added a persistent prompt label, and enabled wrapped toolbar layout. Native runtime visuals/accessibility remain pending.
- [x] AgentPrimitives preserves all public types/builders and re-exports in its five-layer folder.
- [x] AgentPrimitives moves typed status/risk/action/progress/geometry decisions to tested handlers and documents usage.
- [x] AgentPrimitives UI Product Review v3 completed; fixed removable hit target, destructive approval hierarchy, disclosure keyboard/accessibility semantics, excessive repaint requests, and primary narrow-width overlap. Runtime evidence remains pending.
- [x] Alert preserves its public builders and exports; variant mapping/action decisions are in handler and local sizing/opacity in config.
- [x] Alert protects destructive confirmation from backdrop dismissal by default, makes destructive mode opt-in, handles Escape, provides initial Cancel focus and supports id_salt.
- [x] Alert UI Product Review v3 completed; fixed narrow-width close/action layout and dialog width budgeting. Focus trap/restoration and native runtime evidence remain pending.
- [x] Calendar fixes month/day normalization, strict ISO parsing, persistent viewed month/year, selection close behavior, and current-date initialization.
- [x] Calendar disables stale open popups and closes with Escape; transitions and popup-open decisions have handler tests.
- [x] Calendar UI Product Review v3 completed; P1 disabled-popup issue fixed. Native keyboard, popup placement and interaction lifecycle evidence remain pending.
- [x] Button: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`.
- [x] AspectRatio: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`.
- [x] Card, Chrome, Code, Command, Database, DevTools, Dialog.
- [x] Collapsible: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`.
- [x] Diff: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`.
- [x] Explain, Feedback.
- [x] Form: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`.
- [x] HoverCard, Logs, Navigation: existing layer structure verified; handler tests cover log styling and navigation page decisions.
- [x] Input: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`; focused handler tests added. Native runtime evidence remains pending.
- [x] Overlay, Selection, ResponsiveLayout, SqlEditor, ScrollArea.
- [x] RadioGroup: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`; focused handler tests cover orientation spacing and navigation.
- [x] Separator: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`.
- [x] Table, Tabs, Transaction, Workspace, Toggle, Tree: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`; icon mapping, interaction decisions, geometry, reveal clipping and ID salts covered in handler/config.
- [x] Audit `components/mod.rs` re-exports and classify shared infrastructure (`animation`, `interact`, `common`, `common_utils`, `legacy`); added missing `floating_surface` root export. Move common layout/format support into `common::{layout,format}` with explicit root exports and retain `common_utils` as a compatibility facade; keep `legacy::*` compatibility exports until callers migrate.
- [ ] Initiative runtime gate: screenshots/accessibility evidence for normal/loading/error/empty states at 1280×800, 1440×900, and 1920×1080; implementation checklist marks do not imply runtime acceptance.

## Tree verification
- [x] Preserve `DatabaseTreeNode`, `TreeNodeKind`, `TreeNodeKind::icon()`, `reveal_children`, and `components::tree`/`components::*` re-export compatibility.
- [x] Move icon mapping, row/background decisions, expansion toggle decisions, chevron crossfade layers, row geometry and reveal clipping math into `handler.rs`.
- [x] Move Tree-owned dimensions, thresholds, font sizes and legacy animation/data ID salts into `config.rs`; preserve `hover`, `chev_anim`, and `content_h` semantics.
- [x] Keep egui allocation, animation invocation, painting and child UI clipping in `ui.rs`; remove the partial `geometry.rs`/`traversal.rs` split.
- [x] Add focused handler tests for icon mapping, interactions, row layout, chevron fade layers, reveal clipping and ID salts; update README usage/API notes.
- [x] Historical isolated Tree worker Cargo gates were blocked by then-missing modules; this is superseded for the integrated worktree by passing workspace gates recorded in `VERIFICATION.md`.
- [ ] Native runtime screenshots/accessibility evidence remains pending.

## Reusable implementation guide
- [x] Document the repeatable component authoring workflow, responsibilities, review checklist and Select example in `crates/ui/src/components/README.md`.
- [x] Cross-link the durable guide from `PLAN.md`; keep the plan as the migration-specific contract.
- [ ] Apply the guide to the remaining public components during their migration batches.

## Accordion verification
- [x] Preserve public API and disabled-item interaction behavior in multi-expansion mode.
- [x] Use shared spacing token for icon/title gap.
- [x] Add handler regression test for disabled open/closed items.
- [x] Run targeted fmt/test/check and record results in VERIFICATION.md.
- [x] Complete UI Product Review v3; fix missing keyboard/focus/accessibility semantics and title/badge overlap; retain runtime layout evidence as a pending gate.

## Button verification
- [x] Preserve `Button`, `ButtonGroup`, `ButtonSize`, and `ButtonVariant` public re-exports through `components::button` and `components::*`.
- [x] Keep egui allocation/painting, hover/press/focus, tooltip, loading cursor, and widget-info accessibility behavior in UI layer.
- [x] Separate size tokens, palette resolution, width calculation, and content positioning into handler functions.
- [x] Add handler regression tests for size presets, width calculation, alignment positioning, and palette mapping.
- [x] Add Button README usage example.
- [x] Run targeted fmt/test/check and record results in VERIFICATION.md.
- [x] Complete UI Product Review v3 on component states, accessibility, and usability; resolve actionable findings and record limitations.

## AspectRatio verification
- [x] Preserve `AspectRatio` public API and `components::AspectRatio` re-export.
- [x] Separate ratio sanitization and size calculation into handler functions.
- [x] Keep egui allocation/clipping in UI layer.
- [x] Add handler regression tests for invalid ratios and size calculation.
- [x] Add AspectRatio README usage example.
- [x] Run targeted fmt/test/check and record results in VERIFICATION.md.
- [x] Complete UI Product Review v3; layout sizing and clipping contract reviewed, no source-level blocker found.

## Badge verification
- [x] Preserve `Badge` and `BadgeVariant` public re-exports.
- [x] Expose badge text as accessible label metadata.
- [x] Document intrinsic width and dot-over-icon priority.
- [x] Complete UI Product Review v3; source-level contract reviewed, runtime contrast/visual evidence pending.

## Separator verification
- [x] Preserve `Separator` builder API and `SeparatorOrientation` re-export.
- [x] Keep egui allocation and painting in the UI layer.
- [x] Extract horizontal/vertical allocation and line geometry calculations into tested handlers.
- [x] Add Separator README usage example.
- [x] Run targeted fmt/test/check and record results in VERIFICATION.md.
- [x] Complete UI Product Review v3 and fix actionable findings; document source-only evidence/unknown runtime states.
- [x] Document decorative separator semantics; labeled horizontal separators expose label text.

## Collapsible verification
- [x] Preserve `Collapsible` builder API and `components::Collapsible` re-export.
- [x] Keep egui allocation, animation, and frame layout in the UI layer.
- [x] Extract header toggle logic, chevron icon determination, color resolution, and badge rectangle math into tested handlers.
- [x] Add Collapsible README usage example and layer explanations.
- [x] Use the allocated response ID to avoid animation collisions; support an optional stable `.id(Id)` override.
- [x] Support focused Space/Enter activation, accessible button info, and a focus ring while preserving disabled behavior.
- [x] Run targeted fmt/test/check and record results in VERIFICATION.md.
- [x] Complete UI Product Review v3 and fix actionable findings; document source-only evidence/unknown runtime states.

## Form verification
- [x] Preserve `form::field::*` and `components::{FormField,Label}` compatibility exports and existing builders.
- [x] Keep egui layout/rendering in `ui.rs`; centralize validation-error visibility decisions in typed `handler.rs` and keep the label font default local in `config.rs`.
- [x] Add focused compatibility and validation-mode/state-transition tests; document lifecycle, accessibility labeling, disabled/error/helper behavior in README.
- [x] Complete source-only UI Product Review v3: required/error context is attached to the input accessibility label, disabled state is delegated to `Input`, and error text takes precedence over helper text. Native runtime evidence remains pending.
- [x] Resolve prior P3 review suggestions: omit blank accessibility context and support caller-provided unique `FormField` ID salts without changing label defaults.
- [x] Run targeted fmt/test/check and diff checks; record outcomes in `VERIFICATION.md`.

## Select verification
- [x] Preserve Select API/re-exports.
- [x] Separate UI rendering from handler decisions and calculations.
- [x] Add handler tests.
- [x] Add/verify Select README usage example.
- [x] Add explanatory English comments for non-obvious Select UI flow and geometry helpers.

## Dialog verification
- [x] Add the component-layer `ui.rs`, meaningful typed modal dismissal handler, README, and locally documented configuration while retaining `Dialog`, `Sheet`, `DialogFrame`, and compatibility exports.
- [x] Keep `Dialog`/`Sheet` rendering in UI modules and preserve caller-owned `open` state, body/footer layout, animation, and existing backdrop policy.
- [x] Route Sheet through shared modal topmost registration, Escape ownership, focus trap, and backdrop/card layer ordering.
- [x] Add tests for topmost dismissal, inside/outside backdrop decisions, Escape precedence, focus-anchor fallback/preservation, and public compatibility paths.
- [x] Complete source-only UI review and address Sheet overlay-stack P1; backdrop non-dismissal is documented. Runtime screenshots/accessibility traversal remain pending.
- [x] Run targeted fmt/test/check/clippy/diff gates; record in `VERIFICATION.md`.

## Diff verification
- [x] Preserve `DiffViewer`, `DiffLine`, line constructors, summary, theme mapping, and root/component re-exports.
- [x] Keep galley measurement and egui painting in `ui.rs`; move pure content-width geometry into `handler.rs`.
- [x] Document Diff-owned header/row layout offsets and keep theme colors/stroke values on canonical tokens.
- [x] Preserve the >4-digit line-number gutter test and test width calculation with right padding.
- [x] Complete UI Product Review v3; fix content color, empty state, accessible viewer labeling, and clamp stats origin. Retain P2 when stats text is wider than the header and runtime evidence limitation.
- [x] Run targeted fmt/test/check/clippy/diff gates and full UI/native gates; record outcomes in `VERIFICATION.md`.

## Database verification
- [x] Preserve driver/status/action public types, connection-card builders, badge API, and re-exports.
- [x] Separate status/action/name/icon decisions and test all mappings; keep egui presentation in `ui.rs` and documented local geometry in `config.rs`.
- [x] Replace placeholder SSL text color with semantic theme color and expose provider/SSL accessible labels.
- [x] Prevent repeat Connect while Connecting, label Error recovery as Retry (still emits legacy Connect action), and truncate long identity/host labels with hover text.
- [x] Complete source-only UI Product Review v3; record the remaining extreme narrow-width P2 and runtime evidence limitation.
- [x] Run focused tests, UI crate check/clippy/fmt, and diff checks; full crate/native gates recorded in `VERIFICATION.md`.

## Command verification
- [x] Preserve `CommandInput`, `CommandItem`, `CommandGroup`, and `CommandEmpty` APIs/re-exports and their existing response/disabled semantics.
- [x] Split egui presentation from tested hover/selection/color and geometry decisions; keep component-specific defaults in documented config constants.
- [x] Add usage README describing caller-owned filtering/dispatch and the inert `CommandItem::id` contract.
- [x] Expose accessible button labels/selected/disabled metadata, clip row text before the shortcut slot, and avoid active styling for disabled selected rows.
- [x] Complete source-only UI Product Review v3 and address all actionable findings; Form helper/error context is included in the input semantic label; native runtime evidence remains pending.
- [x] Run targeted formatting, tests, crate check/clippy, and diff checks; record outcomes in `VERIFICATION.md`.

## Chrome verification
- [x] Preserve `Avatar`, `AvatarSize`, `AvatarShape`, `AvatarStatus`, `Skeleton`, `EmptyState`, `Toolbar`, and `toolbar_button` public APIs and re-exports.
- [x] Move egui allocation/layout/painting to `ui.rs`; extract avatar and skeleton decisions/geometry into tested handlers.
- [x] Keep local defaults in `config.rs` and consume shared theme/style tokens.
- [x] Add accessible avatar labels, normalize invalid skeleton geometry, add focused handler tests, and document usage in README.
- [x] Complete source-only UI Product Review v3; fix actionable accessibility, geometry, and floating-point test findings. Native runtime evidence remains pending.
- [x] Run targeted formatting, test, crate check, and diff checks; record exact outcomes in `VERIFICATION.md`.

## Card verification
- [x] Preserve the existing `.change(text, is_positive)` builder and document its legacy mapping.
- [x] Add public `MetricTrendDirection` and `MetricTrendTone` types plus the nonbreaking `.trend(text, direction, tone)` builder.
- [x] Keep direction-icon and tone-color mapping independent; omit the icon for `Unspecified` and expose the trend row through `WidgetInfo`.
- [x] Update gallery semantics for slow-query rate and staged mutations; use a wrapped metric header to contain long titles.
- [x] Add focused direction/tone mapping coverage and update the Card README/re-exports.
- [x] Run the targeted Card verification commands recorded in `VERIFICATION.md`.

## Explain verification
- [x] Preserve `ExplainPlanTree`, `PlanNode` builders, public helpers, and component re-exports; move model/adaptation decisions to `handler.rs` while keeping egui rendering in `ui.rs`.
- [x] Document Explain-owned geometry/thresholds and provider/metric behavior; keep shared colors and typography on theme tokens.
- [x] Aggregate PostgreSQL per-loop actual time/rows and planned rows consistently, display valid loop counts, and preserve zero-vs-missing runtime semantics.
- [x] Bound plan parsing, heuristics, adaptation, and rendering by depth/node budgets; expose one truncation finding and sanitize extreme/non-finite metrics.
- [x] Add regression tests for per-loop metrics, invalid/extreme values, depth/wide trees, warning persistence, and accessible UI rendering.
- [x] Complete source-only UI Product Review v3 at `fc1c4f815887c25696bbad13fd72bc9d6b1ae17d`; no open P0/P1/P2 findings. SQLite EXPLAIN normalization remains outside this migration.
- [x] Run targeted/full UI tests, core Explain tests, query output metric test, fmt/check/clippy/native build, and diff checks; record exact outcomes in `VERIFICATION.md`.
- [ ] Capture native runtime screenshots/accessibility evidence at 1280×800, 1440×900, and 1920×1080, including applicable normal/loading/error/empty states.

## Feedback verification
- [x] Confirm the existing five-layer folder, public exports, and caller paths; preserve all existing constructors/builders and add only nonbreaking accessibility-label builders.
- [x] Normalize non-finite progress fractions and invalid heights before animation/allocation; guard beam-edge overflow and extreme finite animation inputs.
- [x] Expose labeled `WidgetType::ProgressIndicator` metadata, report determinate values as percentages, and omit values for indeterminate indicators/spinners.
- [x] Keep geometry/normalization/accessibility data in tested handlers; update comments and README for non-obvious input and semantic behavior.
- [x] Complete source-only UI Product Review v3 at `7a119f968ca2c4c14f21ded5fb2e9059de525583`; no P0/P1/P2 findings remain.
- [ ] Run the current batch's focused/full UI tests, fmt/check/clippy/native release build, diff check, and clean-code scan; record exact results in `VERIFICATION.md`.
- [ ] Capture native runtime screenshots/accessibility evidence at 1280×800, 1440×900, and 1920×1080, including applicable normal/loading/error/empty states.

## Logs and Nav verification
- [x] Preserve `LogEntry`, `LogLevel`, `LogViewer`, pagination, breadcrumb, and header APIs/re-exports.
- [x] Keep log level and navigation decisions in typed handlers; keep egui layout/painting in `ui.rs` and component defaults in `config.rs`.
- [x] Replace navigation presentation literals with named semantic/token-backed configuration and add focused log policy coverage.
- [x] Document caller-owned log ordering/filtering and the component usage example in `logs/README.md`.
- [x] Run targeted tests and quality gates; record exact outcomes in `VERIFICATION.md`.

## Toggle and ScrollArea verification
- [x] Preserve `Toggle`, `ToggleGroup`, `ToggleGroupItem`, `ToggleSize`, `ToggleVariant`, and `ScrollArea` public APIs/re-exports.
- [x] Move toggle size, width, appearance, rounding, and selection decisions into typed handlers; keep egui measurement/allocation/painting in `ui.rs`.
- [x] Keep ScrollArea axis ordering and scrollbar visual lifecycle decisions in `handler.rs`; preserve caller style restoration.
- [x] Replace toggle/scroll-area presentation magic values with documented component-local constants and add focused handler tests.
- [x] Add `toggle/README.md` and update `scroll_area/README.md` with layer responsibilities and usage constraints.
- [x] Integrated crate/workspace test, fmt, check, clippy and native release gates pass; see current verification entry. Native runtime evidence remains pending.

## Quality gates before completion
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --quiet`
- [x] `cargo build --release --locked -p db-pro-native`
- [x] Source review of the integrated batch; historical worker reports and known documentation findings are recorded separately.
- [ ] Capture required native runtime screenshots/accessibility evidence at 1280×800, 1440×900, and 1920×1080.

## Tổng kết bằng tiếng Việt
Đây là kế hoạch migration toàn bộ component công khai. Select là batch đầu; checklist từng component sẽ được đánh dấu theo thay đổi thực tế, kèm README hướng dẫn dùng.
