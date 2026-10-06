# Agent evidence — Calendar & DatePicker intrinsic layout

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | codex · native UI polish lane |
| Issue(s) | n/a — user-reported Calendar & DatePicker layout defect |
| Task state | Done |
| Baseline SHA | `85e48b45c9a4d59bb882190e0ae2742eab407ea1` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Remove the Calendar frame's gallery-column stretch, keep its surface intrinsically sized, make the seven-column grid responsive to narrow gallery columns, constrain the header to the grid width, clamp the DatePicker popup, and record the current coverage boundary. |
| Out of scope | Keyboard DatePicker navigation, editable text entry, date constraints, localization/week-start configuration, and clear/today actions. |

## 2. Progress checkpoint

- Current HEAD (implementation checkpoint): `d5ab476fb8a9c67aca0e3cf89f881dfbf840b33d` (source behavior; documentation follows in the next commit)
- Completed acceptance rows: `[x] intrinsic Calendar surface`, `[x] viewport-clamped DatePicker popup`, `[x] responsive seven-column grid`, `[x] fixed three-region header`, `[x] focused layout tests`, `[x] runtime captures at the three requested viewport targets plus narrow regression target`
- Remaining acceptance rows: `[ ] loading/error/empty gallery traversal`, `[ ] keyboard/accessibility interaction evidence`, `[ ] broader DatePicker feature coverage`
- Findings / risks: `P2` — the component remains click-oriented and has no editable or keyboard DatePicker path; this is documented as follow-up in `FINDINGS.md` and was not silently counted as covered.
- Tests already run: `cargo test -p db-pro-ui components::calendar:: --lib` → 10 passed / 0 failed / 0 ignored in the filtered test target, exit 0; `cargo test -p db-pro-ui --lib` → 929 passed / 0 failed / 0 ignored, exit 0; `cargo test --workspace` → exit 0 with the UI segment at 929 passed / 0 failed / 0 ignored.
- Dependency / blocker changes: none. Runtime capture uses the repository's existing `capture` feature; no new dependency was added.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `d5ab476fb8a9c67aca0e3cf89f881dfbf840b33d` |
| Commit list | `b8bf057b fix(ui): tighten calendar surface and popup placement`; `335f5012f fix(ui): remove calendar grid width reserve`; `cb1ced7a fix(ui): keep calendar grid visible in narrow gallery`; `d5ab476f fix(ui): constrain calendar header to grid width` |
| File / surface inventory | `crates/ui/src/components/calendar/ui.rs` — intrinsic frame allocation, responsive cell sizing, fixed three-region header, popup clamping, focused layout tests; `crates/ui/src/components/calendar/config.rs` — popup screen margin; `crates/ui/src/component_gallery_inputs.rs` and `crates/ui/src/workspace_actions.rs` — responsive gallery and focused Calendar capture route; `docs/plans/active/core-component-style-system/{CHECKLIST,FINDINGS,VERIFICATION}.md` — scope, findings, gates and evidence; `evidence/calendar-datepicker-*.png` — native captures. |
| Acceptance mapping | Large right padding → `Calendar::show` now allocates a `272px` intrinsic frame instead of letting `Frame::show` inherit the gallery column, with no one-sided grid reserve; narrow clipping → `ResponsiveGrid` reflows the gallery and `calendar_layout` shrinks cells to the local width while preserving all seven columns; header overflow → fixed `prev | title | next` regions share the grid width and zero inter-item spacing; popup edge case → `clamp_popup_to_screen` with the same preferred size and an `8px` screen margin; coverage question → explicit covered/not-covered matrix in `FINDINGS.md`; visual acceptance → committed wide and narrow PNG captures. |
| Commands and counts | `cargo fmt --all -- --check` → exit 0; `cargo check --workspace` → exit 0; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; `cargo test --workspace` → exit 0 with the UI segment at 929 passed / 0 failed / 0 ignored; `cargo build --release --locked -p db-pro-native` → exit 0; `cargo build --release --locked -p db-pro-native --features capture` → exit 0; `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` → 14 pass / 0 fail / 2 warning categories, exit 0. |
| CI run IDs / status | not run — local main workflow only |
| Known limitations | The host produced 2560×1600 for the 1280×800 target, 2880×1676 for the 1440×900 target, and 3840×1676 for the 1920×1080 target; the latter two are host-height capped. Capture proves the static surface, not keyboard focus or pointer interaction. |
| Migrations / config implications | none; no persisted state, provider behavior, or database contract changed. The capture-only `DB_PRO_CAPTURE_GALLERY_SECTION=calendar` route is test tooling only. |
| Out-of-scope changes | no provider/runtime/database changes; no worktree created; no push performed. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `d5ab476fb8a9c67aca0e3cf89f881dfbf840b33d` |
| Verdict | ACCEPT WITH P2 |
| P0 / P1 / P2 counts | introduced by this SHA: 0 / 0 / 1; inherited: 0 / 0 / 0 |
| Findings | P2 follow-up only: keyboard/accessibility and richer DatePicker feature cases remain outside this focused layout fix. |
| CI disposition | not run; all required local gates listed above passed at the implementation SHA. |
| Next task(s) unblocked | broader DatePicker interaction design and accessibility pass |

