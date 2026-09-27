# Component UI Layer Structure — Checklist

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
- [x] Card, Chrome, Code, Command, Database, DevTools; [ ] Dialog.
- [x] Collapsible: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`.
- [ ] Diff, Explain, Feedback, Form, HoverCard, Input, Logs, Navigation.
- [ ] Overlay, RadioGroup, ResponsiveLayout, ScrollArea, Selection, SqlEditor.
- [x] Separator: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`.
- [ ] Table, Tabs, Toggle, Transaction, Tree, Workspace.
- [ ] Audit `components/mod.rs` re-exports and classify shared infrastructure (`animation`, `interact`, `common_utils`, `legacy`).

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

## Select verification
- [x] Preserve Select API/re-exports.
- [x] Separate UI rendering from handler decisions and calculations.
- [x] Add handler tests.
- [x] Add/verify Select README usage example.
- [x] Add explanatory English comments for non-obvious Select UI flow and geometry helpers.

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
- [x] Complete source-only UI Product Review v3 and address all actionable findings; native runtime evidence remains pending.
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

## Quality gates before completion
- [x] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [x] `cargo build --release --locked -p db-pro-native`
- [ ] Review each batch diff and confirm no behavior/API regression.

## Tổng kết bằng tiếng Việt
Đây là kế hoạch migration toàn bộ component công khai. Select là batch đầu; checklist từng component sẽ được đánh dấu theo thay đổi thực tế, kèm README hướng dẫn dùng.
