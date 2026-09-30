# Agent evidence — Calendar & DatePicker intrinsic layout

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | codex · native UI polish lane |
| Issue(s) | n/a — user-reported Calendar & DatePicker layout defect |
| Task state | Done |
| Baseline SHA | `85e48b45c9a4d59bb882190e0ae2742eab407ea1` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Remove the Calendar frame's gallery-column stretch, keep its surface intrinsically sized, clamp the DatePicker popup, and record the current coverage boundary. |
| Out of scope | Keyboard DatePicker navigation, editable text entry, date constraints, localization/week-start configuration, and clear/today actions. |

## 2. Progress checkpoint

- Current HEAD (implementation checkpoint): `335f5012f21e7acc1bd73295067243241d25eac4`
- Completed acceptance rows: `[x] intrinsic Calendar surface`, `[x] viewport-clamped DatePicker popup`, `[x] focused layout test`, `[x] runtime captures at the three requested viewport targets`
- Remaining acceptance rows: `[ ] loading/error/empty gallery traversal`, `[ ] keyboard/accessibility interaction evidence`, `[ ] broader DatePicker feature coverage`
- Findings / risks: `P2` — the component remains click-oriented and has no editable or keyboard DatePicker path; this is documented as follow-up in `FINDINGS.md` and was not silently counted as covered.
- Tests already run: `cargo test -p db-pro-ui components::calendar:: --lib` → 8 passed / 0 failed / 0 ignored in the filtered test target, exit 0; `cargo test --workspace` → 1584 passed / 0 failed / 41 ignored, exit 0.
- Dependency / blocker changes: none. Runtime capture uses the repository's existing `capture` feature; no new dependency was added.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `335f5012f21e7acc1bd73295067243241d25eac4` |
| Commit list | `b8bf057b fix(ui): tighten calendar surface and popup placement`; `335f5012f fix(ui): remove calendar grid width reserve` |
| File / surface inventory | `crates/ui/src/components/calendar/ui.rs` — intrinsic frame allocation, shared preferred size, popup clamping, focused size test; `crates/ui/src/components/calendar/config.rs` — popup screen margin; `crates/ui/src/component_gallery_inputs.rs` and `crates/ui/src/workspace_actions.rs` — focused Calendar capture route; `docs/plans/active/core-component-style-system/{CHECKLIST,FINDINGS,VERIFICATION}.md` — scope, findings, gates and evidence; `evidence/calendar-datepicker-*.png` — native captures. |
| Acceptance mapping | Large right padding → `Calendar::show` now allocates a `272px` intrinsic frame instead of letting `Frame::show` inherit the gallery column, with no one-sided grid reserve; popup edge case → `clamp_popup_to_screen` with the same preferred size and an `8px` screen margin; coverage question → explicit covered/not-covered matrix in `FINDINGS.md`; visual acceptance → three committed PNG captures. |
| Commands and counts | `cargo fmt --all -- --check` → exit 0; `cargo check --workspace` → exit 0; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; `cargo test --workspace` → 1584 passed / 0 failed / 41 ignored, exit 0; `cargo build --release --locked -p db-pro-native` → exit 0; `cargo build --release --locked -p db-pro-native --features capture` → exit 0; `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` → 14 pass / 0 fail / 2 warning categories, exit 0. |
| CI run IDs / status | not run — local main workflow only |
| Known limitations | The host produced 2560×1600 for the 1280×800 target, 2880×1676 for the 1440×900 target, and 3840×1676 for the 1920×1080 target; the latter two are host-height capped. Capture proves the static surface, not keyboard focus or pointer interaction. |
| Migrations / config implications | none; no persisted state, provider behavior, or database contract changed. The capture-only `DB_PRO_CAPTURE_GALLERY_SECTION=calendar` route is test tooling only. |
| Out-of-scope changes | no provider/runtime/database changes; no worktree created; no push performed. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `335f5012f21e7acc1bd73295067243241d25eac4` |
| Verdict | ACCEPT WITH P2 |
| P0 / P1 / P2 counts | introduced by this SHA: 0 / 0 / 1; inherited: 0 / 0 / 0 |
| Findings | P2 follow-up only: keyboard/accessibility and richer DatePicker feature cases remain outside this focused layout fix. |
| CI disposition | not run; all required local gates listed above passed at the implementation SHA. |
| Next task(s) unblocked | broader DatePicker interaction design and accessibility pass |

## 5. Research / audit handoff

- Source date: 2026-10-01, local repository inspection and native capture.
- Source URLs / references: `crates/ui/src/components/calendar/ui.rs`, `crates/ui/src/components/calendar/handler.rs`, `crates/ui/src/components/calendar/config.rs`, `crates/ui/src/components/common/layout.rs`, `crates/ui/src/component_gallery_inputs.rs`, and `docs/10-egui-native-migration-plan.md`.
- Factual findings: egui `Frame::begin` starts from `available_rect_before_wrap`, so the former Calendar frame inherited the gallery column width; the fixed content width did not constrain the frame's outer paint rect. The final code wraps the frame in an explicit `allocate_ui_with_layout` region and reuses a preferred size for popup placement.
- Inference: a future full DatePicker should likely share the same semantic size/position helper while adding an explicit keyboard and constraint model; that is a recommendation, not current behavior.
- Decision / recommendation: accept the focused P2 visual fix; schedule the richer interaction model separately instead of expanding this patch.
- Unresolved questions: desired locale and week-start policy; whether direct text entry should replace or complement the click calendar; product rules for min/max and disabled dates.
- Downstream tasks activated: none.

## 6. Tổng kết bằng tiếng Việt

Đã sửa Calendar để frame ôm đúng kích thước nội tại, không còn nền/card kéo dài theo toàn bộ cột gallery; DatePicker popup cũng được clamp theo viewport. Test Calendar 8/8, workspace 1584 passed / 0 failed / 41 ignored, clippy/build/clean-code đều đạt, và đã lưu capture 3 viewport vào plan. Các case keyboard, nhập text, min/max, disabled date, locale và clear/today vẫn là P2 follow-up, chưa coi là đã cover.
