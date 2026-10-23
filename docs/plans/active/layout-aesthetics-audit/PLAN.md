# Layout Aesthetics Audit

## State

- **State:** PLANNING
- **Branch:** `main` (direct edit per owner override)
- **Baseline SHA:** `493e3b9b` (after the viewport/select/chrome fixes)
- **Surface:** the shipped native `egui` application — every primary workspace, not a component gallery.

## Goal

Produce an evidence-backed assessment of how the native layout *looks and reads* — composition,
hierarchy, density, rhythm, type, colour and component consistency — scored against explicit
anchors, with prioritized findings that each name an owner plan.

This is an **evaluation**, not a redesign. No visual direction changes land here.

## Why now

The viewport overflow, complex-picker geometry and light-chrome separation work is finished
(`457cd645`), so the owner asked for a design/aesthetic read of the result. Those fixes were
correctness fixes; nobody has yet judged the composition as a whole.

## Evidence and failure scenario

- Existing plans cover *structure* (`native-visual-redesign`), *spacing tokens*
  (`ui-spacing-consistency`) and *component tokens* (`core-component-style-system`), but none owns a
  repeatable aesthetic evaluation. Without one, each review round re-litigates taste with no shared
  rubric, and regressions are only noticed when the owner spots them (as happened with the light
  chrome step).
- At `493e3b9b` there is no document stating, for example, how many distinct corner radii, type
  sizes or emphasis levels appear on one screen — the numbers that make "looks busy" arguable.

## Scope

**In scope (measurement and judgement only):**

| Dimension | What is measured |
|---|---|
| Composition & hierarchy | primary action per screen, competing emphasis, panel width ratios |
| Density & rhythm | vertical rhythm on repeated rows, gutter consistency, dead space |
| Alignment & grid | edges shared across panels, optical alignment of icons/labels |
| Typography | type scale in use, line lengths, weight usage, truncation behaviour |
| Colour & surface hierarchy | elevation steps, contrast ratios, accent usage per screen |
| Component consistency | same action rendered the same way across surfaces |
| Iconography | size/weight consistency, alignment to text baseline |
| Responsive composition | what survives at 1280×800 with panes open |

**Out of scope:** behaviour changes, provider/database work, new components or primitives, any
redesign proposal that is not backed by a finding in this audit, motion recording (static frames
only).

## Method

1. **Surface inventory (fixed list).** Welcome, Query workspace (editor + output dock), Table
   workspace (data grid, structure), Schema explorer, ER canvas, Settings (General + Appearance),
   Connection dialog (blank + error), Component Gallery, Agent panel, Command palette.
2. **Capture matrix.** Each surface at 1280×800, 1440×900 and 1920×1080, light and dark, plus one
   real state where it exists (empty/loading/error). Use the existing `--features capture` harness;
   record requested vs rendered size and the `exit=-6` TouchBar teardown honestly.
3. **Measure, don't vibe.** For every scored screen record objective numbers alongside the image:
   panel widths, gutters, type sizes, contrast ratios, count of distinct radii/spacing steps, count
   of competing emphasis elements. A score without a number or a cited image is not evidence.
4. **Score with anchors.** Eight dimensions from the table above, 0–4 each (32 points), with written
   anchors (0 = broken, 2 = usable but inconsistent, 4 = deliberate and consistent). Two passes: self,
   then an independent reviewer on the same evidence; disagreement is reported, not averaged away.
5. **Prioritize and route.** Every finding carries severity (P0/P1/P2), the dimension it hits, its
   evidence pointer, and the owning plan. Nothing is filed as "polish later" without an owner.

## Acceptance criteria

- [ ] Every surface in the inventory is captured at all three widths in both themes, and each image is
      inspected; missing cells are listed as missing, never inferred.
- [ ] Each scored screen carries measured numbers next to its score.
- [ ] The rubric, its anchors and the per-surface scores are published.
- [ ] Every finding has severity, evidence pointer and an owning plan; no orphan findings.
- [ ] The three surfaces fixed in `457cd645` are re-judged and their before/after is stated.
- [ ] An independent reviewer repeats the scoring pass; agreements and disagreements are recorded.
- [ ] `docs/plans/STATUS.md` reflects this plan's real state.

## Risks and decisions

- **Taste presented as fact.** Mitigated by anchors, measurements and a second pass; scores are
  labelled as expert judgement, not user testing.
- **Scope explosion into redesign.** Findings route to existing owner plans; this plan does not
  implement visual direction changes.
- **Host height clamp.** 1440/1920 requests render at 838 logical height on this machine; the report
  states that instead of claiming full-height verification.
- **Overlap with `ui-spacing-consistency` and `core-component-style-system`.** Those plans own the
  fixes; this audit owns the judgement and the evidence. Findings that duplicate their open items are
  linked, not re-planned.

## PostgreSQL / SQLite impact

Not applicable: presentation evaluation only; no provider or database behaviour is touched.
