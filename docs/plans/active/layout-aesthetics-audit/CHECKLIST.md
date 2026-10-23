# Layout Aesthetics Audit — Checklist

## 1. Preparation
- [x] Confirm the surface inventory (10 surfaces, one representative each).
- [x] Write the eight-dimension rubric with 0–4 anchors (`RUBRIC.md`).
- [x] Fix the measurement list per screen (panel widths, gutters, type sizes, contrast, distinct
      radii/spacing steps, competing emphasis count).
- [x] Build the capture binary: `cargo build -p db-pro-native --features capture`.

## 2. Evidence capture
- [x] Capture the 60-cell matrix (10 surfaces × 1280×800/1440×900/1920×1080 × light/dark).
- [x] Record requested vs rendered size and process exit for every cell in `evidence/manifest.json`.
- [ ] Capture the declared sub-surfaces: Table **structure**, Settings → **Appearance**, New
      Connection **error** state (each × 3 widths × 2 themes).
- [ ] Capture one real state per surface where it exists (empty / loading / error).
- [ ] Inspect every retained image; record per-cell `inspected: true` in the manifest.
      Progress: 6 of 60 (Explorer, both themes × 3 widths).

## 3. Measurement
- [ ] Extract panel width ratios and gutter values per scored screen.
- [ ] Record the type scale actually in use (sizes + weights) per screen.
- [ ] Compute contrast ratios for the text/surface pairs that appear in the captures.
- [ ] Count distinct corner radii, spacing steps and emphasis levels per screen.

## 4. Judgement
- [ ] Score all eight dimensions per surface with anchors; attach the numbers to each score.
- [ ] Re-judge the surfaces fixed during the audit (viewport/picker/chrome, Explorer popover) and
      state before/after using the retained pre-fix capture where one exists.
- [x] Write the first findings with severity, dimension, evidence pointer and owning plan
      (`FINDINGS.md`: F7a fixed, A1–A6 open).
- [ ] Route every remaining finding to `native-visual-redesign`, `ui-spacing-consistency`,
      `core-component-style-system`, or a new issue.

## 5. Review
- [ ] Independent reviewer repeats the scoring pass on the same evidence.
- [ ] Record agreements and disagreements; do not average them away.
- [ ] Write `REPORT.md` with the score table and the routed finding list.
- [ ] Update `STATUS.md` only when the report exists; the audit stays `PLANNING` until then.