## 5. Research / audit handoff

- Source date: 2026-10-01, local repository inspection and native capture.
- Source URLs / references: `crates/ui/src/components/calendar/ui.rs`, `crates/ui/src/components/calendar/handler.rs`, `crates/ui/src/components/calendar/config.rs`, `crates/ui/src/components/common/layout.rs`, `crates/ui/src/component_gallery_inputs.rs`, and `docs/10-egui-native-migration-plan.md`.
- Factual findings: egui `Frame::begin` starts from `available_rect_before_wrap`, so the former Calendar frame inherited the gallery column width; the fixed content width did not constrain the frame's outer paint rect. A fixed three-column gallery could also clip the Calendar when its local width fell below the seven-column grid, and the former flexible header middle could push the next button outside the grid width. The final code wraps the frame in an explicit `allocate_ui_with_layout` region, reflows the gallery, scales cells to local width, fixes the header to three regions, and reuses a preferred size for popup placement.
- Inference: a future full DatePicker should likely share the same semantic size/position helper while adding an explicit keyboard and constraint model; that is a recommendation, not current behavior.
- Decision / recommendation: accept the focused P2 visual fix; schedule the richer interaction model separately instead of expanding this patch.
- Unresolved questions: desired locale and week-start policy; whether direct text entry should replace or complement the click calendar; product rules for min/max and disabled dates.
- Downstream tasks activated: none.

## 6. Tổng kết bằng tiếng Việt

Đã sửa đủ ba lớp lỗi: frame không còn kéo dài theo cột gallery, grid không còn bị clip khi viewport hẹp, và header không còn đẩy nút phải làm phát sinh padding dư. Calendar co cell để vẫn đủ 7 cột; header dùng ba vùng cố định. Test Calendar 10/10, UI 929 passed, workspace exit 0, clippy/build/clean-code đều đạt. Các case keyboard, nhập text, min/max, disabled date, locale và clear/today vẫn là P2 follow-up, chưa coi là đã cover.

## 7. Component Gallery surface follow-up

| Field | Value |
|---|---|
| Exact SHA | `8ca9cac5470ec6688c52aff4cc7e09a0aacdad16` |
| Task state | Done |
| Scope | Recompose Cards & Metric Displays for responsive density, restore metric-label contrast through `DbProTheme`, and capture the light surface at 1280×800, 1440×900, and 1920×1080. |
| Runtime result | 2×2 at medium widths, four columns when the content region is wide enough, and one-column fallback below the minimum two-card width; no orphan third-row card at the supported gallery widths. |
| Evidence | `evidence/cards-metrics-1280x800.png`, `evidence/cards-metrics-1440x900.png`, `evidence/cards-metrics-1920x1080.png`. |
| Gates | fmt, workspace check, clippy, workspace test (929 UI tests), release build, clean-code scan, and diff check all passed after the implementation SHA. |
| P0 / P1 / P2 | 0 / 0 / 1; loading/error/empty traversal remains a documented P2 follow-up for the static gallery surface. |
| Workflow | Implemented directly on local `main`; no worktree and no push. |

### Tổng kết bằng tiếng Việt

Cards & Metric Displays đã được đưa về layout responsive có chủ đích: vùng trung bình giữ 2×2, vùng đủ rộng mới dùng 4 cột, nhãn metric dùng token semantic dễ đọc hơn. Đã capture light theme ở cả ba kích thước và chạy đủ gate Rust/UI; loading/error/empty vẫn là follow-up vì gallery hiện là surface tĩnh.

## 8. ResponsiveGrid vertical-gap follow-up

| Field | Value |
|---|---|
| Exact SHA | `42d78e203b67bde917438ec424a8f9c8a2d206b` |
| Task state | Done |
| Scope | Add intentional vertical gaps between wrapped `ResponsiveGrid` rows while preserving the existing horizontal token and caller spacing. |
| Root cause | The primitive set `item_spacing.x` only; the second metric row therefore had no explicit vertical rhythm. |
| Fix / coverage | `ResponsiveGrid` now scopes vertical spacing, inserts the configured gap between rows, and has focused coverage asserting the exact row separation. |
| Runtime evidence | Refreshed `evidence/cards-metrics-1280x800.png` and `evidence/cards-metrics-1440x900.png`; the 1920×1080 four-column capture has one row. |
| Gates | Focused test, workspace tests (929 UI tests), fmt, check, clippy, release build, clean-code scan, and diff check passed. |
| Workflow | Implemented directly on local `main`; no worktree and no push yet at this evidence checkpoint. |

### Tổng kết bằng tiếng Việt

