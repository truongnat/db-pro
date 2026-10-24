---
schema_version: 1
id: "MEM-SRC-19d146ca38378026"
title: "Source mirror: docs/plans/active/layout-aesthetics-audit/RUBRIC.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/layout-aesthetics-audit/RUBRIC.md"
source_sha256: "19d146ca383780264aed67e3a43e24235d332190f04eeecdd5145c2d2b8fcb42"
mirror_body_sha256: "19d146ca383780264aed67e3a43e24235d332190f04eeecdd5145c2d2b8fcb42"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Layout Aesthetics Audit — Rubric

Eight dimensions, 0–4 each, 32 points per surface. Anchors are written so two reviewers can disagree
about a *fact* ("is this 2 or 3?") rather than about the axis itself.

A score is only valid when the cell also carries the measured numbers listed under **measure** and a
cited image. No numbers, no score.

---

## 1. Composition & hierarchy

Does the screen say what to look at first?

| Score | Anchor |
|---|---|
| 0 | No discernible focal point; every element carries equal weight |
| 1 | A focal point exists but is contradicted by 3+ competing elements of similar weight |
| 2 | Focal point readable; secondary actions compete occasionally |
| 3 | Clear primary action, secondary actions recede, one emphasis level per region |
| 4 | Hierarchy survives scanning at a glance: primary → content → secondary, with no accidental emphasis |

**Measure:** count of elements at each emphasis level (accent fill / strong text / muted); panel width ratios.

## 2. Density & rhythm

Is the information density deliberate, and does repeated content sit on a rhythm?

| Score | Anchor |
|---|---|
| 0 | Rows and groups at arbitrary spacing; either cramped or mostly empty |
| 1 | Repeated rows drift by >2 pt or use 3+ different gaps |
| 2 | Mostly consistent rhythm with visible exceptions |
| 3 | One rhythm per region; density matches the task (dense grid, calm forms) |
| 4 | Rhythm holds across regions; whitespace groups content instead of separating everything |

**Measure:** distinct vertical gaps in use; row heights; dead-space share per panel.

## 3. Alignment & grid

Do edges line up?

| Score | Anchor |
|---|---|
| 0 | Content edges disagree within the same column |
| 1 | Frequent 1–4 pt misalignments or icons off the text baseline |
| 2 | Main edges align; secondary clusters drift |
| 3 | Shared left/right edges across panels; icons and labels optically centred |
| 4 | A single grid explains every edge, including popovers and toolbars |

**Measure:** left/right edge coordinates of stacked elements per panel; icon-to-text baseline delta.

## 4. Typography

Is the type scale small, purposeful and readable?

| Score | Anchor |
|---|---|
| 0 | Sizes chosen ad hoc; body text below 12 pt or above 16 pt without reason |
| 1 | 5+ sizes in one screen, or weight used to compensate for a missing hierarchy |
| 2 | Recognisable scale with one or two off-scale sizes |
| 3 | 3–4 sizes with clear roles; line length controlled; truncation deliberate |
| 4 | Scale reads as a system; tabular/numeric text aligned; no orphaned one-off sizes |

**Measure:** size/weight pairs in use; longest line length; truncation points.

## 5. Colour & surface hierarchy

Do surfaces stack in a legible order, and does colour carry meaning?

| Score | Anchor |
|---|---|
| 0 | Adjacent surfaces indistinguishable, or accent used decoratively everywhere |
| 1 | Elevation steps exist but are invisible in one theme, or states collapse (hover = base) |
| 2 | Surfaces distinguishable; one state pair ambiguous |
| 3 | Canvas → chrome → raised → floating readable in both themes; states distinct; text ≥ 4.5:1 |
| 4 | The stack explains itself without borders; accent reserved for action/selection |

**Measure:** contrast ratios for each text/surface pair; count of accent-coloured elements per screen; number of distinct surface steps.

## 6. Component consistency

Does the same action look the same everywhere?

| Score | Anchor |
|---|---|
| 0 | The same action rendered 3+ different ways |
| 1 | Variants differ in size, radius or weight without a role difference |
| 2 | Consistent within a surface, differs across surfaces |
| 3 | One control vocabulary across surfaces; variants map to roles |
| 4 | Vocabulary holds in edge states (disabled, loading, error) as well |

**Measure:** inventory of control variants per role; count of radii in use.

## 7. Iconography

Do icons belong to one family and sit right?

| Score | Anchor |
|---|---|
| 0 | Mixed filled/outline/emoji styles in one region |
| 1 | Consistent family but inconsistent size or baseline |
| 2 | Consistent size with occasional optical misalignment |
| 3 | One family, one optical size, aligned to the text baseline |
| 4 | Icons carry meaning consistently; decorative icons absent |

**Measure:** icon sizes in use; baseline delta vs adjacent text.

## 8. Responsive composition

Does the layout still work when the window or a pane shrinks?

| Score | Anchor |
|---|---|
| 0 | Content clips or becomes unreachable at 1280×800 with panes open |
| 1 | Controls survive but labels truncate into ambiguity |
| 2 | Usable with one visible compromise (e.g. squeezed toolbar) |
| 3 | Panels reflow or collapse deliberately; no unreachable control |
| 4 | Every width tested keeps the same task order and priority |

**Measure:** per width: clipping count, truncated labels, unreachable controls.

---

## Scoring rules

- Score per **surface × theme**; the responsive dimension is scored once per surface using both widths.
- A dimension with no evidence is scored `n/a`, never guessed — and the total is then out of fewer points,
  reported as `x / y`.
- Disagreement between passes is recorded as two numbers, not averaged.
- Totals are guidance, not a verdict: a single 0 on dimension 5 or 8 blocks "acceptable" regardless of total.
