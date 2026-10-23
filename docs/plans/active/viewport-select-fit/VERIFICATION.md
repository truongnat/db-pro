# Viewport Select and Radio Allocation Fit — Verification

**Source reference (committed):**

```text
code under test  8a8f1749334065b656f6c08a06c6c520f12737d9  fix(ui): give light chrome its own step …
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
| Unit tests (touched crates) | `cargo test -p db-pro-ui -p db-pro-app` | exit 0 — `db-pro-app` 681 passed / 0 failed / 0 ignored; `db-pro-ui` 725 passed / 0 failed / 0 ignored |
| Clean-code diff ratchet | `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | exit 0 — 11 pass / 5 warn / 0 fail (warnings are pre-existing long-file/clone heuristics in untouched files) |

The 65 ignored tests are the pre-existing `#[ignore]`d provider suites that need live
PostgreSQL/MySQL/SQL Server/SSH fixtures; ignored tests are not counted as passing. All seven new
regression tests appear as `ok` in the workspace run:

```text
components::selection::ui::layout_bounds_tests::{horizontal_radios_do_not_consume_entire_row_width,
  horizontal_checkboxes_do_not_consume_entire_row_width,
  compact_rows_honour_a_parent_wider_than_the_glyph_floor,
  compact_rows_keep_the_glyph_floor_in_a_sliver_parent} ... ok
app::query_context_picker_view::tests::{query_context_picker_has_compact_bounds_for_short_lists,
  query_context_picker_caps_to_a_narrow_viewport,
  query_context_picker_action_on_escape_closes} ... ok
```

Not run: the CI review workflows and the Explorer-filter capture (see §4).

## 3. Runtime evidence — 12-cell matrix

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

Not used as evidence: `DB_PRO_CAPTURE_FILTER` opened the Welcome workspace instead of the Explorer
filter popover on this build, so the Explorer toolbar radios (`explorer_toolbar_view.rs`) have unit
coverage only, not a screenshot.

## 5. Re-verification on the pushed tree

Re-run against code SHA `9e72de50fa821926a3874949e2ebc65289885d0a` at docs HEAD `1e39b606`, after
a reviewer asked for fresh results:

| Gate | Result |
|---|---|
| `cargo +nightly fmt --all -- --check` | exit 0 |
| `bash scripts/check-ui-architecture.sh` | `UI architecture boundary: PASS` |
| `cargo test --workspace` | cargo exited 0 — 2,269 passed / 0 failed / 65 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, 0 diagnostics |
| `cargo check --workspace` | exit 0 |
| `cargo build --release --locked -p db-pro-native` | exit 0 |

The workspace run was sequential (tests → clippy → check → release build) to avoid Cargo-lock
contention. The 65 ignored entries are the pre-existing `#[ignore]`d provider integration suites,
skipped because no live PostgreSQL/MySQL/SQL Server/SSH fixtures were provisioned; they are not
counted as passing.

Runtime smoke on the same tree (fresh temp data dir, not overwriting the committed evidence):

- `appearance-dark-1280x800` re-captured byte-identically — SHA-256
  `2bc6fd89fa758679cc8ef4322963d8a2a11efa6700da0ee1f4ca64b222b1d73`, equal to the committed
  manifest entry. The fix is deterministic.
- `query-picker-dark-1280x800` re-captured with the same compact geometry, but its SHA differs from
  the committed image because the status bar prints the data-directory path, which differs between
  runs. Visual inspection confirms identical layout; the hash difference is not a layout change.

Operational note: `cargo test --workspace` rebuilds `target/debug/db-pro-native` **without** the
`capture` feature, so a capture run after the test suite must first re-run
`cargo build -p db-pro-native --features capture`. Skipping that makes `DB_PRO_CAPTURE_TO` inert and
the process run until timeout (observed as exit 124 with no PNG).

## 6. Theme chrome separation (F7)

Owner feedback: in light mode the regions read as one flat surface, and the sidebar should sit
slightly above the canvas.

| Change | Before | After |
|---|---|---|
| Light `background.panel` (sidebar, topbar, status bar) | `#f9f9f9` | `#f6f6f6` |
| Light `tertiary` / `muted` text | `#707070` | `#6b6b6b` (4.93:1 on the new panel) |
| Transient popups (completion, hover, signature help, column suggestions) | `surface_panel` | `surface_elevated` (documented role) |
| Dark tokens | — | unchanged |

The panel value stops one step short of `hover` (`#f1f1f1`) on purpose: `#f1f1f1` would have made a
hovered sidebar row identical to the base fill. The light ladder is now
canvas `#ffffff` → panel `#f6f6f6` → hover `#f1f1f1` → selected `#eeeeee`, and a new test
(`light_surface_ladder_keeps_hover_and_selected_distinct_from_chrome`) asserts both the inequality
and the descending luminance order so a future palette edit cannot silently collapse a state.

`crates/ui/src/theme/tests` keeps enforcing 4.5:1 for primary/secondary/tertiary on both the canvas
and the panel, and `light_tokens_follow_codex_surface_contract` now pins `#f6f6f6`. The light ramp
steps the change orphaned (`NEUTRAL_50`, `NEUTRAL_500`) were deleted — Clippy's `dead_code` caught
them, which is why the workspace Clippy gate is reported as exit 0 only after that deletion.

Observed in the re-captured evidence: in every light cell the sidebar/topbar is a distinct step
below the white editor, cards and the picker popup read as raised, and no text lost legibility. Dark
cells are pixel-comparable to the previous run because none of their tokens changed.

## 7. Honest limitations

- Matrix heights above 838 logical px are unavailable on this host; the requested 900/1080 cells are
  recorded as clamped, not as verified at those heights.
- Picker keyboard coverage is Escape-only (arrow/Enter navigation inside the popup is not
  implemented by `paint_option`, so there is nothing to drive); pointer-outside dismissal is covered
  by a unit test.
- The picker's "empty" state is unreachable by design: the caller always yields at least one entry
  (`main` for engines without schemas, `public` otherwise), so no empty-state branch is shipped.
- Light-palette separation was judged from the captured surfaces (workspace, Appearance, picker,
  Explorer popover). Dense data views and the ER canvas were not re-inspected for the new step.
- Dense schema lists are captured with 24 long-named entries (5 visible rows inside the bounded
  window); scrolling deeper than the first rows and the full 24-row traversal were not driven.
- The picker has no empty state to capture: the caller always supplies at least one schema, and the
  defensive empty branches were removed rather than shipped untested.
- Keyboard journey inside the open picker is Escape-only; `paint_option` exposes no arrow/Enter
  handling, so the popup is pointer-select plus Escape. Pointer-outside dismissal has a unit test.
- No PostgreSQL or SQLite database behavior changed; the picker is a pure UI surface fed by
  already-loaded catalog/schema state. Provider matrix: PostgreSQL `N/A`, SQLite `N/A` (layout-only).
- `cargo test --workspace` passes, but its 65 `#[ignore]`d provider tests did not run; the CI review
  workflows were not executed (no PR).