Đã sửa đúng lỗi spacing: `ResponsiveGrid` trước đó chỉ có gap ngang, nên các row dọc bị dính. Primitive giờ dùng cùng gap token cho cả hai chiều, có test xác nhận khoảng cách 8px và capture 1280/1440 đã thấy khoảng thở giữa hai hàng.

## 9. Data Grid pagination footer follow-up

### 1. Claim

| Field | Value |
|---|---|
| Agent identity | codex · native UI polish lane |
| Issue(s) | n/a — user-requested Data Grid layout correction |
| Task state | Done |
| Baseline SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` (implementation remains uncommitted in the working tree) |
| Branch / PR | `main` / no PR |
| Scope interpretation | Move Data Grid pagination to the bottom result-grid footer and place Save, Discard, Refresh at its left. Keep Add Row, filter, and sort controls in the top toolbar. |
| Out of scope | Loading/error/empty placeholder design and unrelated prior UI changes. |

### 2. Progress checkpoint

- Current HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4`; it is the baseline, not an immutable representation of the uncommitted implementation.
- Completed acceptance rows: `[x] footer actions and pagination moved below the grid`, `[x] staged-change safety retained`, `[x] lesson added to CHECKLIST.md`, `[x] normal-state runtime captures at all three requested width targets`.
- Remaining acceptance rows: `[ ] loading/error/empty state captures`.
- Findings / risks: `P2` — non-result placeholders do not show this footer; the three 1440/1920 captures are host-height capped and document only the normal state.
- Tests already run: no tests run (not requested). `cargo fmt --all -- --check`, `git diff --check`, and release capture-feature build passed (exit 0).
- Dependency / blocker changes: none.

### 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | No implementation commit; working-tree diff is based on `710ba002612c6d71ef2605f99f2a49743b51c3a4`. |
| Commit list | none |
| File / surface inventory | `crates/ui/src/table_data_toolbar_surface_view.rs` — split top toolbar from footer; `table_data_mutation_toolbar_view.rs` — persistent Save/Discard/Refresh controls and staged-change gating; `table_data_surface_view.rs` — reserve footer space below grid; `table_data_view.rs` — render footer and route existing actions; `CHECKLIST.md` — reusable lesson and follow-up; `evidence/data-grid-footer-*.png` — native captures. |
| Acceptance mapping | Footer location/actions → table-data surface + toolbar composition; Save/Discard gating and refresh guard → mutation toolbar; lesson and runtime evidence → checklist/evidence images. |
| Commands and counts | `cargo fmt --all -- --check` → exit 0; `git diff --check` → exit 0; `cargo build --release --locked -p db-pro-native --features capture` → exit 0. Tests not run. |
| CI run IDs / status | not run |
| Known limitations | State captures for loading/error/empty not collected. Capture targets 1440×900 and 1920×1080 produced host-capped 2880×1676 and 3840×1676 PNGs; 1280×800 produced 2560×1600. |
| Migrations / config implications | none |
| Out-of-scope changes | Existing staged `quick-open-action-visuals` files and other earlier working-tree UI edits were not staged or reverted. |

### 4. Review outcome

n/a — implementer handoff; no independent review performed.

### 5. Research / audit handoff

n/a — focused implementation task.

### 6. Tổng kết bằng tiếng Việt

Pagination đã xuống footer dưới grid; Save, Discard, Refresh nằm bên trái và pagination bên phải. Save/Discard chỉ bật khi có thay đổi chờ lưu; refresh vẫn bị chặn trong trạng thái đó. Build và ba capture trạng thái thường đã đạt; loading/error/empty chưa capture. Thay đổi còn ở working tree, chưa commit.

## 10. Data Grid inline SQL header follow-up

### 1. Claim

| Field | Value |
|---|---|
| Agent identity | codex · native UI polish lane |
| Issue(s) | n/a — user-requested Data Grid header simplification |
| Task state | Done; inline query runtime submission remains unverified |
| Baseline SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` (changes remain uncommitted in the working tree) |
| Branch / PR | `main` / no PR |
| Scope interpretation | Keep Add Row and a direct SQL entry field in the Data Grid header; Enter hands execution to the existing Query path. Keep pagination and Save/Discard/Refresh in the bottom footer. |
| Out of scope | Broad UI redesign, SQL multiline editing, and changing query-result rendering semantics. |

### 2. Progress checkpoint

- Current HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4`; it is the baseline, not an immutable representation of the uncommitted implementation.
- Completed acceptance rows: `[x] top toolbar replaces filter/sort controls with direct SQL entry beside Add Row`, `[x] staged changes and running-query states block unsafe/ambiguous handoff`, `[x] existing query dispatcher preserves destructive-query confirmation`, `[x] query output panel opens after handoff`, `[x] checklist lesson and normal-state captures at 1280×800, 1440×900, and 1920×1080 recorded`.
- Remaining acceptance rows: `[ ] runtime submit against a database and verify result visibility`.
- Findings / risks: the SQL field is intentionally single-line; Enter executes without creating or activating a Query tab and routes returned rows into the grid. No actual query was submitted during runtime verification.
- Tests already run: no tests run (not requested). `cargo fmt --all -- --check`, `git diff --check`, `cargo check --locked -p db-pro-ui`, and `cargo build --release --locked -p db-pro-native --features capture` passed (exit 0).
- Dependency / blocker changes: none.

