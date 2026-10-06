# Table Indexes Responsive Layout — Verification

## Baseline

- Repository baseline SHA: `710ba002612c6d71ef2605f99f2a49743b51c3a4`.
- Implemented `table_indexes_surface_view.rs` blob: `46f85b3a119c511633dc9a31bc81b74882e05fa6` (uncommitted working tree).
- Native 1280×800 logical capture is 2560×1600 physical pixels at 2× scale.

## Automated evidence

- Before fix: `cargo test --locked -p db-pro-ui index_columns_fit_the_standard_table_detail_viewport -- --nocapture` → 0 passed / 1 failed / 0 ignored, exit 101; requested width was 1,100pt for 928pt available.
- After fix: same command → 1 passed / 0 failed / 0 ignored, exit 0.
- `cargo build --release --locked -p db-pro-native --features capture` → exit 0.
- `git diff --check -- crates/ui/src/table_indexes_surface_view.rs crates/native-app/src/capture.rs crates/ui/src/workspace_actions.rs` → exit 0.

## UI runtime evidence

- Deterministic PostgreSQL-labeled capture, before: `evidence/indexes-light-1280x800-before.png`.
- Deterministic PostgreSQL-labeled capture, after: `evidence/indexes-light-1280x800-after.png`; all six columns fit at the standard 1280×800 logical viewport, including full Status text.
- The fixture is local and does not connect to a live PostgreSQL database. SQLite UI runtime was not run.
- 1440×900 and 1920×1080 captures, empty/loading states, and the detail dialog remain unverified. A previous 1440×900 capture attempt stalled and was stopped; do not treat it as a pass.

## Follow-up UI fix — 2026-10-04

- Source blobs: `crates/ui/src/table_indexes_surface_view.rs` at `600e950c84b7128ceee37cba2a938de6d971df94`; shared Dialog implementation at `01db1a334ce323d529cf87db0995a4390a048c72`.
- The index icon/name spacing now uses `SPACE_SM`. The detail dialog skips its DDL row and separator when `definition` is empty, as with SQLite autoindexes whose `sqlite_master.sql` is NULL. Its title text is hidden; the close button remains.
- `cargo test --locked -p db-pro-ui clicking_index_name_opens_its_details` → passed.
- `cargo test --locked -p db-pro-ui index_columns_fit_the_standard_table_detail_viewport` → passed.
- `cargo build --release --locked -p db-pro-native` → passed. `cargo fmt --all -- --check` and `git diff --check` → passed.
- After hiding the title text, 24 dialog-filtered tests, the index click test, release build, formatting check, and diff check all passed.
- Fresh native light-theme capture: `evidence/indexes-light-1280x800-spacing-fix.png` (2560×1600 physical pixels at 2×). It confirms the icon/name gap and full count label. The dialog itself was not captured after the change.

## Review limits

- The active `schema-indexes-runtime` plan still owns live PostgreSQL/SQLite index lifecycle verification.
- No workspace-wide test suite, clippy, or independent Kilo review was run.

## Aligned headers and modal removal — 2026-10-06
- Source SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted changes; header blob `671340bb0fee783f5d44d1d6d9c4b3186ec8effe`, Indexes `2e2e28ec8e97d22bf0ac38cd029016a3e8e3e68d`, Structure `e6f37db2ab7cec51c2d09fc72f95b41241535c4c`, Foreign Keys `f0774bf36127da6c96b64058f7df5cfd459a2ec1`.
- Headers share 30px compact field geometry and 13px label font. Indexes/Structure no longer have modal renderer, activation enum/adapter, or selected-detail state. Passive index labels disable text-selection click semantics; full-name/definition and long-value hover disclosure remains.
- UI release tests: 992 passed / 2 failed / 0 ignored, overall exit 101. The passive-index regression passed after disabling selectable label semantics; the two remaining failures concern create-connection command dispatch, outside this patch, baseline attribution unverified. Full command and failures recorded in the Table Profile Quality verification.
- fmt/diff checks pass. Workspace gates/CI skipped. Database commands unchanged; deterministic capture routes are tooling only.
- Reusable lesson: filter headers should share font and control-height tokens, rather than combining legacy TextEdit content height plus margins with separately sized section labels. Modal-only state and actions should be removed with a redundant modal. Recorded here; no global memory write.

## Final code gates — 2026-10-06
- Final native command: `cargo build --release --locked -p db-pro-native --features capture` → PASS, exit 0.
- Final Profile test command: `cargo test --release --locked -p db-pro-ui --lib table_profile_surface_view` → 3 passed / 0 failed / 0 ignored, exit 0.
- Final Profile blob `0c38004312fbcffa6bfa66b3bd5e9408e8b2e555`; shared header blob `b30acdf40773676f337676503e4bdca111917061`, against HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted patch. Horizontal Profile overflow keeps measured widths and locates its scrollbar immediately after the rows. Header input border uses semantic theme colors even when idle.

