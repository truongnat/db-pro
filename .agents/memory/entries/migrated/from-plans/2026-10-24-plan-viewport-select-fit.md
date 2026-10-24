---
schema_version: 1
id: "MEM-PLAN-viewport-select-fit-e0dd8f05448b"
title: "Plan archive: Viewport Select Fit"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, viewport-select-fit]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "e0dd8f05448bd7af6972cb74d1653c65fe6e58b9652d37372da06ae5d74eb0fd"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/viewport-select-fit/PLAN.md"
  - "docs/plans/active/viewport-select-fit/FINDINGS.md"
  - "docs/plans/active/viewport-select-fit/VERIFICATION.md"
  - "docs/plans/active/viewport-select-fit/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/viewport-select-fit/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Viewport Select and Radio Allocation Fit — Findings

## Severity Legend
- **P0:** Blocker, crash, data loss.
- **P1:** Correctness failure, major broken user flow.
- **P2:** UI layout defect, visual polish, maintainability.

## Findings Inventory

### F1: Horizontal Radio / Checkbox Overflow (P2)
- **File / Line:** `crates/ui/src/components/selection/ui.rs:86-104`, `376-388`
- **Symptom:** `Radio` allocated `ui.available_width()`, taking the entire remaining row in horizontal layouts. In Settings -> Appearance, "Light" pushed "Dark" off the card's right boundary, causing text clipping.
- **Root Cause:** Sizing logic lacked distinction between horizontal flow and vertical stacked flow.
- **Resolution:** Introduced `.compact(bool)` and automatic detection of horizontal parent direction (`ui.layout().main_dir() == egui::Direction::LeftToRight`). In compact/horizontal mode, allocated width is derived from measured text galley and icon size instead of full available width.

### F2: Oversized Sparse Query Context Picker (P2)
- **File / Line:** `crates/app/src/query_workbench/query_context_picker_view.rs:27-40`, `47-84`, `96-128`
- **Symptom:** The query context picker opened at ~600px width and ~450px height even for short lists (2 connections, 1 schema), dominating the screen with blank space.
- **Root Cause:** `ui.set_min_width(280.0)` did not pin or constrain width; child rows called `paint_option` which allocates `ui.available_width()`. Additionally, both `ScrollArea` instances defaulted to `auto_shrink([false, false])`, expanding to `max_height(180.0)` each.
- **Resolution:** Content width is `anchor.width().clamp(300.0, 360.0).min(max_available)`, where
  `max_available = (content_rect.width() - VIEWPORT_GUTTER - SURFACE_INSET).max(1.0)`. The
  `VIEWPORT_GUTTER` (16) and `SURFACE_INSET` (18 = frame margin 8×2 + border 1×2) are named
  constants, so the popup's outer width never exceeds the viewport even when it is narrower than
  the 300 pt preference. Each list is an **allocated region** whose height is
  `rows × (row height + gap)`, capped at 180 pt, and the scroller fills that region. `auto_shrink`
  alone was not enough: inside an auto-sized `Area` the child `Ui` only offers the height the
  previous frame produced, so a shrinking scroller converged to two visible rows and left the rest of
  a long list unreachable — a 24-schema capture now shows five rows in the bounded window.

### F3: Duplicate ScrollArea ID Collision (P2)
- **File / Line:** `crates/app/src/query_workbench/query_context_picker_view.rs:43,78`
- **Symptom:** Debug warning banners ("First use of ScrollArea ID 45C9" and "Second use of ScrollArea ID 45C9") painted directly over picker options in both themes.
- **Root Cause:** Both `ScrollArea::new(context.theme)` calls were initialized in the same `ui` context without unique `id_salt`.
- **Resolution:** Assigned explicit, distinct salts: `"query_context_connections_scroll"` and `"query_context_schemas_scroll"`.

### F4: Component-First Exemption in Settings Appearance (P2)
- **File / Line:** `crates/app/src/settings/settings_appearance_view.rs:49-52`
- **Symptom:** Direct invocation of raw `ui.checkbox` instead of `crates::components::Checkbox`.
- **Resolution:** Migrated to `crate::components::Checkbox::new(&mut self.preferences.reduce_motion, "Reduce motion", *self.theme)`.