### 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | No implementation commit; working-tree diff is based on `710ba002612c6d71ef2605f99f2a49743b51c3a4`. |
| Commit list | none |
| File / surface inventory | `crates/ui/src/table_data_toolbar_surface_view.rs` — direct SQL input and reduced header; `crates/ui/src/table_data_query_state.rs` — draft and inline request state; `crates/ui/src/table_data_view.rs` and `events_query_dispatch.rs` — execute without opening a Query tab; `query_execution_actions.rs` — preserve shared dispatch history/safety; `query_result_events.rs` and `runtime_event_handlers.rs` — route results/errors/cancellation back to the grid; `table_data_filter_view.rs` — active filter chips remain removable below header; `app_modules.rs` and removed `table_data_sort_view.rs` — remove obsolete toolbar sort UI; `CHECKLIST.md` — reusable lesson; `evidence/data-grid-query-header-*.png` — native runtime captures. |
| Acceptance mapping | Header content → toolbar composition; execution safety and output visibility → table-data query handoff; review evidence → checklist and native capture. |
| Commands and counts | `cargo fmt --all -- --check` → exit 0; `git diff --check` → exit 0; `cargo check --locked -p db-pro-ui` → exit 0; `cargo build --release --locked -p db-pro-native --features capture` → exit 0. Tests not run. |
| CI run IDs / status | not run |
| Known limitations | Runtime captures confirm normal-state layout at all three requested targets; 1440×900 and 1920×1080 are host-height capped at 2880×1676 and 3840×1676. Actual SQL submission/result and loading/error/empty-state evidence remain pending. |
| Migrations / config implications | none |
| Out-of-scope changes | Existing staged `quick-open-action-visuals` files and other earlier working-tree UI edits were not staged or reverted. |

### 4. Review outcome

n/a — implementer handoff; no independent review performed.

### 5. Research / audit handoff

n/a — focused implementation task.

### 6. Tổng kết bằng tiếng Việt

Header Data Grid có Add Row và ô nhập SQL; Enter chạy qua query safety/runtime path hiện có và đưa kết quả về grid, không mở Query tab. Khi đang có staged changes hoặc query khác chạy thì bị chặn. Capture xác nhận header gọn và footer ở đáy vùng grid. Build/format/diff-check đạt; chưa chạy SQL thực tế trong phiên xác minh.

## 11. Data Grid clarity and one-page controls follow-up

### 1. Claim

| Field | Value |
|---|---|
| Agent identity | codex · native UI polish lane |
| Issue(s) | n/a — user-requested fix after visual review |
| Task state | Done |
| Baseline SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Clarify that the inline entry is SQL to execute and remove inert page-navigation controls when the result fits on one page. |
| Out of scope | Changing query/filter semantics, database rows, or unrelated existing workspace edits. |

### 2. Progress checkpoint

- Current HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4`; implementation remains uncommitted. Source blob SHA: `table_data_toolbar_surface_view.rs` `7154c0f56273b08697394ff8047d43bbe47cbe8b`; `table_data_pagination_view.rs` `b9670cef901ef793e6c304ed4b8867156f345fa7`.
- Completed acceptance rows: `[x] SQL label, SELECT hint, visible Run button, and Enter shortcut`, `[x] hide page navigation when no previous/next page exists`, `[x] focused navigation-visibility tests`, `[x] inspect three viewport captures`, `[x] submit SQL via Run and Enter and verify returned rows in SQLite`.
- Remaining acceptance rows: `[ ] PostgreSQL live execution` (outside this SQLite runtime pass).
- Findings / risks: startup logged missing replacement-glyph warnings (`◻` / `?`), but the interface rendered and read-only queries completed.
- Tests already run: `cargo test -p db-pro-ui table_data_pagination_view::tests --lib` → 2 passed / 0 failed / 0 ignored, exit 0.
- Dependency / blocker changes: none.

### 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | No implementation commit; the two source blob SHA values are recorded above, based on HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4`. |
| Commit list | none |
| File / surface inventory | `table_data_toolbar_surface_view.rs` — persistent SQL label, clearer SELECT hint, visible Run action; `table_data_pagination_view.rs` — one-page navigation visibility and focused tests; plan CHECKLIST/FINDINGS/VERIFICATION — scope and evidence status. |
| Acceptance mapping | SQL draft clarity → toolbar; no inert single-page arrows → pagination condition + 2 unit tests. |
| Commands and counts | `cargo fmt --all` → exit 0; `git diff --check` → exit 0; `cargo check -p db-pro-ui --lib` → exit 0; focused pagination test → 2/0/0, exit 0; `cargo build --release --locked -p db-pro-native` → exit 0. |
| CI run IDs / status | not run |
| Known limitations | PostgreSQL runtime and loading/error/empty captures were not part of this focused verification. Physical captures: 1280×800 logical → 2560×1600; 1440×900 target → 2880×1676 (host-capped to 838 logical px); 1920×1080 target → 3840×1676 (host-capped to 838 logical px). |
| Migrations / config implications | none |
| Out-of-scope changes | Existing staged/unstaged work unrelated to this follow-up was not modified intentionally. |

