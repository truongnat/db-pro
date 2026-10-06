# Core Component Style System — Verification

## Current checkpoint
- Baseline commit: `dc87da1d`.
- Design context confirmed by the user: database developers/analysts; dark database workstation; shadcn Primitives & Controls reference.
- Source changes: `theme.rs`, `tokens.rs`, Button config/handler, selection wrapping, Accordion spacing, Calendar intrinsic sizing/popup clamping, responsive narrow-column Calendar scaling, fixed-width Calendar header regions, responsive gallery panel layout, focused gallery capture routing, gallery panel state, plan files, and `.impeccable.md`.
- `cargo fmt --all -- --check`: PASS.
- `cargo test -p db-pro-ui theme::tests --lib`: PASS (4 passed).
- `cargo test -p db-pro-ui components::button --lib`: PASS (7 passed).
- `cargo test -p db-pro-ui components::input --lib`: PASS (13 passed).
- `cargo test -p db-pro-ui --lib`: PASS (929 passed, 0 failed, 0 ignored).
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS; the `db-pro-ui` segment reported 929 passed / 0 failed / 0 ignored, with provider/SSH/SQL Server fixture cases ignored where external fixtures were unavailable.
- `cargo build --release --locked -p db-pro-native`: PASS.
- Clean-code scan: PASS (14 pass, 2 warning categories, 0 fail); warnings are inherited numeric casts and existing long UI methods.
- Runtime evidence: disclosure/choice captures written to [`evidence/disclosure-choice-controls-1280x800.png`](evidence/disclosure-choice-controls-1280x800.png), [`evidence/disclosure-choice-controls-1440x900.png`](evidence/disclosure-choice-controls-1440x900.png), and [`evidence/disclosure-choice-controls-1920x1080.png`](evidence/disclosure-choice-controls-1920x1080.png).
- The captures verify the three-panel composition, inline Calendar preview, compact Toggle/ToggleGroup, readable Accordion icon/title spacing, and responsive behavior at all three viewports. The narrow capture also verifies that header navigation does not expand the right side beyond the grid.
- Calendar verification: the capture-only `calendar` gallery target produced [`evidence/calendar-datepicker-800x800.png`](evidence/calendar-datepicker-800x800.png), [`evidence/calendar-datepicker-1280x800.png`](evidence/calendar-datepicker-1280x800.png), [`evidence/calendar-datepicker-1440x900.png`](evidence/calendar-datepicker-1440x900.png), and [`evidence/calendar-datepicker-1920x1080.png`](evidence/calendar-datepicker-1920x1080.png). The narrow capture proves all seven columns remain inside the frame; wider captures retain the intrinsic size and balanced margins.
- Calendar unit tests: `cargo test -p db-pro-ui components::calendar:: --lib` → 10 passed / 0 failed / 919 filtered.
- Loading/error/empty state traversal remains pending.

## Cards & Metric Displays surface follow-up
- Implementation SHA: `8ca9cac5470ec6688c52aff4cc7e09a0aacdad16` (`polish(ui): tighten metric card surface`).
- Source changes: `component_gallery_surfaces.rs` replaces the fixed two-column rows with a responsive metric grid; `components/card/ui.rs` promotes metric labels to `text_secondary`; `workspace_actions.rs` adds a light-theme/cards capture route.
- Runtime evidence: [`evidence/cards-metrics-1280x800.png`](evidence/cards-metrics-1280x800.png), [`evidence/cards-metrics-1440x900.png`](evidence/cards-metrics-1440x900.png), and [`evidence/cards-metrics-1920x1080.png`](evidence/cards-metrics-1920x1080.png).
- Visual result: medium widths remain balanced at 2×2 without a one-card orphan row; the wide target uses four columns; labels remain legible in the light semantic theme.
- Verification after the implementation SHA: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (UI segment: 929 passed), `cargo build --release --locked -p db-pro-native`, clean-code scan (14 pass / 2 warning categories / 0 fail), and `git diff --check` all passed.
- Loading/error/empty state traversal for this static gallery surface remains pending.

# Data Grid condition autocomplete — 2026-10-04

