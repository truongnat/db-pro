# Table Profile Quality — Verification

## Baseline

- Source baseline SHA: `710ba002612c6d71ef2605f99f2a49743b51c3a4`.
- Implemented Profile source blob: `1208c1372e23abfa151cafffc6f03965bd7ee024` (uncommitted working tree).
- The Profile surface source was clean at baseline; the surrounding table workspace has unrelated working-tree changes and is not modified by this task.
- User-provided visual evidence: screenshot of Table Detail → Profile, 2026-10-04; single wide viewport.

## Current work

- Implementation: visible padded profile card, grouped columns, honest completeness labels, conditional numeric summary, and a final Grid column that fills remaining width are present in the working tree. Range labels truncate to actual cell width and retain full values on hover. Plan remains `IMPLEMENTING` pending broader viewport/state review and regression verification.
- `cargo check --locked -p db-pro-ui`: PASS after correcting an initial compile error in the new pattern badge token reference.
- `cargo build --release --locked -p db-pro-native --features capture`: PASS (exit 0).
- Tests: not run, per task scope.
- Native runtime: deterministic sample fixture capture at 1280×800 completed with exit 0 and reviewed at `evidence/profile-light-1280x800.png` (2560×1600 physical pixels, 2× scale).
- Native runtime: 1440×900 capture attempt was stopped after the capture process stalled (exit 130); 1920×1080 and empty/loading/error states not captured.
- PostgreSQL/SQLite live UI: not run; this change consumes the existing result and issues no database operation.

## Limitations

- The supplied screenshot verifies the wide loaded-data state only. The new deterministic fixture verifies the loaded state at 1280×800; it does not verify 1440×900, 1920×1080, empty/loading/error states, or hover disclosure.
- `avg` and `sum` retain their existing `f64` calculation and precision limitations; changing those outputs is outside this focused pass. Mixed numeric/text columns omit Min/Max rather than applying an ambiguous comparison.

## Profile/Structure layout and dynamic numeric widths — 2026-10-06
- Baseline SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4`; uncommitted source blob identities: profile `392f66a7c74e04f0cdaa7e9200bca881e7ec4659`; shared table UI `86c181a99bf15fa86a24791bc44384afad8c877b`.
- `cargo test --release --locked -p db-pro-ui --lib`: FAILED overall, 992 passed / 2 failed / 0 ignored (exit 101). Long numeric rendering, complete badge border and passive index-name regressions passed. Failures: `explicit_disable_selection_is_preserved_on_submit`, `new_postgresql_connection_defaults_to_tls_require`, both report missing create command in app_tests. Baseline attribution is unverified; these connection-form paths are outside this patch and were not changed to make this layout task green.
- Initial debug test builds were interrupted after remaining stalled at rustc; a debug-free attempt was also interrupted. Release tests above provide the actual test results. Initial capture builds failed from an incorrect feature gate / TableView variant; both were corrected before final release build.
- `cargo fmt --all -- --check`, explicit rustfmt check for the included Profile source, and `git diff --check`: PASS. Workspace check/clippy/full workspace tests/benchmarks were not run; no performance improvement claimed.
- Earlier inspected fixture captures: `profile-width-*` and `structure-key-*` at requested 1280×800, 1440×900, 1920×1080. They prove the shared-table width and badge-border change; later numeric/header follow-up screenshots are recorded separately. Larger targets cap at 838 logical pixels high.
- Provider impact: PostgreSQL/SQLite data and commands unchanged; screenshots are deterministic native fixtures, not live-provider evidence. Loading/error/empty-result, horizontal scroll interaction and VoiceOver remain pending.
- Reusable lesson: measure full formatted numeric content with its actual font, include shared cell padding, and preserve that minimum when scrolling is required. Do not truncate number strings or guess percentage widths. Table clips must allow stroked badges to paint into padding without leaking into adjacent cells. Recorded in this plan; no global memory write.

## Final code gates — 2026-10-06
- Final native command: `cargo build --release --locked -p db-pro-native --features capture` → PASS, exit 0.
- Final Profile test command: `cargo test --release --locked -p db-pro-ui --lib table_profile_surface_view` → 3 passed / 0 failed / 0 ignored, exit 0.
- Final Profile blob `0c38004312fbcffa6bfa66b3bd5e9408e8b2e555`; shared header blob `b30acdf40773676f337676503e4bdca111917061`, against HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted patch. Horizontal Profile overflow keeps measured widths and locates its scrollbar immediately after the rows. Header input border uses semantic theme colors even when idle.

## Final native fixture evidence — 2026-10-06
- `evidence/profile-dynamic-numeric-1280x800.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/profile-dynamic-numeric-1440x900.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/profile-dynamic-numeric-1920x1080.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/structure-header-final-1440x900.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/structure-header-final-1280x800.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/structure-header-final-1920x1080.png`: final-code native framebuffer, PNG integrity verified.
- Requested widths 1280/1440/1920 produced physical images 2560×1600, 2880×1676 and 3840×1676; larger heights capped to 838 logical pixels. Selected 1280 and 1920 captures inspected: input border/font/alignment, full long numeric values in wide layout and horizontal overflow in narrow layout. Static fixtures only; scrolling/key/pointer/provider interactions not exercised. The Foreign Keys fixture exposes an inherited narrow Action-cell wrap/clip limitation; this header task does not modify it.