### 4. Review outcome

n/a — implementer handoff; no independent review performed.

### 5. Research / audit handoff

n/a — focused UI fix.

### 6. Tổng kết bằng tiếng Việt

Đã làm rõ ô SQL bằng nhãn, gợi ý SELECT và nút Run; footer ẩn điều hướng khi chỉ có một trang. Hai test phân trang, UI check và release build đều đạt. Đã đo ba viewport và chạy truy vấn đọc SQLite bằng cả nút Run lẫn Enter; mỗi lần grid chỉ hiện đúng một hàng. PostgreSQL và các trạng thái loading/error/empty chưa được kiểm tra.

## 12. DBeaver-style WHERE condition and Add Row follow-up

### 1. Claim

| Field | Value |
|---|---|
| Agent identity | codex · native UI polish lane |
| Issue(s) | n/a — user clarified the expected table-filter interaction |
| Task state | Done |
| Baseline SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` (implementation remains uncommitted) |
| Branch / PR | `main` / no PR |
| Scope interpretation | Display a fixed `SELECT * FROM <quoted schema>.<quoted table> WHERE` prefix, edit only the condition, and retain Add Row at the toolbar start. |
| Out of scope | General SQL editor behavior, write operations, and unrelated existing workspace edits. |

### 2. Progress checkpoint

- Current HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4`. Working-tree source blob SHA values: `table_data_toolbar_surface_view.rs` `416ccdefa8f66a19939b5be7d80f7d44a04506dc`; `table_data_view.rs` `b52979b81983c2706d8b0dd2a8c13c665df21ba2`; `table_data_query_state.rs` `4ffc3ac41a8c55506e8589fa649df1cc44a1d5fe`; `table_data_mutation_toolbar_view.rs` `68e81eb278c2a9dadd61c1b5cc6b558ec909d006`; `table_data_pagination_view.rs` `b9670cef901ef793e6c304ed4b8867156f345fa7`.
- Completed acceptance rows: `[x] fixed quoted schema/table prefix and condition-only input`, `[x] Run and Enter assemble the predicate with that prefix`, `[x] Add Row retained as the first toolbar control`, `[x] identifier and pagination unit tests`, `[x] three viewport captures`, `[x] Run and Enter interaction verified against SQLite`.
- Remaining acceptance rows: `[ ] live PostgreSQL query` and `[ ] loading/error/empty capture states`.
- Findings / risks: capture-driver images use deterministic PostgreSQL fixture state; only the live interaction evidence used the SQLite sample database. Add Row is disabled for read-only inline query results and is available on mutable table data.
- Tests already run: condition-prefix test → 1 passed; pagination tests → 2 passed; `cargo fmt --all -- --check`, `git diff --check`, and `cargo build --release --locked -p db-pro-native --features capture` passed.
- Dependency / blocker changes: none.

### 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | No implementation commit; baseline HEAD is `710ba002612c6d71ef2605f99f2a49743b51c3a4`; changed source blob hashes are recorded above. |
| Commit list | none |
| File / surface inventory | `table_data_toolbar_surface_view.rs` — fixed prefix, condition input and query assembly; `table_data_view.rs` — schema/table prefix construction; `table_data_query_state.rs` — condition draft state; `table_data_mutation_toolbar_view.rs` — persistent Add Row control; `table_data_pagination_view.rs` — one-page controls; `evidence/data-grid-condition-light-*.png` — measured captures. |
| Acceptance mapping | DBeaver-style condition-only entry and Add Row visibility → toolbar; qualified identifiers → quoting helper + unit test; runtime result → native SQLite UI. |
| Commands and counts | `cargo fmt --all -- --check` → exit 0; `git diff --check` → exit 0; condition-prefix test → 1/0; pagination tests → 2/0; release capture-feature build → exit 0. |
| CI run IDs / status | not run |
| Known limitations | PostgreSQL live execution and loading/error/empty screenshots remain pending. Captures: 1280×800 logical → 2560×1600; 1440×900 target → 2880×1676 (host-capped to 838 logical px); 1920×1080 target → 3840×1676 (host-capped to 838 logical px). |
| Migrations / config implications | none |
| Out-of-scope changes | Existing staged/unstaged work unrelated to this follow-up was not intentionally modified. |

### 4. Review outcome

n/a — implementer handoff; no independent review performed.

### 5. Research / audit handoff

n/a — focused UI correction.

### 6. Tổng kết bằng tiếng Việt