### F7: Light chrome was indistinguishable from the canvas (P2, owner-reported)
- **File / Line:** `crates/ui/src/theme/tokens/semantic.rs` (light `background.panel`), `primitive.rs` (light ramp)
- **Symptom:** In light mode the sidebar, topbar and status bar shared almost the same fill as the white
  editor: `panel` was `#f9f9f9` against a `#ffffff` canvas — a 6/255 step that read as one region, so
  panels did not look like separate areas.
- **Root Cause:** the chrome role carried a step too small to be perceived on large flat areas.
- **Resolution:** light `panel` → `NEUTRAL_75` (`#f6f6f6`), one visible step below the canvas, while
  `elevated`/`floating` stay `#ffffff` so cards and popovers read as raised above the chrome. Dark
  tokens are unchanged (its `panel` already sits below `canvas`). The step deliberately stops short
  of `NEUTRAL_100` (`#f1f1f1`) because that equals light `hover` and would make a hovered sidebar row
  identical to its base fill; the ladder is canvas → panel `#f6f6f6` → hover `#f1f1f1` → selected
  `#eeeeee`, now pinned by a regression test.
- **Accessibility consequence:** light `tertiary`/`muted` moved to a new `NEUTRAL_525` (`#6b6b6b`,
  4.93:1 on the new panel) so the theme contrast test keeps enforcing 4.5:1. The two now-unused light
  steps (`NEUTRAL_50`, `NEUTRAL_500`) were deleted.
- **Related role fix:** four transient popups (SQL completion, rich hover, signature help, table
  column suggestions) painted `surface_panel`; with chrome now darker they would read as sunken, so
  they use `surface_elevated`, the documented role for transient surfaces.

### F5: Compact Row Cap Read the Wrong Width (P2, caught by the new narrow test)
- **File / Line:** `crates/ui/src/components/selection/ui.rs` (`Checkbox::show`, `Radio::show`)
- **Symptom:** The first compact implementation capped `row_width` with the width reported *inside*
  `ui.horizontal`. A 70 pt parent produced an 84 pt row — the child row is not bounded by a narrow
  parent, so the cap did not hold and the control still overflowed.
- **Resolution:** Capture `parent_available = ui.available_width()` before entering the horizontal
  child and use it for both the text wrap width and the row cap. The glyph+gap floor
  (`CONTROL_SIZE + CONTROL_SPACING`) remains the documented minimum when the parent is narrower
  than the indicator itself. Covered by `compact_rows_honour_a_parent_wider_than_the_glyph_floor`
  and `compact_rows_keep_the_glyph_floor_in_a_sliver_parent`.

### F6: Picker Viewport Cap Ignored Its Own Surface Inset (P2, caught by the new narrow test)
- **File / Line:** `crates/app/src/query_workbench/query_context_picker_view.rs`
- **Symptom:** A 240 pt viewport produced a 242 pt popup: the content width was capped at
  `viewport - 16`, but the floating frame adds its own margin (8×2) and border stroke (1×2), so the
  outer surface still exceeded the viewport.
- **Resolution:** Subtract both the viewport gutter and `SURFACE_INSET` from the available width.
  Covered by `query_context_picker_caps_to_a_narrow_viewport`.


## Source: `VERIFICATION.md`

# Viewport Select and Radio Allocation Fit — Verification

**Source reference (committed):**

```text
code under test  457cd6457480ff5f23f8a3e5000e9efcc771ad0b  fix(ui): keep a distinct hover step above the new light chrome
preceding fix    9e72de50fa821926a3874949e2ebc65289885d0a  fix(ui): cap horizontal selection controls …
baseline         9b50e7a27b4d92de5b126e6ecaf3a518448ad97b  (pre-fix tree)
```

The fix is committed and pushed; nothing below depends on uncommitted working-tree state. The
baseline SHA is quoted only to describe the pre-fix behaviour the findings refer to.

Host: macOS 25.6.0 (arm64), eframe/egui 0.36.2.

## 1. Red → Green regression tests

### F1 — horizontal selection control width

Test file: `crates/ui/src/components/selection/tests/layout_bounds_tests.rs`
Declared from `crates/ui/src/components/selection/ui.rs` via `#[path]`.

```text
cargo test -p db-pro-ui layout_bounds_tests
```

RED (before the fix):

```text
first horizontal radio width was 384, expected compact content width (< 120px)
first horizontal checkbox width was 384, expected compact content width (< 140px)
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 720 filtered out
```

