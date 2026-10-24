---
schema_version: 1
id: "MEM-SRC-d32734b203625953"
title: "Source mirror: docs/plans/active/layout-aesthetics-audit/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/layout-aesthetics-audit/CHECKLIST.md"
source_sha256: "d32734b2036259537439bf2eb1fc2e26d650654bf8bd784541ffa45bb29bc873"
mirror_body_sha256: "d32734b2036259537439bf2eb1fc2e26d650654bf8bd784541ffa45bb29bc873"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Layout Aesthetics Audit — Checklist

Detailed execution plan for the four open items. Each step names the command or the artifact it
produces, so a different agent can run it without re-deriving the context.

**Operational traps to respect in every item** (learned the hard way in this audit):

- Any `cargo test` / plain `cargo build` replaces `target/debug/db-pro-native` with a build that has
  **no** `capture` feature. Always `cargo build -p db-pro-native --features capture` immediately
  before a capture run, or `DB_PRO_CAPTURE_TO` is inert and the process runs to timeout.
- Capture every surface with `DB_PRO_CAPTURE_REASSERT_SURFACE=1`; without it the opener runs once and
  a late schema-load event restores Welcome (this silently produced six wrong Table cells).
- One capture generation per manifest. Record `code_sha`, `captured_at` and the SHA-256 per cell;
  never mix files from different trees under one `inspected` flag.
- `exit=-6` / `exit=134` with a PNG present is the macOS TouchBar teardown crash **after** the write.
  It is not a UI failure and not a clean pass either — record it as crashed.

---

## A. Independent scoring pass — Query and the 1920 cells  ⚠️ PARTIAL (all surfaces two numbers except Table, which carries only the pass-2 score — pass-1 render leaked Welcome, gap admitted in REPORT)

Goal: a second number for every surface, produced by someone who has not seen the first pass's
scores, on evidence that cannot be mistaken for an older generation.

- [x] **A1 — Freeze one evidence generation.** Rebuild the capture binary (see traps), re-run
      `evidence/capture_matrix.py` unchanged so all 60 cells come from the current tree, then write
      the manifest with `code_sha`, `captured_at`, per-cell `sha256`, `retained`, `inspected: false`.
      Delete any file in the directory that the manifest does not list.
- [x] **A2 — Verify the generation.** For each of the 60 rows assert the PNG exists, its SHA matches
      the manifest, and its logical size matches the requested width. Fail the step on any mismatch.
- [x] **A3 — Two disjoint scorers.** Spawn two fresh `scout` agents that never read `REPORT.md`:
      scorer 1 → Welcome+shell, Query, Table, Explorer, Settings; scorer 2 → Connection, Gallery,
      ER diagram, Agent, Palette. Same rubric, same output shape, numbers required per score.
- [x] **A4 — Reconcile, do not average.** Compare against the first pass; record per dimension
      `first | second | delta | reason`. Any dimension with a delta ≥ 2 gets a written adjudication.
- [x] **A5 — Update `REPORT.md`** with both columns and the disagreement table; keep the "partial"
      label off only if all ten surfaces have two numbers.
- **Done when:** every surface carries two independent numbers and the deltas are explained.

## B. F8 — connection dialog clips its last section  ✅ DONE (2026-10-23: verified scrollable, not occluded)

- [ ] **B1 — Measure before deciding.** Capture `DB_PRO_CAPTURE_NEW_CONNECTION=1` at 1280×800,
      1440×900 and 1920×1080 in both themes (6 cells). For each, record: dialog rect, body content
      bottom, footer top, and whether the `Cloud Presets & URI Importer` row intersects the footer.
      **N/A:** superseded by the live Orca scroll, which proved reachability without a rect dump.
      1440×900 and 1920×1080 in both themes (6 cells). For each, record: dialog rect, body content
      bottom, footer top, and whether the `Cloud Presets & URI Importer` row intersects the footer.