- Baseline source SHA: `710ba002612c6d71ef2605f99f2a49743b51c3a4`; implementation is uncommitted in the working tree, so this SHA identifies the base only.
- `cargo fmt --all` → exit 0.
- `cargo check --locked -p db-pro-ui` → exit 0.
- `cargo test --locked -p db-pro-ui table_data_toolbar_surface_view::tests -- --nocapture` → 4 passed / 0 failed / 0 ignored, exit 0.
- `cargo clippy --locked -p db-pro-ui --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all -- --check` and `git diff --check` → exit 0.
- `cargo build --release --locked -p db-pro-native` → exit 0.
- CUA runtime capture/interaction → not verified. The previous DB Pro Measure app could not be reattached after relaunch; the new release binary started from the shell but was not exposed as a CUA app target.
- PostgreSQL / SQLite runtime matrix: not applicable to completion matching; no database request is issued until the existing Run action is invoked.

## Data Grid clarity and one-page follow-up — 2026-10-04
- `cargo fmt --all` and `git diff --check`: PASS.
- `cargo check -p db-pro-ui --lib`: PASS.
- `cargo test -p db-pro-ui table_data_pagination_view::tests --lib`: 2 passed / 0 failed / 0 ignored, exit 0.
- `cargo build --release --locked -p db-pro-native`: PASS.
- Native runtime evidence: `db-pro-native` opened the SQLite sample database. Captures at 1280×800, 1440×900, and 1920×1080 are in [`evidence/data-grid-clarity-light-1280x800.png`](evidence/data-grid-clarity-light-1280x800.png), [`evidence/data-grid-clarity-light-1440x900.png`](evidence/data-grid-clarity-light-1440x900.png), and [`evidence/data-grid-clarity-light-1920x1080.png`](evidence/data-grid-clarity-light-1920x1080.png). The 1440×900 and 1920×1080 targets were capped by the host display to 838 logical pixels high (physical captures 2880×1676 and 3840×1676); 1280×800 measured 2560×1600 physical pixels.
- Live SQLite interaction: Run with `SELECT * FROM customers WHERE id = 1` and Enter with `SELECT * FROM customers WHERE id = 2` each rendered exactly the matching row; the native status reported `Query completed · 1 rows`. This exercised the read-only sample database only; PostgreSQL remains unverified.
- Startup emitted missing replacement-glyph warnings (`◻` / `?`); they did not prevent rendering or query execution.
- Visual changes under test: single-page pagination hides navigation controls while retaining row range/page size; SQL draft is explicitly labeled and has a visible Run action.

## DBeaver-style WHERE condition follow-up — 2026-10-04
- `cargo fmt --all -- --check` and `git diff --check`: PASS.
- `cargo test -p db-pro-ui table_data_toolbar_surface_view::tests --lib`: 1 passed; `cargo test -p db-pro-ui table_data_pagination_view::tests --lib`: 2 passed.
- `cargo build --release --locked -p db-pro-native --features capture`: PASS.
- Live SQLite UI: condition `id = 2` submitted with Enter rendered Bob; `id = 1` submitted with Run rendered Alice; both status messages reported one row.
- Normal-state captures: [`evidence/data-grid-condition-light-1280x800.png`](evidence/data-grid-condition-light-1280x800.png), [`evidence/data-grid-condition-light-1440x900.png`](evidence/data-grid-condition-light-1440x900.png), and [`evidence/data-grid-condition-light-1920x1080.png`](evidence/data-grid-condition-light-1920x1080.png). Physical captures are 2560×1600, 2880×1676, and 3840×1676 pixels; host display caps the latter two to 838 logical pixels high. Capture driver uses deterministic PostgreSQL fixture data.
- The rebuilt UI shows Add Row before the quoted fixed prefix, with only the predicate editable. PostgreSQL live execution and loading/error/empty capture states remain unverified.

