# Agent evidence — Layout Aesthetics Audit

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | omp · audit lane — 9 per-surface `scout` agents (pass 1) + 5 `scout` scorers (pass 2) |
| Issue(s) | Owner request for an aesthetics/design evaluation + "audit kỹ hơn, mỗi feature một sub agent"; owner follow-ups on the Explorer popover and cursor behaviour |
| Task state | `Review` — all planned steps executed; two-pass scoring reconciled; 8 findings recorded (F9–F15), no P0 |
| Baseline SHA | `493e3b9b` · code under test `65fad16d` · docs trail `65fad16d..6f4b7255` |
| Branch / PR | `main` (direct edit per owner override) · no PR |
| Scope interpretation | Score every primary surface against the eight-dimension rubric using per-surface agents, fix owner-reported defects as routed findings, and report the recurring design problems with owners. |
| Out of scope | Redesign proposals without a finding; provider work; motion recording; scoring cells that do not exist. |

## 2. Progress checkpoint

- Evidence matrix at `65fad16d` (60 cells, `evidence/manifest.json` with per-cell sha256); audit docs trail it by a few doc-only commits.
- Completed:
  - [x] `RUBRIC.md` — 8 dimensions, 0–4 anchors.
  - [x] Pass 1: 9 agents scored the first matrix (pre-ladder theme).
  - [x] Fixes landed during pass 1: F7a popover inset (`1b94044e`), F7b nav selection pill (`bc99ad10`),
        light-theme elevation ladder (`c735211d`), input field boundary, chevron order (`faab6466`).
  - [x] Matrix re-captured at `65fad16d` with `DB_PRO_CAPTURE_REASSERT_SURFACE=1`; 60/60 PNGs,
        sha256-verified.
  - [x] Pass 2: 5 independent scorers × 12 cells each, no access to REPORT.md.
  - [x] Live Orca verification: Table data, Table structure, Settings→Appearance, Connection error,
        dialog scroll (F8), hover-cursor (F15).
- Independent pass — **partial**: all surfaces have two numbers **except Table** (pass-1 capture
  leaked Welcome, so Table carries only the pass-2 score of 21/22 light, 22/32 dark — the gap is
  admitted in REPORT.md, not filled retroactively).
- F8 — **resolved, not a defect**: `DialogFrame::body` wraps the form in `egui::ScrollArea` with
  `VisibleWhenNeeded`; live Orca scroll revealed the clipped section fully. Downgraded to P3
  affordance note.
- F15 — **recorded**: `on_hover_cursor(PointingHand)` observed as OS arrow on 3 of 6 sampled
  hover targets; `screencapture -C` proven via Terminal I-beam. Unit test green, runtime glyph
  wrong → open defect, root cause unproven (winit mapping vs egui propagation vs focus).
- F9 — **recorded**: shared `components::scroll_area::ScrollArea` hardcodes `AlwaysHidden` → grid
  right-edge clip invisible. **Separate cause** from the dialog's `VisibleWhenNeeded` (which fades
  but exists) — two distinct code paths, do not merge.
- Findings / risks: F9–F14 routed; F8 P3; F15 open defect; cursor list is a runtime-visual defect,
  not a test gap.
- Tests executed: `cargo test -p db-pro-ui -p db-pro-app` → 725 + 681 passed / 0 failed; fmt, clippy
  workspace, locked release build all 0.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | evidence matrix `65fad16d` · pass-1 fixes `1b94044e` `bc99ad10` `c735211d` `a87da3e6` `8ba71006` `faab6466` · docs trail = `git log` since matrix |
