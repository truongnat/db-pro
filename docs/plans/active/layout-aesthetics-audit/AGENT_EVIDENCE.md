# Agent evidence — Layout Aesthetics Audit

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | omp · audit lane (evaluation + one routed defect fix) |
| Issue(s) | Owner request for an aesthetics/design evaluation; owner follow-up "cái này cũng siêu xấu" on the Explorer filter popover |
| Task state | `In Progress` — rubric and matrix exist, scoring and the independent pass do not |
| Baseline SHA | `493e3b9b`; the audit's own fix landed at `1b94044e`, helper refactor after it |
| Branch / PR | `main` (direct edit per owner override) · no PR |
| Scope interpretation | Judge the native layout against an eight-dimension rubric with measured evidence, route findings to owning plans, and fix the owner-reported popover defect as a routed finding (F7a). |
| Out of scope | Redesign proposals without a finding; provider work; motion recording; scoring surfaces that were not captured. |

## 2. Progress checkpoint

- Current HEAD: working tree on top of `c245183e` (helper refactor not yet committed at writing)
- Completed rows:
  - [x] `RUBRIC.md` — eight dimensions, 0–4 anchors, measurement list per dimension
  - [x] 60-cell capture matrix (10 surfaces × 3 widths × 2 themes) with a committed runner
  - [x] Explorer popover inset fixed and re-captured in all six cells
  - [x] Findings F7a (fixed) and A1–A6 (open, routed)
- Remaining rows: inspection of 54 of 60 cells, all scoring, the declared sub-surfaces
  (Table structure / Settings Appearance / Connection error), real states, independent pass, `REPORT.md`
- Findings / risks: F7a P2 fixed; A1–A6 unrated and therefore not yet prioritised
- Tests already run: `cargo test --workspace` 2,272 passed / 0 failed / 65 ignored; `cargo test -p
  db-pro-app` 681 passed; fmt and workspace clippy exit 0; locked release build exit 0
- Dependency / blocker changes: none

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `1b94044e` (popover fix) · `c245183e` (manifest + runner) · helper refactor pending commit |
| Commit list | `5829c0ce` plan opened · `7e81a380` F2 correction · `d7c23aa5` audit handoff + rubric alignment · `1b94044e` popover inset · `c245183e` honest manifest + runner |
| File / surface inventory | `docs/plans/active/layout-aesthetics-audit/{PLAN,RUBRIC,CHECKLIST,FINDINGS,VERIFICATION,AGENT_EVIDENCE}.md` · `evidence/manifest.json` · `evidence/capture_matrix.py` · six Explorer PNGs · `crates/app/src/explorer/explorer_toolbar_view.rs` (inset) · `crates/ui/src/components/common/frames.rs` (`popup_content_frame`) |
| Acceptance mapping | "rubric published" → `RUBRIC.md`; "matrix captured" → manifest 60/60; "every image inspected" → **6/60, open**; "scores with numbers" → not started; "independent pass" → not started |
| Commands and counts | fmt 0 · clippy workspace 0 · `cargo test --workspace` 2,272 / 0 / 65 · `cargo test -p db-pro-app` 681 / 0 · locked release build 0 · ui-architecture PASS |
| CI run IDs / status | not run (no PR) |
| Known limitations | 1440/1920 render at 838 logical height; six cells exit `-6` from the post-capture TouchBar teardown; 54 cells are reproducible but not retained in the repo (SHA recorded); no real-state or sub-surface captures yet |
| Migrations / config implications | none; capture env vars are opt-in fixtures |
| Out-of-scope changes | `none` |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | n/a — no independent review of the audit yet |
| Verdict | n/a |
| P0 / P1 / P2 counts | fixed: 0 / 0 / 1 (F7a) · open observations: 6, unrated |
| Findings | `FINDINGS.md` — F7a fixed with before/after evidence; A1–A6 open |
| CI disposition | not executed |
| Next task(s) unblocked | inspect the remaining 54 cells, then score; add the three declared sub-surfaces |

## 5. Research / audit handoff

- **Source date:** 2026-10-23
- **References:** `RUBRIC.md`, `evidence/manifest.json`, the six Explorer PNGs, `crates/ui/src/theme/tokens/semantic.rs`, `crates/ui/src/components/selection/config.rs`, `crates/app/src/explorer/explorer_toolbar_view.rs`.
- **Factual findings:**
  - The popover's content sat at `menu_margin = SPACE_XS` (4 pt) from its rounded edge, so the divider
    and rows bled into the edge; measured from the pre-fix capture and confirmed in source.
  - `CONTROL_SIZE = SPACE_LG` (16 pt) against ~11–13 pt labels, component-wide (A5).
  - In light mode the Welcome cards are white on a white canvas with a hairline border only, while
    dark uses an elevated fill (A1).
- **Inference:** [INFERENCE] the light elevation gap and the wide-width imbalance (A6) come from the
  canvas/elevated roles rather than from the cards' own frames; needs the panel-role experiment
  before a fix is proposed.
- **Decision / recommendation:** keep the audit `PLANNING`; score only inspected cells; the popover
  fix is a routed finding, not a redesign.
- **Unresolved questions:** independent reviewer identity; whether motion should be judged at all.
- **Downstream tasks activated:** A5 → `core-component-style-system`; A1/A4/A6 → `native-visual-redesign`; A2 → audit follow-up.

## 6. Tổng kết (Vietnamese summary)

Audit đang ở `PLANNING`, chưa chấm điểm. Đã có: rubric 8 chiều × 0–4 với anchor và danh sách số đo,
matrix 60 ô (10 surface × 3 kích thước × 2 theme) cùng manifest ghi rõ ô nào **được giữ lại** và ô nào
**đã đọc** — hiện chỉ 6 ô Explorer đã đọc, 54 ô còn lại mới capture.

Đã sửa lỗi bạn báo ("siêu xấu"): popover filter của Explorer trước đây chỉ cách mép 4pt nên dòng kẻ và
`Reset` dính sát viền; nay nội dung thụt 8pt qua helper `popup_content_frame` trong tầng component
(không dựng `egui::Frame` thô trong view), có khoảng thở trước divider/header/Reset. Ảnh 6 ô
Explorer đã capture lại và đọc.

Gate trên cây hiện tại: fmt 0, clippy workspace 0, workspace **2.272 pass / 0 fail / 65 ignored**,
`db-pro-app` 681 pass, release build locked 0, ui-architecture PASS.

Còn thiếu: đọc + chấm 54 ô, ba sub-surface đã khai báo (Table structure, Settings Appearance,
Connection error), state thật, và vòng chấm độc lập. Vì vậy chưa có `REPORT.md` và chưa chuyển trạng
thái.