## WHERE badge and Add Row follow-up — 2026-10-04
- Baseline / source identity: HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4`; working-tree blob IDs: `table_data_toolbar_surface_view.rs` `e8dda4c6f471ef2ff728fc593e5bfc305460e841`, `table_data_view.rs` `9ebcd0770650ea7c24b9deff1c3da0caacf2a5ca`.
- Red/green regression: focused toolbar-capability test failed before the fix (0 passed / 1 failed, exit 101); after the fix, `cargo fmt --all -- --check && cargo test -p db-pro-ui table_data_view::tests --lib` passed (2 passed / 0 failed, exit 0). Live SQLite interaction separately verified the filtered-result case.
- Build: `cargo build --release --locked -p db-pro-native --features capture` passed after the UI changes. Earlier focused prefix and pagination tests remain recorded above.
- Live SQLite interaction: entered `id = 2`, pressed Enter, and saw Bob with `Query completed · 1 rows`; clicked Add Row and verified the `Insert Row · customers` form opens while filtered. Closed with Escape without staging or saving; refreshed and confirmed all three original rows remain.
- Fresh captures: `evidence/data-grid-where-badge-light-1280x800.png` (2560×1600 physical), `evidence/data-grid-where-badge-light-1440x900.png` (host-capped to 838 logical px high, 2880×1676 physical), and `evidence/data-grid-where-badge-light-1920x1080.png` (host-capped to 838 logical px high, 3840×1676 physical). The 1280 capture was inspected; captures use deterministic PostgreSQL fixture data, not a live PostgreSQL connection.
- Result: the full prefix is no longer visible; `WHERE` is the only prefix badge. Add Row and Save toolbar availability no longer follow the filtered result-set flag, while cell edit restrictions remain independent.
- Limitations: live PostgreSQL not verified. No database insert was committed; the verified interaction covers opening the insert form only. Loading/error/empty states remain pending.

## Filtered result after staged save — 2026-10-04
- Baseline HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4`. Working-tree source blob IDs: `table_data_query_state.rs` `59b69eebaa9936226017b42c0ffc45ee64ea8f29`; `table_editor_view.rs` `fcb02fb218c2314fa27446f456133d16cc10504d`; `events_query_dispatch.rs` `237c8ba0b4020ddffcabd7b5738307eb952d57df`; `query_result_events.rs` `09b241274eb0ec572e42e027cd9b95076b601fea`; `runtime_event_handlers.rs` `392748a2fb85af9f3db3a2335eac53e5e7d8bfb7`; `app_tests.rs` `c7493a1668bd87e9a2de25eda6986c0b7cc6674f`.
- Red/green: `cargo test -p db-pro-ui staged_save_completion_reruns_the_active_inline_sql --lib` failed before refresh routing (0 passed / 1 failed, exit 101); passed after (1 passed / 0 failed, exit 0).
- Query state: `cargo test -p db-pro-ui table_data_query_state::tests --lib` → 5 passed / 0 failed / 0 ignored, exit 0. `cargo fmt --all -- --check` and `git diff --check` passed.
- Native build: `cargo build --release --locked -p db-pro-native --features capture` passed.
- The regression test simulates a successfully applied inline `SELECT`, completes staged-save handling, receives the next runtime command, and verifies it is the same filtered `RunQuery`. Live database insert/save was not repeated; no new user data was written during this verification.
- Behavior: only a successful inline query becomes the active refresh query. Query failure/cancellation does not replace it; table reset clears it. Manual Refresh also preserves the currently executed condition.

## ResponsiveGrid vertical-gap follow-up
- Implementation SHA: `42d78e203b67bde917438ec424a8f9c8a2d206b` (`fix(ui): add vertical responsive grid gaps`).
- Root cause: `ResponsiveGrid` configured only horizontal `item_spacing`; successive rows inherited no intentional vertical separation and metric cards visually touched.
- Fix: the primitive now owns both axes of spacing, neutralizes inherited vertical spacing inside its local scope, and inserts the shared gap between rendered rows. The focused test asserts the exact 8px row gap for a two-row grid.
- Runtime evidence refreshed at 1280×800 and 1440×900; the 1920×1080 wide layout remains one row and therefore has no inter-row gap to render.
- Verification: focused responsive-grid test passed; full workspace tests (UI segment: 929 passed), fmt, workspace check, clippy, release build, clean-code scan (13 pass / 3 warning categories / 0 fail), and diff check passed.