| Commit list | full history on `main` since `493e3b9b`; see `git log 493e3b9b..HEAD` |
| File / surface inventory | `docs/plans/active/layout-aesthetics-audit/{PLAN,RUBRIC,CHECKLIST,FINDINGS,VERIFICATION,REPORT,AGENT_EVIDENCE}.md` · `evidence/{manifest.json,capture_matrix.py,60 matrix PNGs,6 live PNGs}` · `crates/app/src/{explorer,settings,query_workbench,connection}` · `crates/ui/src/components/{common,nav,scroll_area,dialog}` |
| Acceptance mapping | rubric → `RUBRIC.md`; matrix → manifest 60/60 sha256-verified; two-pass scores → `REPORT.md`; findings → `FINDINGS.md` (F1–F15); independent pass → **partial (Table single number)** |
| Commands and counts | fmt 0 · clippy 0 · `db-pro-ui` 725 / `db-pro-app` 681 passed · release build 0 · 60/60 cells captured + 6 live |
| CI run IDs / status | not run (no PR) |
| Known limitations | 1440/1920 render at 838 logical height (host clamp); Table pass-1 number missing (gap admitted); cursor hover verified on 3 targets only; real-state cells (loading/error) not in matrix |
| Migrations / config implications | none; capture env vars are opt-in fixtures |
| Out-of-scope changes | `none` |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | pass 2 at `65fad16d`; live verification at `faab6466`+ |
| Verdict | `ACCEPT WITH P2` — no P0/P1 open |
| P0 / P1 / P2 counts | fixed: 0 / 1 / 2 (F7a P2, F7b P1, light-ladder P2 class) · open: F9 F10 F11 F12 F13 F15 (all P2) + F8 F14 (P3/P3-cluster) |
| Findings | `FINDINGS.md` (F1–F15) + `REPORT.md` (two-pass table + adjudication) |
| CI disposition | not executed |
| Next task(s) unblocked | F9 fix (scrollbar visibility wrapper) → own plan; F15 (winit cursor) → own plan; `native-visual-redesign` for F10–F12 |

## 5. Research / audit handoff

- **Source date:** 2026-10-23
- **References:** 9 pass-1 reports + 5 pass-2 reports (`agent://Audit*`, `agent://Score*`), `RUBRIC.md`,
  `evidence/manifest.json` (60 cells, sha256 each), 6 live Orca PNGs, `REPORT.md`.
- **Factual findings (measured):** light `canvas = elevated = editor = floating = #ffffff` with
  `panel #f6f6f6` before the ladder; `surface_2 = subtle = panel = #f6f6f6` made a selected row
  invisible in light; type scales 4–7 sizes per surface; wide layouts did not distribute (toolbar
  62% empty at 1920, welcome grid fixed 620 pt).
- **Pass-2 measurements (new, distinct):** Settings ~60% dead space; connection grid uses three
  column splits (~200px drift); 10 decorative accent headers on the diagram; active-tab light
  contrast below threshold; shared `ScrollArea` wrapper `AlwaysHidden` on every consumer.
- **Inference:** [INFERENCE] most light-theme weaknesses traced to the missing elevation step (fixed
  at `c735211d`); the remaining weakness cluster is affordance (F9) + consistency (F11–F13).
- **Decision / recommendation:** route F9+F15 to their own fixes (component wrapper + winit layer);
  F10–F13 to `native-visual-redesign`; F14 batch as polish.
- **Unresolved questions:** whether the light ladder should gain a 4th step; whether F15 is winit or
  egui (needs a minimal repro outside the app); whether the Table pass-1 number should be
  regenerated at the baseline SHA or accepted as missing.
- **Downstream tasks activated:** none yet — routing table only.

## 6. Tổng kết (Vietnamese summary)

Hai vòng chấm: 9 agent (pass 1, pre-fix, mean 17.4/32) → sửa light ladder + selection pill + input
boundary → 5 agent độc lập (pass 2, mean 23.4/32). Mọi delta ≥2 đã phán xử trong REPORT.

Live Orca xác nhận 4 sub-surface + dialog scroll (F8 không phải defect) + hover cursor (F15:
`on_hover_cursor` không tới OS — defect winit/egui, unit test xanh nhưng runtime sai).

Còn lại: Table chỉ có 1 con số (pass-1 leak Welcome, ghi gap rõ), F9–F15 chưa sửa (routing, không
trong scope audit). Không P0/P1 mở.