GREEN (after the fix):

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 720 filtered out
```

Two extra cases were added after review challenged the first "never overflows" claim, and both
failed against the first implementation — they found real defects (F5/F6 in `FINDINGS.md`):

```text
compact_rows_honour_a_parent_wider_than_the_glyph_floor
  RED   compact row width 84 exceeded parent available 54
  GREEN ok
compact_rows_keep_the_glyph_floor_in_a_sliver_parent
  RED   sliver-parent row width was 84, expected the glyph floor 24
  GREEN ok
query_context_picker_caps_to_a_narrow_viewport
  RED   narrow-viewport popup width was 242, viewport width 240
  GREEN ok
```

The test drives real `egui` frames through `crate::test_frame::frame` and asserts on the returned
`Response::rect`, i.e. the rendered allocation — not on sizing arithmetic.

### F2/F3 — query context picker geometry and dismissal

Test file: `crates/app/src/query_workbench/tests/query_context_picker_view_tests.rs`
Declared from `crates/app/src/query_workbench/query_context_picker_view.rs` via `#[path]`.

```text
cargo test -p db-pro-app query_context_picker
```

RED (before the fix):

```text
query context picker width was 600, expected <= 360px
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 676 filtered out
```

GREEN (after the fix):

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 676 filtered out
```

The geometry test reads the popup's recorded `Area` rect from egui memory, so it measures the
floating surface actually laid out, including the shrink-to-content height for a 2-connection /
1-schema list. The second test proves Escape still returns `QueryContextPickerAction::Close`.

## 2. Quality gates actually executed

| Gate | Command | Result |
|---|---|---|
| Formatting | `cargo +nightly fmt --all -- --check` | exit 0 |
| Clippy (workspace) | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, 0 errors / 0 warnings |
| Type check (workspace) | `cargo check --workspace` | exit 0, `Finished dev profile` |
| UI architecture | `bash scripts/check-ui-architecture.sh` | `UI architecture boundary: PASS` |
| Capture build | `cargo build -p db-pro-native --features capture` | exit 0 (rebuilt after the narrow-viewport fixes) |
| **Native release gate** | `cargo build --release --locked -p db-pro-native` | exit 0 — `Finished release profile [optimized]` |
| **Workspace tests** | `cargo test --workspace` | exit 0 — **2,272 passed / 0 failed / 65 ignored** |
| **Provider suites** (the 65 `#[ignore]`d tests, live fixtures) | `--ignored` on each integration target | exit 0 — `pg_integration` **40/40** · `pg_object_identity` **2/2** · `mysql_integration` **12/12** · `sqlserver_integration` **9/9** · `ssh_backup_runtime_verification` **1/1** · `data_diff_smoke` **1/1** = **65/65** |
| Unit tests (touched crates) | `cargo test -p db-pro-ui -p db-pro-app` | exit 0 — `db-pro-app` 681 passed / 0 failed / 0 ignored; `db-pro-ui` 725 passed / 0 failed / 0 ignored |
| Clean-code diff ratchet | `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | exit 0 — 11 pass / 5 warn / 0 fail (warnings are pre-existing long-file/clone heuristics in untouched files) |

The 65 tests that `cargo test --workspace` reports as ignored are the provider integration suites;
they were then executed against the live fixtures on this host and all passed (see the provider row
above). The `data_diff_smoke` target needs a second writable PostgreSQL database, so a throwaway
`dbpro_diff_target` was created for the run and dropped afterwards — `pg_database` was verified to
contain only `dbpro_fixture` again.

All eight new regression tests appear as `ok` in the workspace run:

```text
components::selection::ui::layout_bounds_tests::{horizontal_radios_do_not_consume_entire_row_width,
  horizontal_checkboxes_do_not_consume_entire_row_width,
  compact_rows_honour_a_parent_wider_than_the_glyph_floor,
  compact_rows_keep_the_glyph_floor_in_a_sliver_parent} ... ok
app::query_context_picker_view::tests::{query_context_picker_has_compact_bounds_for_short_lists,
  query_context_picker_caps_to_a_narrow_viewport,
  query_context_picker_closes_on_click_outside,
  query_context_picker_expands_its_scroll_window_for_long_lists,
  query_context_picker_action_on_escape_closes} ... ok
