# Verification

Implementation source SHA: `04c1c9b3df3052ea24c23ddc361b203d4425222a`.

## Automated

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 1605 passed, 0 failed, 41 ignored; db-pro-ui: 948 passed |
| `cargo build --release --locked -p db-pro-native --features capture` | PASS |
| `cargo build --release --locked -p db-pro-native` | PASS |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | PASS — 14 pass, 2 warnings, 0 failures; warnings are function-argument count and existing function-length ratchet |
| `git diff --check` | PASS |

The two new geometry tests cover tab title/close separation and field size/accessory reservations over parent gaps 0, 8, and 16. Another verifies required-marker, field, and helper gaps; a grid test checks that cell content and its parent keep their own spacing while the grid owns row/column gaps. The retained temporary geometry probe measured Input, clearable/disabled-clearable Input, Password, and Select at width 300: all are 38 points high and 300 points wide across the sampled contexts. Required marker gap is 4 points.

## Native evidence

The native capture build recorded Query tabs, Gallery form, error form, feedback/progress/shortcuts, and navigation tabs at requested 1280×800, 1440×900, and 1920×1080 window sizes. PNGs and their manifest are in `evidence/native/`; `overview.jpg` provides a contact sheet and `query-tabs-crop.png` shows the new tab insets at 2× physical pixels.

The Mac display constrained the 1440×900 and 1920×1080 requests to physical 2880×1676 (1440×838 logical) and 3840×1676 (1920×838 logical). Thus those captures do not prove the full requested heights. At 1440 captures, the PNG was written successfully before the isolated app required SIGTERM to close. Capture logs also show pre-existing missing replacement-glyph warnings.

## Performance and lifecycle

No frame-prepare timing, glyph-cache hit-rate, raster count, or atlas-growth measurement was collected. The change adjusts egui layout dimensions and spacing; it does not change fonts or rendering paths. Performance impact is therefore unmeasured, not claimed as a benchmark pass.

P0/P1: 0. Seven P2 spacing findings in the preceding audit were addressed in the scoped components and Gallery layout. The formal full-height native viewport gate remains pending because this machine cannot provide the requested logical heights, so lifecycle remains `RUNTIME_VERIFY`.