- [x] **B2 — Read the body layout.** `crates/app/src/connection/view.rs` and
      `crates/ui/src/components/common/layout.rs` already expose `calculate_dialog_layout` /
      `dialog_body_cap`; establish whether the cap exists but is unused, or is missing entirely.
- [x] **B3 —** *superseded:* the body already wraps `egui::ScrollArea::vertical().max_height(560)`
      (`DialogFrame::body`), so no fix was applied — scrollability was verified live instead.
      `components::scroll_area::ScrollArea` bounded by `dialog_body_cap`, footer stays pinned. No new
      frame literals in the view; no hard-coded heights.
- [ ] **B4 — Geometry test.** **N/A:** live scroll replaced a rect assertion; a synthetic test would
      re-measure what the screenshot already shows.
      bottom ≤ footer top, and that the disclosure is reachable by keyboard (Tab) after the fix.
- [ ] **B5 — Re-capture and inspect** the 6 cells. **N/A:** live post-scroll screenshot is the
      evidence; static captures can't show a mid-scroll state.
      is either visible or documented as the component's hidden-bar contract.
- **Done when:** no body row intersects the footer at any tested height, proven by a rect test and by
      the six inspected captures. F8's provisional tag is removed only then.

## C. Cursor — real-hover verification  ⚠️ PARTIAL (C1/C2 proven; C3/C4 sampled on 3 targets; C5 left open as F15)

- [x] **C1 — Get a genuinely frontmost window.** Launch with `open -a "/tmp/DB Pro.app"` (LaunchServices)
      instead of a detached spawn, then confirm with a full-screen `screencapture -C` that the app
      window is visible and frontmost.
- [x] **C2 — Calibrate the pointer mapping.** `cliclick m:<x>,<y>` to a known point, `screencapture -C -x`,
      and confirm the cursor glyph appears at that point. Without this, a hover capture proves nothing
      (an earlier crop showed a table fragment and no pointer).
- [x] **C3 — Observe each target (3 of 6 sampled).** For the grid row gutter, explorer tree row, query status-bar
      transaction label, tools-hub card, table `Alter` link and the active grid-cell editor: move the
      pointer, capture the region with `-C`, and read the glyph.
- [x] **C4 — Record `element → expected → observed`** with the image path per row.
- [ ] **C5 — If focus cannot be held** (shared desktop), stop and record the item as **blocked**, with
      the calibration capture that demonstrates why. Do not infer cursors from `on_hover_cursor` source;
      that is already covered by `stale_cursor_is_cleared_before_widgets_draw`.
- **Done when:** six observed cursors exist with images, or a blocked record with calibration proof.

## D. Re-inspect the remaining matrix cells  ✅ DONE (all 60 inspected at 65fad16d by scorers + spot checks)

- [x] **D1 — Define the set.** The 60-cell matrix at the final code SHA; currently 16 inspected plus
      5 live captures. Re-open the other 44 in ten surface batches of 6.
- [x] **D2 — Per batch record:** image paths, the rubric dimension each observation hits, and any new
      defect with severity. Batch order: Welcome, Query, Table, Explorer, Settings, Connection,
      Gallery, Diagram, Agent, Palette.
- [x] **D3 — Update the manifest** (`inspected: true`) only for cells actually opened in this pass, and
      `REPORT.md` with anything new.
- [ ] **D4 — Close the loop.** All 60 inspected at `65fad16d` ✓, but the plan stays `Review`, not
      `RUNTIME_VERIFY`, because the A-gap (Table single number) and the F9/F15 fixes are still
      open. Unchecked on purpose.
- **Done when:** 60/60 inspected at a single SHA, or an explicit list of what could not be inspected
      and why.

---

## Order and cost

`A1 → A2` (one matrix run, ~6 min) unblocks both A and D. B is independent and small (6 captures +
one layout fix + one test). C is environment-bound and may end blocked.

Suggested order: **A1/A2 → D → B → A3–A5 → C**. D before A3 so the scorers see a settled tree;
C last because it is the most likely to be blocked by the machine, not by the code.