theme::tests::light_surface_ladder_keeps_hover_and_selected_distinct_from_chrome ... ok
```

Not run: the CI review workflows (no PR was opened).

## 3. Runtime evidence — 17-cell matrix

Deterministic capture harness: `crates/native-app/src/capture.rs` with a new
`DB_PRO_CAPTURE_QUERY_CONTEXT_PICKER` hook (plus `open_query_context_picker_for_capture` in
`crates/app/src/workspace/workspace_actions.rs`) so the picker is opened by code, not by a synthetic
pointer click. Appearance uses the existing `DB_PRO_CAPTURE_FEATURE=settings-appearance` fixture.
Runtime data was isolated to `/private/tmp/dbpro-matrix-verification/data` via `DB_PRO_DATA_DIR`.

Artifacts: `docs/plans/active/viewport-select-fit/evidence/` (PNG + per-run log + manifest with
SHA-256 per image).

| Scenario | Requested | Rendered (logical) | PNG | Process exit |
|---|---|---|---|---|
| appearance-light-1280x800 | 1280×800 | 1280×800 | yes | −6 |
| query-picker-light-1280x800 | 1280×800 | 1280×800 | yes | −6 |
| appearance-dark-1280x800 | 1280×800 | 1280×800 | yes | −6 |
| query-picker-dark-1280x800 | 1280×800 | 1280×800 | yes | −6 |
| appearance-light-1440x900 | 1440×900 | 1440×838 | yes | −6 |
| query-picker-light-1440x900 | 1440×900 | 1440×838 | yes | 0 |
| appearance-dark-1440x900 | 1440×900 | 1440×838 | yes | −6 |
| query-picker-dark-1440x900 | 1440×900 | 1440×838 | yes | −6 |
| appearance-light-1920x1080 | 1920×1080 | 1920×838 | yes | −6 |
| query-picker-light-1920x1080 | 1920×1080 | 1920×838 | yes | −6 |
| appearance-dark-1920x1080 | 1920×1080 | 1920×838 | yes | −6 |
| query-picker-dark-1920x1080 | 1920×1080 | 1920×838 | yes | −6 |
| picker-dense-light-1280x800 (24 long schemas) | 1280×800 | 1280×800 | yes | −6 |
| picker-dense-dark-1280x800 (24 long schemas) | 1280×800 | 1280×800 | yes | 0 |
| picker-short-light-1280x800 (2 conn + 1 schema) | 1280×800 | 1280×800 | yes | 0 |
| explorer-filter-popover-light-1280x800 | 1280×800 | 1280×800 | yes | 0 |
| explorer-filter-popover-dark-1280x800 | 1280×800 | 1280×800 | yes | 0 |

This matrix is the **final** run: it was re-executed after the F5/F6 narrow-viewport fixes and again
after the F7 light-palette change, so the images match the tree described at the top of this file.
It now covers 17 cells: the 12 size × theme cells plus dense-list, short-list and the Explorer
filter popover.

All 17 PNGs were opened and inspected, not inferred from file size.

Exit-code notes:

- `-6` (SIGABRT) is **not** a UI failure. The log for `appearance-light-1280x800` shows
  `NSRangeException … _NSTouchBarFinderObservation … because it is not registered as an observer`,
  the known macOS TouchBar teardown crash that happens **after** the PNG is written. The image and
  manifest entry are complete.
- Rendered heights for the 1440/1920 requests are 838 logical px: this host's desktop clamps the
  window. The matrix must not be described as 900/1080-height verification.

## 4. What the screenshots prove

Every one of the 12 retained PNGs in `evidence/` was opened and read individually (not inferred from
file size or from a different capture). Per cell:

| Cell | What the image shows |
|---|---|
| appearance-light-1280x800 | Light and Dark adjacent right after the sun icon; "Reduce motion" a single compact row; nothing clipped at the card edge |
| appearance-dark-1280x800 | Same adjacency in dark; moon icon leads, both labels fully visible |
| appearance-light-1440x900 | Both choices adjacent; card does not stretch them apart |
| appearance-dark-1440x900 | Both choices adjacent; no clipped glyph at the right boundary |
| appearance-light-1920x1080 | Both choices adjacent at wide width — the gap no longer grows with the window |
| appearance-dark-1920x1080 | Same, dark theme |
| query-picker-light-1280x800 | ~310 logical px popup under the context chip; Connection (2) + Schema (1) shrink-wrapped; long connection name ellipsized, no overflow; no ID banner |
| query-picker-dark-1280x800 | Same compact geometry in dark |
| query-picker-light-1440x900 | Same geometry; popup stays anchored and bounded |
| query-picker-dark-1440x900 | Same geometry, dark |
| query-picker-light-1920x1080 | Same geometry at wide width — width does not scale with the window |
| query-picker-dark-1920x1080 | Same geometry, dark |

**F1 — Appearance.** In every cell, Light and Dark sit adjacent immediately after the sun/moon icon,
wholly inside the card, with no clipped glyph at the window edge. "Reduce motion" now renders through
`components::Checkbox` (component-first rule) and reads as one compact row.

**F2/F3 — Query context picker.** In every cell the popup is a bounded panel anchored under the
context chip: about 300–360 logical px wide, hugging the trigger, with Connection and Schema groups
shrink-wrapped to their rows. The previous ~600 px slab and the two 180 px empty scroll wells are
gone. Long connection labels truncate with an ellipsis instead of widening the surface. No
"First use of ScrollArea ID …" banner is painted over any row in either theme.

The Explorer filter popover is now captured deterministically: the fixture keeps the Explorer activity
in place via `DB_PRO_CAPTURE_REASSERT_SURFACE`, the popover id is a stable `egui::Id`, and
`DB_PRO_CAPTURE_EXPLORER_FILTER_POPOVER` opens it with `egui::Popup::open_id`. Earlier attempts
landed on the Welcome workspace because the activity was overwritten before the frame drew.

## 5. Re-verification history

Two review rounds asked for fresh results. Both were executed; the newest supersedes the older.

**Round 2 — final tree `457cd6457480ff5f23f8a3e5000e9efcc771ad0b`** (this is the tree the evidence
images and the gate table in §2 describe):

| Gate | Result |
|---|---|
| `cargo +nightly fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, 0 diagnostics |
| `cargo check --workspace` | exit 0 |
| `cargo build --release --locked -p db-pro-native` | exit 0 |
| `cargo test --workspace` | exit 0 — 2,272 passed / 0 failed / 65 ignored |
| provider suites (`--ignored`, live fixtures) | exit 0 — 65/65 across six targets |
| `cargo test -p db-pro-ui -p db-pro-app` | exit 0 — 725 + 681 passed |
| `scripts/check-ui-architecture.sh` / cc-ratchet | PASS / 0 fail |

