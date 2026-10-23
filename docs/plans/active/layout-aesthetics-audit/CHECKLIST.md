# Layout Aesthetics Audit — Checklist

## 1. Preparation
- [ ] Confirm the surface inventory with the owner (10 surfaces; add/remove before capturing).
- [ ] Write the eight-dimension rubric (32-point total) with 0–4 anchors into `RUBRIC.md`.
- [ ] Fix the measurement list per screen (panel widths, gutters, type sizes, contrast, distinct
      radii/spacing steps, competing emphasis count).
- [ ] Build the capture binary: `cargo build -p db-pro-native --features capture`.

## 2. Evidence capture
- [ ] Capture each surface at 1280×800, 1440×900, 1920×1080 in light and dark.
- [ ] Capture one real state per surface where it exists (empty / loading / error).
- [ ] Record requested vs rendered size and process exit for every cell in a manifest.
- [ ] Inspect every retained image; list any missing cell as missing.

## 3. Measurement
- [ ] Extract panel width ratios and gutter values per screen from the images or the layout source.
- [ ] Record the type scale actually in use (sizes + weights) per screen.
- [ ] Compute contrast ratios for text/surface pairs that appear in the captures.
- [ ] Count distinct corner radii, spacing steps and emphasis levels per screen.

## 4. Judgement
- [ ] Score all eight dimensions per surface with anchors; attach the numbers to each score.
- [ ] Re-judge the three surfaces fixed in `457cd645` and state before/after.
- [ ] Write findings with severity, dimension, evidence pointer and owning plan.
- [ ] Route each finding: `native-visual-redesign`, `ui-spacing-consistency`,
      `core-component-style-system`, or a new issue when no plan owns it.

## 5. Review
- [ ] Independent reviewer repeats the scoring pass on the same evidence.
- [ ] Record agreements and disagreements; do not average them away.
- [ ] Update `STATUS.md` and move the plan forward only when the report is complete.
