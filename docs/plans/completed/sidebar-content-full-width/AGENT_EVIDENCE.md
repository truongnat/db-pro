# Agent evidence — sidebar-content-full-width (closure)

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | fix lane (sidebar layout / clip) — closure handoff |
| Issue(s) | user-reported: tree still overlapped by the width (driver badge hidden) + filter field border missing both ends |
| Task state | Done (claimed, implemented, gated, runtime-verified) |
| Baseline SHA | `a3a6a5290e3cf992354ee187169818a5c57122ac` — the root-cause fix was authored off this base; this handoff rebases onto current `main` (`1a20eb25`) and closes the runtime-evidence gate |
| Branch / PR | `fix/sidebar-column-overflow` (rebased tip `7b9cfc14`) · merged into `main` as part of this closure; no PR (owner opted to merge directly into `main`) |
| Scope interpretation | rebase the sidebar fix onto the now-advanced `main` (which carries the new-connection secret+input fix), close the UI runtime-evidence gate by capturing the affected surface, flip the plan to `COMPLETED`, and merge into `main` |
| Out of scope | the `SearchInput` `36.0` / `20.0` P2 reservations; the outside-stroke loss on non-sidebar columns (both documented as P2 residuals in `FINDINGS.md`); provider connectors; Tauri host; `SearchInput` width contract |

## 2. Progress checkpoint

- Current branch HEAD (pre-merge): `7b9cfc14` (rebase of original `55593272` onto `main` `1a20eb25`)
- Rebase result: clean, no conflicts (sidebar fix touched `sidebar_view.rs`, `explorer_view.rs`, `explorer_tree.rs`, `components/input/{layout,search}.rs`, `app_tests.rs`; the new-connection fix touched `components/input/{text,password,tests}.rs` — disjoint files in `components/input/`, so no overlap)
- Completed acceptance rows: content rect locked to `sidebar_width`; drag separator locked; ScrollArea / tree rows forced to content width; header paint order fixed; duplicate Plus removed; explorer toolbar lays refresh button out first; `tree_width` bound before `ScrollArea`; driver badge width measured from the label; sidebar 1px clip bleed; `paint_field_chrome` inset path
- Remaining acceptance rows: none — all five revert-verified guards pass; runtime screenshots captured at all three gate sizes
- Findings / risks: P1 tree-row overflow (fixed, runtime-verified); P1 1px border loss on flush widgets (fixed, runtime-verified); P2 residuals (SearchInput guesses, non-sidebar outside-stroke) logged
- Tests already run (rebased tree): `cargo clippy --workspace --all-targets --offline -- -D warnings` → exit 0; `cargo test --workspace --offline` → **518 passed / 0 failed** (plus ignored, unchanged); `cargo build --release --locked -p db-pro-native` → exit 0; clean-code `--diff --ratchet --ci` → 15 pass / 1 warn / 0 fail
- Dependency / blocker changes: none (the `capture` feature already lives on `main`, so it lands on the branch via rebase — no new infra needed)

## 3. Implementation handoff

| Field | Value |
|---|---|
| Original fix SHA | `55593272` (pre-rebase) |
| Rebased tip | `7b9cfc14` (rebased onto `main` `1a20eb25`; no conflicts) |
| Closure commit(s) | planned on the branch: the runtime-evidence commit (CHECKLIST / VERIFICATION / FINDINGS flipped to COMPLETED + screenshots + capture-intent commit message) and the move-to-completed commit; both land before the merge |
| File / surface inventory | `crates/ui/src/sidebar_view.rs` — clip bleed; `crates/ui/src/explorer_view.rs` — toolbar `right_to_left` + `tree_width` bound; `crates/ui/src/explorer_tree.rs` — badge width measured from painter; `crates/ui/src/components/input/layout.rs` + `search.rs` — `paint_field_chrome` inset path; `crates/ui/src/app_tests.rs` — five revert-verified guards |
| Acceptance mapping | "tree still overlapped by the width, badge hidden" → `the_driver_badge_is_fully_visible_in_the_navigator_tree` + runtime screenshots; "filter field border missing both ends" → `the_filter_field_border_is_painted_inside_the_field` + runtime screenshots |
| Commands and counts | `cargo clippy --workspace --all-targets --offline -- -D warnings` → exit 0; `cargo test --workspace --offline` → 518/0; `cargo build --release --locked -p db-pro-native` → exit 0; clean-code `--diff --ratchet --ci` → 15/1/0; `cargo build --release --locked --offline -p db-pro-native --features capture` → exit 0; capture runs (1280×800 / 1440×900 / 1920×1080) → 3 PNGs in `screenshots/`, all on the default explorer view with a populated store |
| CI run IDs / status | not run (no CI trigger in this session) |
| Known limitations | static screenshots cannot show hover / scrollbar / drag states — these are covered by the five revert-verified guards (`the_sidebar_paints_nothing_past_its_clip`, `the_driver_badge_is_fully_visible_in_the_navigator_tree`, `the_filter_field_border_is_painted_inside_the_field`, `the_tree_row_spans_its_layout_width_not_its_clip`) |
| Migrations / config implications | none |
| Out-of-scope changes | none on this branch |

## 4. Review outcome

Self-verified. P0 = 0, P1 = 0 (fixed and runtime-verified), P2 = 2 (logged: `SearchInput` `36.0` / `20.0` reservations; outside-stroke loss on non-sidebar columns).

## 5. Research / audit handoff

- Source date: 2026-09-17
- Source URLs / references: `crates/ui/src/sidebar_view.rs`, `crates/ui/src/explorer_view.rs`, `crates/ui/src/explorer_tree.rs`, `crates/ui/src/components/input/{layout,search}.rs`; `docs/plans/active/sidebar-content-full-width/{PLAN,FINDINGS,VERIFICATION,CHECKLIST}.md`
- Factual findings: `Ui::set_max_width` ends with `max_rect = max_rect.union(min_rect)` (`placer.rs`); `Painter::with_clip_rect` *intersects*; `Shape::rect_stroke` paints entirely outside its path (`StrokeKind::Outside`, `tessellator.rs`); `ScrollArea` narrows its clip by the scrollbar (`scroll_area.rs`)
- Inference: the sidebar defects are *clip*-defects (layout vs paint clip), not tessellation defects — disproved by rasterizing the tessellator's own output
- Decision / recommendation: ship as-is; runtime evidence now captured (PNGs in `screenshots/`); plan moves to `completed/`
- Unresolved questions: none for this scope
- Downstream tasks activated: none

## 6. Tổng kết (Vietnamese summary)

Đóng gate runtime-evidence cho fix sidebar (`fix/sidebar-column-overflow`): rebase sạch lên
`main` (`1a20eb25`), capture 3 PNG (1280×800 / 1440×900 / 1920×1080) bằng `capture` feature
cho thấy driver badge `SQLITE` / `PG` đọc được đầy đủ và border filter + `New query` kín 4
cạnh — đúng hai P1 mà user báo. CHECKLIST / VERIFICATION / FINDINGS flipped sang COMPLETED.
Gates xanh: clippy -D warnings sạch, 518 test pass, release build sạch, clean-code 15/1/0.
P2 residuals (`SearchInput` reservations, outside-stroke ngoài sidebar) đã ghi nhận, không
blocker. Bước tiếp: commit docs + screenshots trên branch, move plan sang `completed/`, rồi
fast-forward merge vào `main`.