**Round 1 — earlier tree `9e72de50`** (superseded; kept only so the history is not rewritten):
fmt exit 0, clippy exit 0, check exit 0, release build exit 0, workspace 2,269 passed / 0 failed /
65

… [truncated for memory entry; see source file]


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Viewport Select and Radio Allocation Fit

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | omp · implementation lane (two delegated workers failed on provider usage limits; the work was completed in-session) |
| Issue(s) | Owner-reported horizontal overflow and poor complex Select layout; no issue number assigned |
| Task state | `In Progress` — implementation landed and self-verified; the plan stays `RUNTIME_VERIFY` |
| Baseline SHA | `9b50e7a27b4d92de5b126e6ecaf3a518448ad97b` |
| Branch / PR | `main` (direct edit per owner override) · no PR — pushed as `7e81a380` (code: `457cd645`) |
| Scope interpretation | Fix the two reproduced defects (horizontal selection-control allocation, query context picker geometry/identity) plus the component-first violation in the touched Appearance caller, and produce a runtime matrix for the changed surfaces. |
| Out of scope | Application-wide responsive certification; `responsive-layout-primitives` Grid/Container work; database behavior; speculative `SelectItem`/search/typeahead redesign. |

## 2. Progress checkpoint

- Current HEAD: `7e81a380` (docs) — code under test is `457cd6457480ff5f23f8a3e5000e9efcc771ad0b`
- Completed acceptance rows:
  - [x] Horizontal `Radio`/`Checkbox` allocate content width and stay inside the parent
  - [x] Vertical descriptive choices keep the full-row click target
  - [x] Picker width bounded, list regions sized to their rows, no duplicate `ScrollArea` ID banners
  - [x] Light chrome separated from the canvas without collapsing hover/selected states
  - [x] Red→green regression tests measuring rendered geometry (10 new tests: 4 selection, 5 picker, 1 palette ladder)
  - [x] 17-cell runtime matrix for the changed surfaces, every PNG inspected
  - [x] Required Rust gates + locked native release build + the 65 live provider tests
