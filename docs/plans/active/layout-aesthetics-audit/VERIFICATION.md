# Layout Aesthetics Audit — Verification

**Status: converged.** Two scoring passes on one frozen matrix, five live Orca surfaces verified,
and the open items each carry an honest label (partial/blocked) rather than a false "done".

## Source reference

```text
code under test     65fad16d   matrix SHAs match this tree
evidence manifest   evidence/manifest.json — 60 cells, sha256 per cell, all inspected=true
live captures       evidence/live/ — Orca screenshots (not capture-harness)
audit HEAD          fede8872 + subsequent doc-only commits (77ddfa61)
```

## What exists

| Item | State |
|---|---|
| Rubric with 0–4 anchors, eight dimensions | `RUBRIC.md` |
| Matrix | 60 cells captured at `65fad16d`, `DB_PRO_CAPTURE_REASSERT_SURFACE=1`, sha256-verified |
| Cells inspected | **60 of 60** (5 scout scorers × 12 each + spot checks; `inspected=true` in manifest) |
| Live Orca surfaces | Table data, Table structure, Settings→Appearance, Connection error, dialog scroll, quick open — 6 captures in `evidence/live/` |
| Pass-1 scores | 9 surfaces scored pre-ladder (mean 17.4/32); Table had to score from source only (capture leaked Welcome) |
| Pass-2 (independent) scores | 5 reviewers, no access to REPORT.md; **all surfaces two numbers except Table** (single pass-2 number — gap admitted, not filled) |
| Cursor verification | `screencapture -C` pipeline proven (Terminal I-beam); `PointingHand` observed as arrow on live hover → recorded as **F15 defect**, not a test gap |
| Real-state captures (empty/loading/error) | error verified live; empty/loading not in the matrix |
| Heights >838 logical | not exercised (host clamp — needs an external display) |

## The matrix integrity story

- First run silently captured Welcome for all 6 table cells (missing `REASSERT`); fixed at
  `c245183e` and enforced in `capture_matrix.py`.
- One generation rule: any cell not in `manifest.json` at the current `code_sha` is stale, not
  evidence.
- `exit=-6` / `exit=134` in the runner log is the macOS TouchBar teardown crash **after** the PNG
  write — a crash, not a UI failure and not a clean pass.
- `inspected=true` means the PNG was opened by a reviewer, not just written.

## Open verification gaps

| Gap | Why | Plan |
|---|---|---|
| Table pass-1 score | first capture leaked Welcome; a retro score would fabricate | accept as an admitted single-number surface, or re-score the baseline SHA in a follow-up |
| Cursor hover | OS glyph never changes despite `on_hover_cursor` | F15 — needs a winit/egui fix before re-verifying |
| >838 logical height | host clamps the window | external display or a taller fixture |

## Verification commands executed

- `cargo test -p db-pro-ui -p db-pro-app` — 725 + 681 pass, 0 fail.
- `cargo +nightly fmt --all -- --check`, `cargo clippy --workspace --all-targets -D warnings`,
  `cargo build --release --locked -p db-pro-native` — all exit 0.
- `python3 evidence/capture_matrix.py` — 60/60 PNGs written, manifest sha256 verified.
- `orca computer scroll --element-index <scrollbar>` — F8: dialog body scrolls, clipped row
  revealed. `screencapture -C` calibration — Terminal I-beam proven, app arrow observed.

## 2026-10-23 — Dialog header/footer separators (64a96f57) — evidence caveat

**Change**: `DialogFrame` + `paint_dialog_card` paint `Separator` hairlines so
dialog cards read as header / body / footer (light & dark verified in
`evidence/dlg-sep-*`).

**Captured sizes** (18 PNGs): requested 1280×800 / 1440×900 / 1920×1080.
**Actual framebuffer**: 1280×800, 1440×838, 1920×838 logical points
(@2× → 2560×1600, 2880×1676, 3840×1676 px).

**Why**: macOS clamps `InnerSize` to the physical display's visible area — the
workstation screen is ~900 logical points tall minus menu bar/Dock, so heights
above ~838pt cannot be realized on a windowed capture. Widths land exactly.
This is an environment constraint, not a bug in the separators: the hairlines
are drawn relative to the card's own height and verified at every captured
dimension. Full-height evidence requires a display (or virtual display) with
≥1080pt visible height.

**States verified per size**: normal, error (`DB_PRO_CAPTURE_CONNECTION_ERROR` —
note it must be set *alone*; combined with `NEW_CONNECTION` the else-if chain
picks the normal dialog first), and pending (`TEST_PENDING`). Empty-state does
not apply to a form dialog.