Thanh lọc giờ hiện sẵn `SELECT * FROM "schema"."table" WHERE`, ô nhập chỉ nhận điều kiện; Run và Enter đều ghép câu truy vấn rồi trả kết quả vào grid. Add Row ở đầu thanh và bị khóa nếu đang xem kết quả chỉ đọc. Đã chụp ba kích thước và chạy thử cả hai cách gửi điều kiện trên SQLite; PostgreSQL và các trạng thái loading/error/empty còn chờ.

## 13. WHERE badge and Add Row follow-up

### 1. Claim

| Field | Value |
|---|---|
| Agent identity | codex · native UI correctness lane |
| Issue(s) | n/a — user reported long SQL prefix and Add Row not working |
| Task state | Done |
| Baseline SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Hide the visible qualified SQL prefix behind a compact WHERE badge and restore Add Row for a filtered writable table result. |
| Out of scope | Committing an inserted sample row, PostgreSQL live verification, and unrelated pre-existing worktree changes. |

### 2. Progress checkpoint

- Current HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4`; source blob IDs: `table_data_toolbar_surface_view.rs` `e8dda4c6f471ef2ff728fc593e5bfc305460e841`; `table_data_view.rs` `9ebcd0770650ea7c24b9deff1c3da0caacf2a5ca`.
- Completed acceptance rows: `[x]` hide displayed prefix and show WHERE badge; `[x]` enable Add Row for a writable filtered table; `[x]` retain write-access and in-flight-query checks; `[x]` verify live SQLite filtering and insert form; `[x]` capture target viewport sizes.
- Remaining acceptance rows: live PostgreSQL interaction and loading/error/empty-state evidence remain pending.
- Findings / risks: P2 toolbar lock incorrectly depended on `inline_query_result`; corrected without changing the grid's independent cell-edit guard. No P0/P1 found in scope.
- Tests already run: pre-fix focused test failed 0/1 (exit 101); post-fix UI tests 2 passed / 0 failed / 0 ignored (exit 0); fmt check passed; release capture-feature build passed.
- Dependency / blocker changes: none.

### 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Source claims identify baseline HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` and the two modified source blob IDs above; no commit created. |
| Commit list | none |
| File / surface inventory | `crates/ui/src/table_data_toolbar_surface_view.rs` — compact WHERE badge; `crates/ui/src/table_data_view.rs` — toolbar capability and regression tests; `docs/plans/active/core-component-style-system/{CHECKLIST,FINDINGS,VERIFICATION,AGENT_EVIDENCE}.md` — task evidence; `evidence/data-grid-where-badge-light-*.png` — viewport captures. |
| Acceptance mapping | Compact badge → toolbar source + captures; filtered Add Row → capability helper regression test + live SQLite Insert Row form. |
| Commands and counts | Focused toolbar-capability test pre-fix: 0/1, exit 101; `cargo fmt --all -- --check && cargo test -p db-pro-ui table_data_view::tests --lib`: 2/0, exit 0; `cargo build --release --locked -p db-pro-native --features capture`: exit 0. |
| CI run IDs / status | not run |
| Known limitations | PostgreSQL not runtime-verified; no persistent row insert was made; loading/error/empty states remain pending; 1440/1920 capture height is host-capped. |
| Migrations / config implications | none |
| Out-of-scope changes | Existing staged and unstaged work was preserved. |

### 4. Review outcome

n/a — implementer handoff; no independent review performed.

### 5. Research / audit handoff

n/a — focused UI correctness fix; no external sources used.

### 6. Tổng kết bằng tiếng Việt

Đã ẩn câu SQL dài, chỉ còn badge `WHERE`; đã sửa nguyên nhân Add Row bị khóa nhầm khi lọc bảng. Test tái hiện đỏ trước sửa và xanh sau sửa; SQLite trả đúng Bob, nút Add Row mở form thêm dòng và đã đóng mà không ghi dữ liệu. PostgreSQL chưa được kiểm tra.

## 14. Preserve inline WHERE query after staged save

### 1. Claim

| Field | Value |
|---|---|
| Agent identity | codex · native UI correctness lane |
| Issue(s) | n/a — user reported filtered table returning all rows after insert/save |
| Task state | Done |
| Baseline SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Preserve the last successfully executed inline table SQL and use it to refresh the view after staged writes. |
| Out of scope | Repeating a live database write, broader pagination behavior, and unrelated existing workspace changes. |

### 2. Progress checkpoint

