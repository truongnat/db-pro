# Core Component Style System — Checklist

## Discovery
- [x] Confirm target audience, use cases, tone and screenshot reference.
- [x] Choose dark database workstation direction with compact shadcn-inspired controls.
- [x] Define first batch boundary: theme, tokens, Button, Input, Card, focus/interaction.

## Implementation
- [x] Recalibrate dark `DbProTheme` surfaces, borders, text and accent.
- [x] Align shared radius, spacing, control-height and typography tokens.
- [x] Restyle Button variants without changing public API or click semantics.
- [x] Restyle Input family with clear idle/hover/focus/error states through shared theme tokens.
- [x] Restyle Card and shared focus treatment through shared surface/border tokens.
- [x] Add or update characterization tests for visual-token contracts.
- [x] Fix gallery primitive overflow: wrap Checkbox/Radio copy to local cell width.
- [x] Make unavailable controls explicit and visually legible in disabled state.
- [x] Recompose Disclosure & Choice into three compact panels without nested outer-card chrome.
- [x] Show Calendar inline, persist gallery toggle/calendar state, and fix Accordion icon/title spacing.
- [x] Keep Calendar surface at intrinsic width instead of stretching to the gallery column.
- [x] Reflow the gallery and scale Calendar cells when the available column is narrower than the seven-column grid.
- [x] Constrain Calendar header navigation to the same three fixed regions as the grid.
- [x] Clamp DatePicker popup horizontally and flip it above the trigger when the viewport bottom is tight.
- [x] Add focused Calendar layout coverage and a capture target for the affected surface.
- [x] Recompose Cards & Metric Displays with a responsive grid that avoids orphan cards at medium widths.
- [x] Increase metric title contrast through the semantic theme token and capture the light surface at all target widths.

## Verification
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `cargo build --release --locked -p db-pro-native`
- [x] Runtime evidence at 1280×800, 1440×900, 1920×1080 for Disclosure & Choice; captures are in `evidence/disclosure-choice-controls-*.png`.
- [x] Runtime evidence for Calendar & DatePicker at 1280×800, 1440×900, and 1920×1080; captures are in `evidence/calendar-datepicker-*.png`.
- [x] Runtime evidence for Cards & Metric Displays at 1280×800, 1440×900, and 1920×1080; captures are in `evidence/cards-metrics-*.png`.
- [ ] Review loading, error and empty states where the batch is visible.

## Open follow-up
- [ ] Apply calibrated core primitives to shell/data-grid/sidebar in the next batch.
- [ ] Capture independent accessibility/focus evidence for keyboard interactions.
- [ ] Expand DatePicker interaction coverage for keyboard focus/navigation, direct text entry, min/max or disabled dates, locale/week-start configuration, and clear/today actions.

## Reusable UI lessons
- [x] Content tabs use the same subtle fill for active and hover states; avoid a decorative bottom underline that makes the selected tab look like a different component state.
- [x] Pin data-grid pagination and Save, Discard, Refresh to the bottom of the grid viewport; reserve the top toolbar for row creation, filters, and sorting. In egui, `allocate_ui_with_layout` can shrink to its content, so make the inner grid frame claim the reserved viewport height.
- [x] Keep the Data Grid's primary header focused on Add Row and direct SQL entry; execute through the existing query safety/runtime path while routing results back into the grid without opening a Query tab.

## Data Grid footer follow-up
- [x] Pin pagination below the result grid at the viewport bottom; place Save, Discard, Refresh on the footer's left and pagination on the right.
- [x] Keep Save/Discard visible but disabled without staged changes; preserve refresh blocking while staged changes exist.
- [x] Verify normal table-data layout at 1280×800, 1440×900, and 1920×1080 capture targets; higher-width captures are host-height capped.
- [ ] Capture loading, error, and empty table-data states; those states currently render the placeholder instead of the grid footer.

## Data Grid inline query follow-up
- [x] Replace top filter and sort controls with a direct SQL input beside Add Row; Enter executes inline and routes returned rows to the Data Grid.
- [x] Guard against staged mutations and an already-running query; do not create or activate a Query document.
- [x] Preserve removable active filter chips below the primary header and keep Save, Discard, Refresh, and pagination in the bottom footer.
- [x] Verify the normal Data Grid header at 1280×800, 1440×900, and 1920×1080; higher-width captures are host-height capped and saved in `evidence/data-grid-query-header-*.png`.
- [x] Verify inline SQL submission and returned rows against the live SQLite sample database; PostgreSQL runtime remains unverified.

## Data Grid clarity and one-page follow-up
- [x] Assemble a fixed, quoted `SELECT * FROM <schema>.<table> WHERE` prefix internally and show only a compact `WHERE` badge; let the user enter only the condition.
- [x] Keep Add Row first in the toolbar and enabled for writable table connections while inline filtering is active; preserve connection and in-flight-query guards.
- [x] Hide page-navigation controls when neither a previous nor next page exists; keep the row range and page-size selector.
- [x] Add focused tests for safe identifier quoting and single-page/adjacent-page navigation visibility.
- [x] Capture and inspect the updated native surface at 1280×800, 1440×900, and 1920×1080; 1440×900 and 1920×1080 were host-height capped to 838 logical pixels. Evidence is in `evidence/data-grid-condition-light-*.png`.

## Data Grid WHERE badge and Add Row follow-up
- [x] Hide the full SQL prefix in the toolbar and retain only a compact `WHERE` badge before the condition field.
- [x] Reproduce and fix Add Row being disabled on a filtered table result; keep cell editing guarded separately.
- [x] Verify the filtered SQLite result and open the Insert Row dialog without staging or saving data.
- [x] Capture normal table-data state at 1280×800, 1440×900, and 1920×1080 in `evidence/data-grid-where-badge-light-*.png`.

## Data Grid filtered-save refresh follow-up
- [x] Retain the last successfully executed inline SQL separately from the editable condition draft.
- [x] Re-run that SQL after staged changes are saved and when refreshing a filtered table; clear it when switching tables.
- [x] Add a command-boundary regression test covering inline query success → save completion → same filtered SQL dispatched.

## Data Grid condition autocomplete follow-up
- [x] Suggest only columns from the selected table and filter them by the identifier prefix at the caret.
- [x] Support Up/Down selection, Enter/Tab insertion, Escape close, pointer selection, and preserve focus after insertion.
- [x] Keep suggestions out of SQL values and quoted string literals; quote inserted column identifiers safely.
- [x] Add focused coverage for scope, prefix matching, value/literal suppression, and replacing an identifier around a middle caret.
- [ ] Capture and inspect the open suggestions popup in the native UI at the required viewport sizes; interactive visual proof remains pending.

## Tổng kết bằng tiếng Việt
Checklist tách rõ việc chuẩn hóa core components khỏi việc lan style ra toàn bộ shell/data-grid/sidebar. Runtime evidence vẫn là gate riêng, không suy diễn từ source/test.

## WHERE input geometry correction — 2026-10-06
- [x] WHERE is a fixed prefix with its own themed fill, with predicate entry immediately after it.
- [x] Input height uses the same small-button token as Add Row and Run.
- [x] Explicit gap before Run; reserve its width on the same toolbar row.
- [ ] Full loading/error/empty and interactive runtime matrix remains pending.

## Empty WHERE executes unfiltered SELECT — 2026-10-06
- [x] Run stays enabled for an empty draft.
- [x] Run/Enter with empty or whitespace-only draft generates SELECT without WHERE.
- [x] Nonempty draft still generates WHERE and uses existing mutation/running-query guards and result row limits.
- [ ] Live PostgreSQL/SQLite empty Run/Enter interactions remain pending.
