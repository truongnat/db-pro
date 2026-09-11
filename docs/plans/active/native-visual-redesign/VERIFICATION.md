# Native Visual Redesign — Verification

## Current state

Wave 1 implementation and runtime screenshot pass are complete on `feature/native-visual-redesign`.
No overall feature completion claim has been made: independent review and any provider/runtime
regression follow-up remain open. The baseline screenshot and source evidence are recorded in
`FINDINGS.md`.

## Commands

| Command | Result | Evidence |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | 2026-09-11, after Wave 1 implementation |
| `cargo check --workspace --offline` | PASS | 2026-09-11 |
| `cargo clippy --workspace --offline --all-targets -- -D warnings` | PASS | 2026-09-11 |
| `cargo test --workspace --offline` | PASS | 2026-09-11; provider integration suites ran independently where configured, PostgreSQL live tests remain ignored without fixture env |
| `cargo build --locked -p db-pro-native --offline` | PASS | 2026-09-11; rebuilt after default-maximized launch change |
| Native launch screenshot at 1280×800 | PASS | Orca capture `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/bc48a216-856d-4464-80a3-d1290fa72785-screenshot.png`; window 1280×832 including titlebar, content 1280×800 |
| Native launch screenshot at 1440×900 | PASS | Orca capture `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/7676317c-9b76-4b65-a7a5-dd5898392cb5-screenshot.png`; window 1440×932 including titlebar, content 1440×900 |

## Runtime observations

- Fresh native process opened dark-first with the redesigned Explorer/welcome shell.
- Settings → Dark and activity-rail → Explorer were exercised through the real macOS window;
  the resulting screenshots show the selected state, grid canvas and context-driven connection actions.
- The native entrypoint now requests a maximized window by default with a 1280×800 inner-size
  fallback for environments that do not honor the maximize request.
- Orca's macOS accessibility snapshot exposes only the native window controls for this egui
  surface, so widget-level verification is screenshot-based rather than accessibility-tree-based.

## Provider matrix

| Provider | UI behavior affected by Wave 1 | Automated evidence | Runtime evidence |
|---|---|---|---|
| PostgreSQL | None; presentation-only shell | Existing native/runtime suites | Existing walkthrough remains valid; rerun if connection interaction changes |
| SQLite | None; presentation-only shell | Existing native/runtime suites | Existing walkthrough remains valid; rerun if connection interaction changes |