- Current HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4`. Working-tree source blob IDs: `table_data_query_state.rs` `59b69eebaa9936226017b42c0ffc45ee64ea8f29`; `table_editor_view.rs` `fcb02fb218c2314fa27446f456133d16cc10504d`; `events_query_dispatch.rs` `237c8ba0b4020ddffcabd7b5738307eb952d57df`; `query_result_events.rs` `09b241274eb0ec572e42e027cd9b95076b601fea`; `runtime_event_handlers.rs` `392748a2fb85af9f3db3a2335eac53e5e7d8bfb7`; `app_tests.rs` `c7493a1668bd87e9a2de25eda6986c0b7cc6674f`.
- Completed acceptance rows: `[x]` store the last successful inline SQL separately from the draft; `[x]` re-run it after save and manual refresh; `[x]` clear state when switching tables; `[x]` regression coverage.
- Remaining acceptance rows: live PostgreSQL and write-after-filter UI operation were not rerun; loading/error/empty UI states remain pending from the broader plan.
- Findings / risks: P2 — prior save-completion path reloaded the base table without its inline WHERE query. No P0/P1 found in this scope.
- Tests already run: regression test failed before refresh routing (0/1, exit 101), passed after (1/0); query-state tests 5/0/0; fmt check passed; release capture-feature build passed; diff check passed.
- Dependency / blocker changes: none.

### 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Source claims identify baseline HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` and the working-tree blob IDs above; no commit created. |
| Commit list | none |
| File / surface inventory | `table_data_query_state.rs` — active/pending SQL lifecycle; `events_query_dispatch.rs`, `query_result_events.rs`, `runtime_event_handlers.rs` — record successful query and abandon failed/cancelled query; `table_editor_view.rs` — refresh dispatch; `app_tests.rs` — save-completion command regression; plan evidence files. |
| Acceptance mapping | Filter retained after Save → `staged_save_completion_reruns_the_active_inline_sql`; stale/failure/reset handling → query-state lifecycle test. |
| Commands and counts | Red `cargo test -p db-pro-ui staged_save_completion_reruns_the_active_inline_sql --lib`: 0/1, exit 101; green same test: 1/0, exit 0; `cargo test -p db-pro-ui table_data_query_state::tests --lib`: 5/0/0; `cargo fmt --all -- --check`; `git diff --check`; `cargo build --release --locked -p db-pro-native --features capture`: all exit 0. |
| CI run IDs / status | not run |
| Known limitations | Test verifies the emitted runtime query command but does not repeat the SQLite write; PostgreSQL runtime and loading/error/empty states are unverified. |
| Migrations / config implications | none |
| Out-of-scope changes | Existing staged and unstaged workspace work was preserved. |

### 4. Review outcome

n/a — implementer handoff; no independent review performed.

### 5. Research / audit handoff

n/a — source and regression test only; no external sources used.

### 6. Tổng kết bằng tiếng Việt

Sau Save, grid giờ chạy lại đúng câu SQL lọc đã hoàn tất thành công thay vì tải toàn bộ bảng. Nội dung đang gõ dở không bị coi nhầm là filter đã chạy; query lỗi/hủy giữ filter thành công gần nhất, còn chuyển bảng thì xóa filter. Regression test đỏ trước khi nối refresh và xanh sau sửa; native release build đạt. Chưa ghi thêm dữ liệu vào database khi kiểm tra.

## 15. Data Grid condition autocomplete

### 1. Claim

| Field | Value |
|---|---|
| Agent identity | codex · native UI interaction lane |
| Issue(s) | n/a — user reported poor condition input, missing current-table column suggestions, and rough interaction |
| Task state | Done |
| Baseline SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Refine the inline WHERE editor so completion opens only at a column-name position, stays scoped to the selected table, inserts readable identifiers, and remains visually anchored without shifting the grid. |
| Out of scope | Query parser/editor redesign, database execution changes, and unrelated existing worktree edits. |

### 2. Progress checkpoint

- Current HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4`; modified source blobs: `table_data_toolbar_surface_view.rs` `d4a4d9f042219fc26c8cdb8c0e5be474130d7b3a`, `table_data_query_state.rs` `6402fa5ed5cb1ea096aefe465a447f5700dba61d`, `table_data_view.rs` `b0104c49d35a3ffe770dfa82a9704bb26adcdf8b`, `workspace_actions.rs` `2ed4ecd3c67a8c12463e3fc33fb4261e74fd25e7`, `capture.rs` `fa50e07bdb8c7a6e1226766be6e7e28eba0a47b7`.
- Completed acceptance rows: `[x]` no popup on an empty focused field; `[x]` current-table, case-insensitive prefix suggestions only at column positions; `[x]` suppress value/string-literal suggestions; `[x]` Up/Down and Enter/Tab navigation plus pointer selection; `[x]` safe identifiers such as `email` insert unquoted and unsafe/keyword names use quoting; `[x]` compact popup overlays the grid without moving it; `[x]` focused tests, clippy, release build, and inspected captures at all three viewport targets.
- Remaining acceptance rows: `[ ]` live keyboard/pointer selection in the running app and live SQLite/PostgreSQL provider behavior for this completion pass.
- Findings / risks: P2 — this pass proves the suggestion rendering at three viewport targets but does not prove interactive acceptance; no P0/P1 found in focused source/tests.
- Tests already run: `cargo test --locked -p db-pro-ui table_data_toolbar_surface_view::tests --lib` → 6 passed / 0 failed / 0 ignored; `cargo clippy --locked -p db-pro-ui --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `git diff --check`, and `cargo build --release --locked -p db-pro-native --features capture` passed (exit 0).
- Dependency / blocker changes: none.