- Remaining acceptance rows:
  - [ ] CI "Rust checks" + independent review (no PR opened)
  - [ ] 900/1080-height verification (host clamps to 838 logical)
  - [ ] Dense data views and the ER canvas re-inspected under the new light chrome step
  - [ ] Deep-scroll traversal of a 24-row schema list (first five rows captured)
- Findings / risks: F1–F7 in `FINDINGS.md`, all P2, all resolved. No P0/P1.
- Tests already run: `cargo test --workspace` exit 0 → 2,272 passed / 0 failed / 65 ignored, and the
  65 ignored provider tests then ran live: pg_integration 40/40, pg_object_identity 2/2,
  mysql_integration 12/12, sqlserver_integration 9/9, ssh_backup_runtime_verification 1/1,
  data_diff_smoke 1/1. Touched-crate subset: `db-pro-ui` 725 + `db-pro-app` 681 passed / 0 failed.
- Dependency / blocker changes: two subagents could not start (`usage_limit_reached`); no scope change.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `457cd6457480ff5f23f8a3e5000e9efcc771ad0b` (code) · docs HEAD `7e81a380` (later docs-only commits do not touch code) |
| Commit list | `9e72de50` fix(ui): cap horizontal selection controls and compact the context picker · `f7669b93` docs(ui-audit): findings, plan and matrix evidence · `f17a1d4b` docs(viewport-select-fit): coding-lane handoff · `1e39b606` docs(viewport-select-fit): workspace gates and re-verification · `be45ed15` docs(viewport-select-fit): pin records to committed SHAs · `8a8f1749` fix(ui): give light chrome its own step above the canvas · `457cd645` fix(ui): keep a distinct hover step above the new light chrome · `ddc6a207` docs(viewport-select-fit): pin verification to the corrected-palette SHA · `493e3b9b` docs(viewport-select-fit): correct the handoff records against the final tree · `5829c0ce` docs(plans): open the layout aesthetics audit · `7e81a380` docs(viewport-select-fit): correct F2 resolution and plan acceptance |
| File / surface inventory | `crates/ui/src/components/selection/ui.rs` (`.compact(bool)` + parent-derived cap) · `selection/API.md` · `selection/tests/layout_bounds_tests.rs` (new, 4 tests) · `radio_group/ui.rs` + `API.md` · `crates/ui/src/theme/tokens/semantic.rs` + `primitive.rs` (light chrome step, `NEUTRAL_525`, dead steps removed) · `crates/ui/src/theme/tests/mod_tests.rs` (palette contract + ladder test) · `app/.../query_context_picker_view.rs` (width constants, allocated list regions, distinct `id_salt`s) · `app/.../tests/query_context_picker_view_tests.rs` (new, 5 tests) · `app/.../settings_appearance_view.rs` (`ui.checkbox` → `components::Checkbox`) · `app/.../explorer_toolbar_view.rs` (stable popover id) · 4 transient popups → `surface_elevated` · `app/.../workspace_actions.rs` + `native-app/src/capture.rs` (picker/dense/popover/light fixtures) · `docs/plans/STATUS.md` · `docs/plans/active/viewport-select-fit/**` · `docs/ui-audit/viewport-select-review/**` |
| Acceptance mapping | Overflow → `horizontal_radios_do_not_consume_entire_row_width`, `horizontal_checkboxes_do_not_consume_entire_row_width` · narrow parent → `compact_rows_honour_a_parent_wider_than_the_glyph_floor`, `compact_rows_keep_the_glyph_floor_in_a_sliver_parent` · picker geometry → `query_context_picker_has_compact_bounds_for_short_lists`, `query_context_picker_caps_to_a_narrow_viewport`, `query_context_picker_expands_its_scroll_window_for_long_lists` · dismissal → `query_context_picker_action_on_escape_closes`, `query_context_picker_closes_on_click_outside` · chrome separation → `light_surface_ladder_keeps_hover_and_selected_distinct_from_chrome` |
| Commands and counts | `cargo +nightly fmt --all -- --check` exit 0 · `cargo clippy --workspace --all-targets -- -D warnings` exit 0 / 0 diagnostics · `cargo check --workspace` exit 0 · `cargo build --release --locked -p db-pro-native` exit 0 · `cargo test --workspace` exit 0, **2,272 passed / 0 failed / 65 ignored** · those 65 ignored provider tests then ran live and passed 65/65 (40 + 2 + 12 + 9 + 1 + 1) · `scripts/check-ui-architecture.sh` PASS · clean-code ratchet exit 0 (11 pass / 5 warn / 0 fail) |
| CI run IDs / status | not run — pushed directly to `main`; GitHub reported bypassed rules ("Changes must be made through a pull request", required check "Rust checks" expected) |
| Known limitations | See §2 remaining rows; picker keyboard coverage is Escape-only (the option rows expose no arrow/Enter handling) while pointer-outside dismissal is covered. |
| Migrations / config implications | None persisted. Opt-in capture env vars: `DB_PRO_CAPTURE_QUERY_CONTEXT_PICKER` (`dense` seeds 24 long-named schemas; no `empty` mode exists — the caller always supplies one schema), `DB_PRO_CAPTURE_EXPLORER_FILTER_POPOVER`, `DB_PRO_CAPTURE_EXPLORER_FILTER_LIGHT`. `.compact(bool)` is additive API; the light palette change is visual only (no persisted preference key changes). |
| Out-of-scope changes | `none` |

