# Verification

Baseline: `78d914887642dd400c7e2048db9a24d0403619c8` + working-tree fix.

## Automated gates (executed)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --offline` | PASS |
| `cargo clippy -p db-pro-ui --all-targets --offline -- -D warnings` | PASS |
| `cargo test -p db-pro-ui --offline` | 949 passed / 0 failed / 0 ignored |
| `cargo build -p db-pro-native --features capture` (debug) | PASS |
| `cargo build -p db-pro-native` (release, via perf scan) | PASS, 32.1MB |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | 14 pass / 2 warn / 0 fail — warnings: `StatusBar::show` 51 lines (>50 heuristic) and `paint_status_overflow` 4 params, matching sibling `paint_status_item` signature style |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | PASS (partial) — 4 executed checks pass; benchmark/db/er sections not executed (need window server / live DB) |

Not run: `cargo test --workspace` in full (focused `db-pro-ui` run only); no
runtime/provider behavior changed, all touched code is presentation layer.

## Runtime evidence (native captures)

`DB_PRO_CAPTURE_COMPONENT_GALLERY=1`, `DB_PRO_CAPTURE_GALLERY_SECTION`,
`DB_PRO_WINDOW_SIZE`, `DB_PRO_CAPTURE_GALLERY_SCROLL` (new harness env), files in
`evidence/`:

| File | Surface | Result |
|---|---|---|
| `database-shell-dark-1280x800.png` | card columns + status bar | card inside col, no bleed into "Connection Health"; status bar shows `…` for clipped left items, no overlap |
| `database-shell-light-1280x800.png` | same, light | same; readable on light surface |
| `database-shell-dark-1440x838-clamped.png` | wider viewport | all left items incl. `UTF-8 · READ COMMITTED` fit; clear gap to `Ln 42, Col 18` |
| `database-shell-dark-1920x838-clamped.png` | widest | same (display clamps height to 838) |
| `devtools-explain-dark-1280x800.png` | explain card @scroll 900 | `Hash Join`/`Seq Scan` stats right-aligned or clipped at scroll viewport — no overlap; no ID-collision callouts |
| `devtools-explain-light-1280x800.png` | same, light | same |
| `rendering-text-light-1280x800.png` | text rasterization grid | all `Query 0x7F3A` alpha rows readable on light bg |
| `rendering-dark-1280x800.png` | rendering section, dark | unchanged behavior on dark |

## Harness additions (capture-only)

- `DB_PRO_CAPTURE_GALLERY_SECTION=rendering` arm in `workspace_actions.rs`.
- `DB_PRO_CAPTURE_GALLERY_SCROLL=<f32>` → `vertical_scroll_offset` on the gallery
  detail `ScrollArea` (`component_gallery_view.rs`). Both inert in normal runs.

## Residuals

- P2 follow-up: audit other `ui.id().with(…)` / unsalted `ScrollArea` sites inside
  `ui.columns` children repo-wide (egui quirk documented in FINDINGS §F5).
- Stat text on very narrow plan rows is clipped by the scroll viewport rather
  than overlapping — expected; horizontal scrollbar exposes it.