### 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Source blob IDs above identify the uncommitted implementation against baseline HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4`; no commit created. |
| Commit list | none |
| File / surface inventory | `table_data_toolbar_surface_view.rs` — input frame, conditional suggestions, compact popup, insertion rules and tests; `table_data_query_state.rs` — draft and completion selection/dismiss state; `table_data_view.rs` — current table columns passed to toolbar; `workspace_actions.rs` + `capture.rs` — deterministic focused capture fixture; `VERIFICATION.md` + evidence PNGs. |
| Acceptance mapping | Table-scoped names and prefix matching → current `UiTableInfo.columns` and focused tests; focused empty/value/literal suppression and token/caret behavior → helper tests; identifier quoting → helper test; popup layout/focus at requested viewports → inspected native captures. |
| Commands and counts | `cargo test --locked -p db-pro-ui table_data_toolbar_surface_view::tests --lib`: 6/0/0; `cargo clippy --locked -p db-pro-ui --all-targets -- -D warnings`: exit 0; `cargo fmt --all -- --check`, `git diff --check`, and `cargo build --release --locked -p db-pro-native --features capture`: exit 0. |
| CI run IDs / status | not run |
| Known limitations | The 1440×900 and 1920×1080 capture targets are limited by the host to 838 logical pixels high (physical images 2880×1676 and 3840×1676). Captures show the deterministic PostgreSQL UI fixture, not a live provider. Keyboard/pointer acceptance and provider execution remain unverified in this pass. |
| Migrations / config implications | none |
| Out-of-scope changes | Existing staged and unstaged changes were preserved; no commit or push. |

### 4. Review outcome

n/a — implementer handoff; no independent review performed.

### 5. Research / audit handoff

n/a — no external sources used.

### 6. Tổng kết bằng tiếng Việt

Ô WHERE chỉ mở gợi ý khi nhập ở vị trí tên cột, lọc theo prefix của bảng hiện tại, không gợi ý khi focus ô trống hoặc nhập giá trị; tên an toàn như `email` không bị quote. Popup gọn, nổi trên grid và đã được capture/kiểm tra ở ba kích thước; 6 test, clippy, fmt, diff và release build đều đạt. Chưa xác minh chọn gợi ý bằng phím/chuột trực tiếp hoặc truy vấn trên PostgreSQL/SQLite.

## WHERE input geometry correction — 2026-10-06
- Claim / scope: fixed shaded WHERE prefix inside the input, immediate predicate entry, common 28px control height and separated Run on the same row. Task state: Review; no PR or commit created.
- Exact source identity: baseline HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted toolbar patch. Source file: `crates/ui/src/table_data_toolbar_surface_view.rs`; supporting records: CHECKLIST, FINDINGS, VERIFICATION and three `where-input-alignment-*` captures in this plan.
- Gates: fmt, diff check, UI library check, native release/capture build exit 0; focused toolbar tests 6 passed / 0 failed / 0 ignored. Initial compile failed on module qualification, then corrected. Full workspace/clippy/CI skipped. See VERIFICATION for exact commands.
- Self-review: ACCEPT WITH P2; no introduced P0/P1 found. Inherited pending loading/error/empty-result and interactive/provider verification remain P2 follow-ups. This is not independent approval or feature completion.
- Native captures inspected at all three requested widths; host caps 1440/1920 heights to 838 logical pixels. No live PostgreSQL/SQLite claims; database execution unchanged.
- Research / audit: n/a. Migrations/config: none.
- Tổng kết bằng tiếng Việt: Đã sửa prefix WHERE, vị trí nhập, chiều cao và khoảng cách Run; chụp kiểm tra cả ba chiều rộng. Test toolbar 6/6 và native build đạt. Chưa kiểm tra lại tương tác trực tiếp hoặc loading/error/empty-result; chưa commit/push.

## Empty WHERE execution follow-up — 2026-10-06
- Claim: make empty Run/Enter execute unfiltered SELECT; task state Review; main, no PR/commit.
- Source identity: baseline `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted `table_data_toolbar_surface_view.rs` and `table_data_view.rs` patch. SQL prefix no longer includes WHERE; predicate generation adds it only when nonempty. Existing guarded inline command path retained.
- Implementation handoff: focused toolbar tests 7/0/0, fmt/diff/native capture-release build exit 0; commands and capture listed in VERIFICATION. Workspace gates/CI skipped. No migration/config change.
- Self-review: ACCEPT WITH P2; introduced P0/P1 findings 0/0. Inherited live PostgreSQL/SQLite and broader UI verification pending; no independent approval claim.
- Research/audit: n/a.
- Tổng kết bằng tiếng Việt: Để trống WHERE giờ bấm Run hoặc Enter sẽ chạy SELECT không có điều kiện; Run luôn bật. 7 test đạt, native build đạt và ảnh ô trống đã kiểm tra. Chưa thử lại trực tiếp với hai provider; chưa commit/push.