## Final native fixture evidence — 2026-10-06
- `evidence/indexes-header-final-1280x800.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/indexes-header-final-1440x900.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/indexes-header-final-1920x1080.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/foreign-keys-header-final-1920x1080.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/foreign-keys-header-final-1280x800.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/foreign-keys-header-final-1440x900.png`: final-code native framebuffer, PNG integrity verified.
- Requested widths 1280/1440/1920 produced physical images 2560×1600, 2880×1676 and 3840×1676; larger heights capped to 838 logical pixels. Selected 1280 and 1920 captures inspected: input border/font/alignment, full long numeric values in wide layout and horizontal overflow in narrow layout. Static fixtures only; scrolling/key/pointer/provider interactions not exercised. The Foreign Keys fixture exposes an inherited narrow Action-cell wrap/clip limitation; this header task does not modify it.

## Constraints header / Type spacing — 2026-10-06
- Baseline HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted working-tree patch.
- `draw_metadata_filter_header_with_trailing` shares the same label/input typography, height and semantic borders as the other metadata headers. The existing total-label wrapper remains unchanged for its callers.
- Constraints category controls reuse parent `tab_button` (selected fill `theme.surface_hover`) and render in reversed iteration under right-to-left layout, preserving visible All → Primary Key → Unique → Check → Not Null order. Type icon/badge spacing uses SPACE_SM (8 px). Filtering predicates unchanged.
- Native capture build `cargo build --release --locked -p db-pro-native --features capture`: PASS, exit 0. Formatting and diff checks: PASS.
- Loaded fixture captures `evidence/constraints-header-{1280x800,1440x900,1920x1080}.png`: PNG integrity checked. 1280 screenshot inspected for input/label alignment, category position/style, Type spacing. Physical images 2560×1600, 2880×1676, 3840×1676; larger heights capped by host.
- Pending: category/search interaction and loading/error/empty-state captures; live PostgreSQL and SQLite not exercised. Existing full UI suite result 992 passed / 2 failed remains unresolved; those connection tests were not rerun for this display-only patch. Workspace check/clippy/tests skipped.
- Reusable lesson: use the shared parent tab component for child categories and expose a trailing header slot instead of duplicating form sizing/colors. Recorded here; no global memory writes authorized.

- Required shipped build `cargo build --release --locked -p db-pro-native`: PASS, exit 0 (7.96 s); log `/tmp/db-pro-constraints-release.log`.

## Dependencies follow-up — 2026-10-06
- Baseline SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted patch. Shared header and parent `tab_button` reused; input sizing/borders/font, right-aligned categories and SPACE_SM direction icon gap match Constraints. Search/direction predicates and OpenTable action unchanged. Removed unused duplicated search renderer.
- Compact `matching of total` count shares the right-aligned tab row (for example, `2 of 3`), with a hover explanation. It uses the same direction and text predicates as the table. Initial screenshot exposed full-height allocation for a row right alignment; wrapping `with_layout` in `ui.horizontal` keeps the count compact above the table.
- Reusable lesson: right-to-left `with_layout` in a vertical parent may consume available height; constrain row allocation with `horizontal` and verify the table remains visible in the screenshot. Recorded here, no global memory writes.
- Existing UI full-suite 992 passed / 2 failed connection tests remain unclassified; not rerun for this reversible display-only patch. Workspace check/clippy/tests and live SQLite/PostgreSQL interactions skipped. Loading/error/empty and category/search/navigation interaction matrix pending.
- Final capture build: PASS, exit 0 (28.47 s). Fresh loaded screenshots `evidence/dependencies-header-{1280x800,1440x900,1920x1080}.png` verified as valid PNGs, sizes 2560×1600, 2880×1676, 3840×1676. 1280 native screenshot inspected: table remains directly beneath compact header/count rows; input border/font, gray active category, right alignment and direction gap visible. Host caps larger logical height to 838.
- Formatting and diff checks passed; no new tests added for presentation-only changes.

- Required `cargo build --release --locked -p db-pro-native`: PASS, exit 0 (5.15 s).
- Final count-layout captures `evidence/dependencies-count-{1280x800,1440x900,1920x1080}.png` are valid PNGs at physical sizes 2560×1600, 2880×1676, 3840×1676. 1280 screenshot inspected: `2 of 3` sits at the right edge of the filter row, while the two outgoing rows remain directly under the header. Capture build and shipped release build both PASS; formatting and diff checks PASS.
