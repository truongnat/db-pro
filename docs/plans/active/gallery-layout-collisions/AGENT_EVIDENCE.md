# Agent evidence — gallery layout collisions fix

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Devin (interactive CLI session) |
| Issue(s) | user report "UI hỏng" + 3 screenshots (no issue #) |
| Task state | In Progress → Review (handoff) |
| Baseline SHA | `78d914887642dd400c7e2048db9a24d0403619c8` — all assertions vs this tree + the fix diff below |
| Branch / PR | `main` (owner workflow override; no worktree, no PR created) |
| Scope interpretation | Fix four confirmed egui layout/visibility collisions on Component Gallery + production-shared components (`ExplainPlanTree` also used by `query_output_actions_view.rs`) |
| Out of scope | systemic `ui.columns`/`id().with` audit beyond the captured collisions; production `QueryStatusBar` layout (separate implementation, no observed collision); archived frontend |

## 2. Progress checkpoint

- Current HEAD: `78d914887642dd400c7e2048db9a24d0403619c8` (fix committed as new commit on top)
- Acceptance rows: all done — see `CHECKLIST.md`
- Findings: P2 only — `FINDINGS.md` §F1–F5
- Tests run: `cargo test -p db-pro-ui --offline` → **949 passed / 0 failed / 0 ignored** (exit 0)

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | top commit on `main` — `fix(ui): bound layout collisions across gallery surfaces` (baseline `78d91488`; a commit cannot embed its own SHA) |
| Commit list | 1 commit — `fix(ui): bound layout collisions across gallery surfaces` |
| File / surface inventory | `components/explain/ui.rs` (stat region ≥ text width, header labels truncate, ScrollArea `id_salt(tree_id)`) · `components/database/ui.rs` (SSL gap = `item_spacing.x + SPACE_XS`) · `components/workspace/ui.rs` + `handler.rs` (right-block measurement, left bound, `…` overflow, tests) · `components/diff/ui.rs`, `components/dev_tools/ui.rs` (scroll `id_salt`s) · `component_gallery_rendering.rs` (theme-based alpha glyphs) · `component_gallery_view.rs`, `workspace_actions.rs` (capture-only `SCROLL`/`rendering` envs) |
| Acceptance mapping | F1→explain ui.rs; F2→database ui.rs; F3→workspace ui.rs/handler.rs + tests; F4→rendering; F5→id_salt trio |
| Commands and counts | fmt PASS · `cargo check --workspace --offline` PASS · clippy db-pro-ui `-D warnings` PASS · tests 949/0/0 · clean-code 14/2 warn/0 fail · perf scan PASS partial (4/4 executed) |
| CI run IDs / status | not run — local-only branch, no PR |
| Known limitations | `render_node`/`ConnectionCard::show` exceed the 50-line heuristic (pre-existing ratchet warnings); captures above 1280×800 logical clamp height to 838 (display limit) |
| Migrations / config implications | none — two new capture env vars are opt-in |
| Out-of-scope changes | none |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `n/a` — implementer handoff; external review not performed |
| Verdict | n/a |
| P0 / P1 / P2 counts | 0 / 0 / 0 introduced; 4 P2 fixed + 1 P2 defect discovered-and-fixed |
| Findings | n/a |
| CI disposition | not run |
| Next task(s) unblocked | none |

## 5. Research / audit handoff

- Source date: 2026-10-02 session
- References: `crates/ui/src/components/{explain,database,workspace,diff,dev_tools}/`, `component_gallery_{surfaces,rendering,view}.rs`, egui 0.29.1 `ui.rs` (`columns_dyn`, `new_child`) + `scroll_area.rs:526` (`Id::new("scroll_area")`)
- Factual findings + inference separation: FINDINGS.md
- Unresolved questions: full inventory of `id().with()` collisions across other gallery sections (P2 follow-up)
- Downstream tasks activated: none

## 6. Tổng kết (Vietnamese summary)

Đã sửa 4 lỗi UI xác nhận bằng capture thực tế: text chồng nhau trong Execution Plan (vùng stat được cấp tối thiểu bằng độ rộng text thay vì tràn sang trái), card kết nối tràn 8pt sang cột kế (do `item_spacing.x` bị tính hai lần), StatusBar trái/phải đè nhau (đo block phải trước, bên trái dừng sớm với dấu `…`), và hàng mẫu glyph vô hình ở light theme (màu lấy từ `theme.text_primary`). Phát hiện và sửa thêm va chạm widget Id của egui giữa các `ScrollArea` không `id_salt` trong `ui.columns`. Gates: fmt/check/clippy/test (949/0) pass; clean-code 0 fail; perf scan PASS. Bằng chứng capture: `docs/plans/active/gallery-layout-collisions/evidence/`.