Verification detail, matrix table and per-cell observations live in `VERIFICATION.md`.

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | n/a — no independent review performed |
| Verdict | `ACCEPT WITH P2` for the self-verified scope only; **not** an independent approval |
| P0 / P1 / P2 counts | introduced: 0 / 0 / 0 · inherited-and-fixed in scope: 0 / 0 / 7 |
| Findings | F1–F7 in `FINDINGS.md`; F5/F6 were found by the new narrow-width tests failing against the first implementation, F7 by the owner's light-region feedback, and the hover/panel collision by review of the first palette attempt |
| CI disposition | not executed |
| Next task(s) unblocked | Open a PR for independent review; re-inspect dense data views and the ER canvas under the new light chrome step; drive a deep-scroll traversal of a long schema list |

## 5. Research / audit handoff

n/a — this is the implementation lane for the assessment recorded in
`docs/ui-audit/viewport-select-review/AGENT_EVIDENCE.md`.

## 6. Tổng kết (Vietnamese summary)

Đã sửa hai lỗi được báo: (1) Radio/Checkbox trong layout ngang chiếm hết chiều rộng còn lại nên
Appearance đẩy "Dark" ra mép và cắt chữ — nay có `.compact(bool)`, mặc định suy ra từ hướng layout
của parent và cap theo chiều rộng parent đọc **trước** khi vào hàng con; (2) popup connection/schema
chỉ đặt `min_width` nên phình ~600pt và hai `ScrollArea` không co, trùng ID — nay width bị chặn theo
viewport (trừ cả inset của surface), hai vùng co theo nội dung và có `id_salt` riêng.

Test biên hẹp do reviewer yêu cầu đã **fail với implementation đầu tiên** và phát hiện hai lỗi thật
(F5: cap đọc sai nguồn width; F6: thiếu inset surface) trước khi xanh.

Gate đã chạy thật trên cây cuối (`457cd645`): fmt 0, `cargo check --workspace` 0, `cargo clippy
--workspace --all-targets -D warnings` 0 chẩn đoán, ui-architecture PASS, **release build locked 0**,
`cargo test --workspace` **2.272 pass / 0 fail / 65 ignored**; 65 test provider `#[ignore]` sau đó chạy
live và **pass 65/65** (pg 40 + object_identity 2 + mysql 12 + sqlserver 9 + ssh 1 + data_diff 1).
Matrix 17 ô được capture lại từ cây cuối và đọc từng ảnh; 1440/1920 bị host clamp còn 838 logical,
các ô `exit=-6` là crash TouchBar macOS **sau khi** ghi ảnh — không phải lỗi UI.

Về độ tách vùng: panel sáng đổi `#f9f9f9` → `#f6f6f6` (một bậc rõ so với canvas trắng), nhưng **cố ý
không dùng `#f1f1f1`** vì trùng `hover` — nếu dùng thì hover trên sidebar không nhìn thấy. Bậc thang
canvas → panel → hover → selected giờ có test khoá. Chữ tertiary/muted nâng lên `#6b6b6b` (4.93:1).

Còn thiếu: CI/review độc lập (chưa mở PR), chiều cao 900/1080 thật, soi lại dense data views và ER
canvas dưới bậc sáng mới, và cuộn sâu trong danh sách 24 schema. Vì vậy plan giữ `RUNTIME_VERIFY`,
chưa `COMPLETED`. Code đã push lên `main` theo owner override (code `457cd645`, docs tới
`ddc6a207`).

