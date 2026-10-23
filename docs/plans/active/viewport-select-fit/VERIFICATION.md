# Viewport Select and Radio Allocation Fit — Verification

**Source reference:** `HEAD = 9b50e7a27b4d92de5b126e6ecaf3a518448ad97b` **plus uncommitted
working-tree changes** (this fix was not committed). The baseline SHA alone does not identify the
fixed tree. Working-tree reference at the time of the runs below:

```text
git rev-parse HEAD  -> 9b50e7a27b4d92de5b126e6ecaf3a518448ad97b
git diff --stat     -> 9 files changed, 181 insertions(+), 53 deletions(-)
sha256 crates/ui/src/components/selection/ui.rs
  -> 35eff9da323074c830f607cd93d3c83d0b85d4b4419ddcb3dc937580d61614f8
sha256 crates/app/src/query_workbench/query_context_picker_view.rs
  -> c458992e61401e581fc6e2afdfdd249ce91df7c5c79010e618312ab6e0bf4536
```

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
| **Workspace tests** | `cargo test --workspace` | exit 0 — **2,269 passed / 0 failed / 65 ignored** |
| Unit tests (touched crates) | `cargo test -p db-pro-ui -p db-pro-app` | exit 0 — `db-pro-app` 679 passed / 0 failed / 0 ignored; `db-pro-ui` 724 passed / 0 failed / 0 ignored |
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
| appearance-light-1280x800 | 1280×800 | 1280×800 | yes | 0 |
| appearance-dark-1280x800 | 1280×800 | 1280×800 | yes | 0 |
| appearance-light-1440x900 | 1440×900 | 1440×838 | yes | −6 |
| appearance-dark-1440x900 | 1440×900 | 1440×838 | yes | 0 |
| appearance-light-1920x1080 | 1920×1080 | 1920×838 | yes | 0 |
| appearance-dark-1920x1080 | 1920×1080 | 1920×838 | yes | 0 |
| query-picker-light-1280x800 | 1280×800 | 1280×800 | yes | −6 |
| query-picker-dark-1280x800 | 1280×800 | 1280×800 | yes | −6 |
| query-picker-light-1440x900 | 1440×900 | 1440×838 | yes | −6 |
| query-picker-dark-1440x900 | 1440×900 | 1440×838 | yes | −6 |
| query-picker-light-1920x1080 | 1920×1080 | 1920×838 | yes | −6 |
| query-picker-dark-1920x1080 | 1920×1080 | 1920×838 | yes | −6 |

This matrix is the **final** run: it was re-executed after the F5/F6 narrow-viewport fixes so the
images match the tree described at the top of this file, not the earlier intermediate build.

All 12 PNGs were opened and inspected, not inferred from file size.

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

Re-run at `f17a1d4b931e3090322e71e2c701e064dd20d694` (the pushed HEAD) after a reviewer asked for
fresh results:

| Gate | Result |
|---|---|
| `cargo +nightly fmt --all -- --check` | exit 0 |
| `bash scripts/check-ui-architecture.sh` | `UI architecture boundary: PASS` |
| `cargo test --workspace` | exit 0 — 2,269 passed / 0 failed / 65 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, 0 diagnostics |
| `cargo check --workspace` | exit 0 |
| `cargo build --release --locked -p db-pro-native` | exit 0 |

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

## 6. Honest limitations

- Matrix heights above 838 logical px are unavailable on this host; the requested 900/1080 cells are
  recorded as clamped, not as verified at those heights.
- Only the short-list state (2 connections, 1 schema) was captured. Dense PostgreSQL schema lists
  beyond the 180 px bound and long connection names were exercised only by the unit test's bounds
  assertion, not by a screenshot.
- Loading/error/empty states of the picker were not captured. The empty-list labels added in the
  picker are covered by reading, not by a screenshot.
- Keyboard journey inside the open picker (arrow movement, Enter/Escape) is covered for Escape only.
- Pointer-driven dismissal (`clicked_outside`) was not re-tested here.
- No PostgreSQL or SQLite database behavior changed; the picker is a pure UI surface fed by
  already-loaded catalog/schema state. Provider matrix: PostgreSQL `N/A`, SQLite `N/A` (layout-only).
- `cargo test --workspace` passes, but its 65 `#[ignore]`d provider tests did not run; the CI review
  workflows were not executed (no PR).
