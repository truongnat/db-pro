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
  fallback for environments that do not honor the maximize request, and reapplies maximize after
  eframe restores persisted window geometry.
- Fresh launch after the reapply fix reported a full-monitor 2473×1409 window in the macOS harness;
  Orca capture: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/1e95a1ff-8f08-4952-8613-7ee3c17684ba-screenshot.png`.
- Orca's macOS accessibility snapshot exposes only the native window controls for this egui
  surface, so widget-level verification is screenshot-based rather than accessibility-tree-based.

## Provider matrix

| Provider | UI behavior affected by Wave 1 | Automated evidence | Runtime evidence |
|---|---|---|---|
| PostgreSQL | None; presentation-only shell | Existing native/runtime suites | Existing walkthrough remains valid; rerun if connection interaction changes |
| SQLite | None; presentation-only shell | Existing native/runtime suites | Existing walkthrough remains valid; rerun if connection interaction changes |

## Wave 2 evidence

| Check | Result | Evidence |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | 2026-09-11, after Wave 2 edits |
| `cargo check --workspace --offline` | PASS | 2026-09-11 |
| `cargo clippy --workspace --offline --all-targets -- -D warnings` | PASS | 2026-09-11 |
| `cargo test --workspace --offline` | PASS | 2026-09-11; 186 core, 39 infrastructure, 25 SQLite integration, 54 UI, 21 Tauri, 4 runtime, 1 native; 10 PostgreSQL live tests ignored without fixture env |
| `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` | PASS | 2026-09-11; 33 checks passed, 0 warnings/fails; macOS xargs compatibility warnings are emitted by the existing script |
| Native ER empty state | PASS | Orca capture `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/0c439307-0f22-4749-8151-e9d9ddade3c9-screenshot.png`; grid is visible behind the empty state |
| Native Agent context | PASS | Orca capture `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/81be016b-5cb5-47f7-a543-b434bd652deb-screenshot.png`; Preview, provider, connection, driver, table count and schema badges remain legible |
| Native Transfers placeholder | PASS | Orca capture `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/6a932db9-1e89-4a2e-b712-daeb8830603e-screenshot.png`; status badge stays compact |
| Native Monitor placeholder | PASS | Orca capture `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/fbcd36fb-b737-46d9-8ce2-0c232d35bb36-screenshot.png`; status badge stays compact |

## Wave 3 evidence

| Check | Result | Evidence |
|---|---|---|
| `cargo build --locked -p db-pro-native --offline` | PASS | 2026-09-11, rebuilt and relaunched native binary |
| `cargo fmt --all -- --check` | PASS | 2026-09-11, final Wave 3 check |
| `cargo check --workspace --offline` | PASS | 2026-09-11, final Wave 3 check |
| `cargo clippy --workspace --offline --all-targets -- -D warnings` | PASS | 2026-09-11, final Wave 3 check |
| `cargo test --workspace --offline` | PASS | 2026-09-11; 186 core, 39 infrastructure, 25 SQLite integration, 55 UI, 21 Tauri, 4 runtime, 1 native; 10 PostgreSQL live tests ignored without fixture env |
| `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` | PASS | 2026-09-11; 33 checks passed, 0 warnings/fails; existing macOS xargs compatibility warnings emitted |
| SQLite connection test/save/connect | PASS | Real temporary SQLite fixture; Orca screenshot `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/17360595-801c-40cb-bd92-6f78a0d44c59-screenshot.png` shows connected status and loaded schema |
| SQLite Data Editor | PASS | Three rows, five typed columns and null/JSON/decimal values visible in `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/c3fb448e-d9a1-4f7f-b3ff-bd0dae3dd867-screenshot.png` |
| SQLite Query Results | PASS | Three rows rendered through the shared result grid in `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/f2e70db6-7005-478d-a877-8213785ba81c-screenshot.png` |
| Centered native Insert row dialog | PASS | `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/bdb43a73-40fa-4d56-8c1c-e60ebc2ede5d-screenshot.png` |
| Full-window default | PASS | Fresh native window reported `2473×1409` at `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/4ff65e9a-5dcb-4267-8790-68c1a695b33f-screenshot.png` |

The provider matrix remains asymmetric by evidence: SQLite has the Wave 3 runtime walkthrough;
PostgreSQL remains presentation-only for this wave because no live fixture was authorized/configured.
The overall plan remains `IMPLEMENTING`: query/editor polish, keyboard/DPI/clipboard/file-picker
smoke coverage, wider provider walkthroughs and independent review are still open.