## Data Grid condition autocomplete redesign — 2026-10-04
- Baseline HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4`; source remains uncommitted in the shared working tree.
- Behavior: completion waits for a column-name prefix or a new-condition position (`AND`, `OR`, `(`); it does not open on an empty focused field or while entering values. Suggestions use current-table columns, selected with Up/Down and accepted with Enter/Tab. Common safe identifiers such as `email` are inserted without quotes; unsafe or keyword identifiers are quoted.
- Visual treatment: compact themed editor, foreground dropdown capped at seven rows and 420 logical pixels, with column type shown and no persistent keyboard-help footer. The dropdown overlays the grid without changing its layout.
- Focused tests: `cargo test --locked -p db-pro-ui table_data_toolbar_surface_view::tests --lib` → 6 passed / 0 failed / 0 ignored. `cargo fmt --all -- --check`, `git diff --check`, `cargo clippy --locked -p db-pro-ui --all-targets -- -D warnings`, and `cargo build --release --locked -p db-pro-native --features capture` passed.
- Runtime visual evidence (captured and inspected): [`evidence/data-grid-condition-autocomplete-light-1280x800-final.png`](evidence/data-grid-condition-autocomplete-light-1280x800-final.png), [`evidence/data-grid-condition-autocomplete-light-1440x838-host-capped-final.png`](evidence/data-grid-condition-autocomplete-light-1440x838-host-capped-final.png), and [`evidence/data-grid-condition-autocomplete-light-1920x838-host-capped-final.png`](evidence/data-grid-condition-autocomplete-light-1920x838-host-capped-final.png). Physical captures are 2560×1600, 2880×1676, and 3840×1676 pixels; host display limits heights above 838 logical pixels.
- The completion capture route seeds `em`, focuses the field once, and captures the `email` result. This verifies the rendered suggestion and layout. Live keyboard acceptance and query execution were not exercised in this pass; PostgreSQL remains a deterministic UI fixture rather than an independently verified live provider.
- Remaining verification: interactive key/mouse acceptance in the running app and PostgreSQL/SQLite provider matrix.

## Tổng kết bằng tiếng Việt
Các gate đã PASS: fmt, workspace tests (929 UI tests), clippy và native release build. Evidence đã đủ narrow 800×800 cùng 1280×800, 1440×900 và 1920×1080; header và grid đều đã kiểm tra runtime. Chỉ còn traversal loading/error/empty của gallery.

Autocomplete WHERE cũng đã qua 6 focused tests, clippy và native capture build; popup gợi ý `email` đã được chụp và kiểm tra ở ba viewport. Chưa kiểm tra trực tiếp phím/chọn chuột hoặc chạy query từ popup; hai viewport lớn bị giới hạn chiều cao bởi màn hình host.

## WHERE input geometry correction — 2026-10-06
- Source identity: HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted `crates/ui/src/table_data_toolbar_surface_view.rs` changes.
- `cargo fmt --all -- --check`, `git diff --check`, `cargo check --locked -p db-pro-ui --lib`, and `cargo build --release --locked -p db-pro-native --features capture`: PASS (exit 0). The first compile attempt failed on a missing module qualification; corrected before the successful final build.
- Captured and inspected: `evidence/where-input-alignment-1280x800.png` (focused draft `em` with completion), `evidence/where-input-alignment-1440x900.png`, and `evidence/where-input-alignment-1920x1080.png` (empty input with disabled Run). Physical dimensions: 2560×1600, 2880×1676, 3840×1676; larger targets capped to 838 logical pixels high by host.
- Native framebuffer evidence verifies passive shaded prefix, predicate immediately after WHERE, matching control heights, and Run remaining on the same row with a visible gap. Initial capture exposed Run wrapping; remaining row width now comes from `available_rect_before_wrap`.
- Loading/error/empty-result states and direct keyboard/mouse interaction remain pending; these are deterministic PostgreSQL UI fixtures, not live provider verification. PostgreSQL and SQLite execution paths were not changed or retested.
- Learning pass: composite controls must take height from the shared control token; a wrapping row needs remaining rectangle width, not the row's total available width. Recorded here; no global memory write.
- `cargo test --locked -p db-pro-ui table_data_toolbar_surface_view::tests --lib`: 6 passed / 0 failed / 0 ignored, exit 0. Workspace check/clippy/full tests skipped for this layout-only correction.

## Empty WHERE executes unfiltered SELECT — 2026-10-06
- Source identity: HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted toolbar/view patch.
- `cargo test --locked -p db-pro-ui table_data_toolbar_surface_view::tests --lib`: 7 passed / 0 failed / 0 ignored, exit 0, including empty/whitespace predicates and nonempty predicate SQL.
- `cargo fmt --all -- --check`, `git diff --check`, `cargo build --release --locked -p db-pro-native --features capture`: PASS, exit 0. Workspace tests/check/clippy skipped for this narrow correction.
- Captured and inspected `evidence/where-empty-run-enabled-1280x800.png`: native empty input shows enabled Run. Static fixture capture is not live database execution. Larger viewports and loading/error/empty-result states not recaptured in this follow-up.
- PostgreSQL and SQLite: quoted SELECT generation shared; live empty Run/Enter execution independently pending. Existing safety/pending-change/running-query checks and configured result row limit remain in the inline query dispatch path.
- Learning pass: an empty optional WHERE predicate means an unfiltered SELECT; omit the WHERE clause rather than disabling execution. Recorded here; no global memory write.
