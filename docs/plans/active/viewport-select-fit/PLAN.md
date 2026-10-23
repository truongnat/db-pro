# Viewport Select and Radio Allocation Fit

## State

- **State:** IMPLEMENTING
- **Branch:** `main` (direct edit per owner instruction)
- **Baseline SHA:** `9b50e7a27b4d92de5b126e6ecaf3a518448ad97b`
- **Surface:** Native Rust `egui` UI (`crates/ui` selection/radio_group and `crates/app` settings_appearance + query_context_picker).

## Goal

Resolve horizontal overflow caused by unconditional full-width allocation in horizontal `Radio` / `RadioGroup` / `Checkbox` components, and compact the oversized query connection/schema picker while eliminating duplicate `ScrollArea` ID collisions.

## Evidence and Failure Scenario

1. **Appearance Horizontal Overflow (F1):**
   - At baseline SHA `9b50e7a27b4d92de5b126e6ecaf3a518448ad97b`, `crates/ui/src/components/selection/ui.rs:351-355` allocated `row_width = ui.available_width().max(...)`.
   - In `RadioGroup::horizontal(true)` (`crates/ui/src/components/radio_group/ui.rs:102-106`) and callers like `crates/app/src/settings/settings_appearance_view.rs:15-30`, the first option ("Light") allocated the entire remaining width of the card.
   - The second option ("Dark") was pushed ~800px to the far right window edge, resulting in text cutoff in both Light and Dark modes.

2. **Oversized Sparse Query Context Picker (F2):**
   - At baseline SHA `9b50e7a27b4d92de5b126e6ecaf3a518448ad97b`, `crates/app/src/query_workbench/query_context_picker_view.rs:27-96` only specified `ui.set_min_width(280.0)`.
   - Because `paint_option` allocates `ui.available_width()`, the floating popup expanded to ~600px wide across the editor.
   - Furthermore, two `ScrollArea::new(...)` instances lacked `auto_shrink([true, true])`, expanding to `max_height(180.0)` each even for short lists (2 connections, 1 schema), creating ~270px of empty dead space.

3. **Duplicate ScrollArea ID Collision (F3):**
   - Both `ScrollArea` instances in `query_context_picker_view.rs:43,78` lacked explicit `id_salt`, resulting in identical IDs and debug warning banners ("First use of ScrollArea ID 45C9" and "Second use of ScrollArea ID 45C9") painted over options.

## Severity and Scope

- **Severity:** P2 (UX layout defect and visual polish).
- **In Scope:**
  - `crates/ui/src/components/selection/ui.rs`: content-aware compact width allocation for horizontal parents or explicit `.compact(true)` on `Radio` and `Checkbox`.
  - `crates/ui/src/components/radio_group/ui.rs`: propagate `compact(horizontal)` to each `Radio`.
  - `crates/app/src/settings/settings_appearance_view.rs`: migrate raw `ui.checkbox` to `Checkbox` component per `RULES.md`.
  - `crates/app/src/query_workbench/query_context_picker_view.rs`: pin bounded content width (`300.0..=360.0`), apply `auto_shrink([true, true])`, assign unique `id_salt`s (`"query_context_connections_scroll"` and `"query_context_schemas_scroll"`), and handle empty lists cleanly.
  - Native capture harness additions in `capture.rs` and `workspace_actions.rs` to support deterministic matrix captures.
  - Regression unit tests in `crates/ui` and `crates/app`.
- **Out of Scope:**
  - Complete redesign of all application select controls.
  - Introducing heavy new frameworks or changing database behavior.

## Acceptance Criteria

- [x] `Radio` and `Checkbox` in horizontal parents allocate content-derived width, allowing siblings to sit adjacent without wrapping or clipping.
- [x] Vertical descriptive choices maintain full clickable row width when in vertical containers.
- [x] Query context picker popup width is bounded and hugs the trigger; each list region is sized to its rows and capped, so short lists stay compact and long lists scroll in a bounded window.
- [x] No duplicate `ScrollArea` ID warning banners appear in the context picker.
- [x] All unit regression tests pass (10 new tests: 4 selection, 5 picker, 1 palette ladder).
- [x] 17-cell matrix captures (2 surfaces × 3 viewports × 2 themes + dense list + short list + Explorer popover) executed and visually verified.
- [x] Light chrome reads as its own region without collapsing hover/selected states (owner-reported F7).
