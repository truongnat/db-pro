# UAT Fixes Wave — Verification

Plan state: IMPLEMENTING. Entries added per fix.

## Evidence baseline
- UAT drives: seven sequential orca sessions on the live debug build
  (2026-10-23/24) — findings in FINDINGS.md.
- Capture caveat: macOS visible-area clamp → heights >838 logical pt cannot be
  captured on this workstation; record actual sizes when used as evidence.

## UAT-4 — row-limit disclosure

### Provider trace

- `crates/core/src/application/query_service.rs:157-178` calls
  `DbConnector::query` and validates/returns the connector result; it does not
  discover a total row count.
- `crates/infrastructure/src/postgres/connector.rs:97-114` fetches only while
  `result_rows.len() < max_rows`, then assigns `row_count = result_rows.len()`.
- SQLite follows the same cap-first loop in
  `crates/infrastructure/src/sqlite/actor.rs:768-783`; MySQL checks the cap
  before requesting the next streamed row in
  `crates/infrastructure/src/mysql/connector.rs:109-120`; SQL Server breaks
  collection when its current row count reaches the max in
  `crates/infrastructure/src/sqlserver/connector.rs:102-116`.
- Therefore a returned count equal to the cap does **not** prove more rows
  existed. UI wording discloses the injected query limit but does not claim
  truncation.

### UI and test evidence

- Toolbar-injected cap travels through destructive confirmation and execution
  completion; query result headers and inline-query pagination disclose it.
- Regression cases cover results exactly at the cap and below the cap; both
  report the applied cap, avoiding a false truncation assertion.
- Focused tests and full suite results are recorded below after completion.
- Visual review: existing compact, muted result-header label style retained;
  no color, spacing, border, or layout changes. `DbProTheme` continues to own
  both themes. `frontend-ui-engineering`, `theme-factory`, and `ui-design-review`
  guidance reviewed; theme-factory's deck/artifact palette workflow is not
  applicable because no theme is being selected or changed.
- Runtime matrix: normal successful-results surface captured and inspected in dark/light at requested 1280×800, 1440×900, and 1920×1080. macOS clamps larger heights to 838 logical points. Loading/error/empty-state screenshots were not captured; the mandatory visual matrix remains pending, so this UAT is not runtime-verified or completed.

## UAT-1 — explorer filter popover click-through

- Implemented in `76ead2690a874eef8f55b5cb710dad5ed56d4aa4`: a screen-covering
  click sink below Tooltip-layer popovers consumes outside dismissal presses.
- Regression test: `cargo test -p db-pro-ui dismiss_scrim_consumes_the_outside_click`
  → 1 passed / 0 failed / 0 ignored; validates popover opens, outside press
  closes it without activating the underlying row, and inside interaction works.
- Native open-popover captures inspected: `/tmp/uat1/explorer-filter-dark-1280x800.png`,
  `...-light-1280x800.png`, plus dark/light 1440×900 and 1920×1080 requests.
  Rendered outputs were 2560×1600, 2880×1676, and 3840×1676 pixels respectively
  (1280×800, 1440×838, 1920×838 logical); larger requested heights are host-clamped.
- State coverage: open popover captured; outside-dismiss behavior and inside
  interaction are exercised by the component-level UI regression test.
- Theme/design review: existing popup frame and semantic theme tokens retained;
  no visual redesign or raw color changes.

## UAT-5 — workspace active-tab visibility

- Implemented in `a7362d39`: the tab-strip view tracks the active target and
  requests scrolling only when that target changes; `scroll_to_cursor(None)`
  reveals a clipped selection without repeatedly pulling the strip while the
  user scrolls. The selected tab keeps its existing close affordance.
- Regression tests: `cargo test -p db-pro-app active_tab_` → 3 passed;
  `cargo test -p db-pro-app app_lifecycle::tests::scheduler_repaint_cadence_tests`
  → 3 passed. Full `cargo test -p db-pro-app -p db-pro-ui` → 1,449 passed,
  0 failed, 0 ignored.
- Quality gates passed after the final scroll behavior change:
  `cargo +nightly fmt --all -- --check`, `cargo check --workspace`,
  `cargo clippy -p db-pro-ui -p db-pro-app --all-targets -- -D warnings`,
  `cargo build --release --locked -p db-pro-native`, and
  `bash scripts/check-ui-architecture.sh`.
- Native capture fixture seeds 18 query tabs with the last active. Six captures
  inspected: `/tmp/uat5/workspace-tabs-{dark,light}-{1280x800,1440x900,1920x1080}.png`.
  The selected rightmost tab and its close × are visible. Rendered pixel sizes
  were 2560×1600, 2880×1676, and 3840×1676 (logical 1280×800, 1440×838, and
  1920×838); macOS clamps larger requested heights.
- The overflowing strip was exercised in the app's real workspace-tab capture
  fixture, and the horizontal ScrollArea regression test confirms its offset
  moves when the far-right active tab becomes selected.
- No theme tokens or visual styling changed; both existing themes are covered.
