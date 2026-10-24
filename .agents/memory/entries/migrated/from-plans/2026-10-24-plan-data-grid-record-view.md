---
schema_version: 1
id: "MEM-PLAN-data-grid-record-view-4c825caa44dd"
title: "Plan archive: Data Grid Record View"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, data-grid-record-view]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "4c825caa44dda8f20a9d72f069ed3ed187d7d069ab8da410546b53c874afa39b"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/data-grid-record-view/PLAN.md"
  - "docs/plans/active/data-grid-record-view/FINDINGS.md"
  - "docs/plans/active/data-grid-record-view/VERIFICATION.md"
  - "docs/plans/active/data-grid-record-view/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/data-grid-record-view/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Data Grid Record View

State: RUNTIME_VERIFY · implementation authorized 2026-10-06; source and automated verification complete, exact-height/live-provider evidence pending.

User scope: research delivered, then implementation authorized; click a Record field label/value to copy its corresponding text. Reference: supplied Data Grid screenshot; “db-eavar” interpreted as DBeaver.

Implemented behavior: a narrow vertical rail immediately left of the `#` gutter, with Record anchored at its bottom. Selecting one row and pressing Record replaces the grid with that row's fields. Grid returns to the same result and selection.

Implemented scope: Table → Data, viewing/copying one loaded record in a shared Field/Value Table. No new SQL, provider adapter, Query tab, editing workflow, multi-record layout, or automatic database Save. Existing Save/Discard semantics remain authoritative.

Architecture: native egui in `crates/ui`; reuse TableDataState selection, UiQueryResult, staged_cell_value. Keep presentation mode distinct from the existing right-side inspector. Mount the rail outside grid scroll areas. See FINDINGS.md for source evidence and acceptance details.

Acceptance: exactly one selected row, complete values, correct NULL/empty distinction, same staged values as Grid, stable selection on Grid/Record switches, safe result replacement, visible rail at all required sizes/states. Runtime evidence is required before feature completion.


## Source: `FINDINGS.md`

# Research: dải nút dọc và Single Record View

Ngày nghiên cứu: 2026-10-06. Source baseline: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`, branch `main`; source files inspected had no local modifications. Phần §1–8 là research snapshot trước triển khai; cập nhật implementation ở §9.

## 1. Kết luận

Đề xuất thêm dải điều khiển rộng khoảng 36–40 logical px, ngay bên trái cột số thứ tự `#`. Đây là UI chrome, không phải một cột dữ liệu; không tham gia sort, resize, copy hay export. Nút Record nằm ở đáy dải, có tooltip và trạng thái active. Có nút Grid để quay lại.

Chọn một row → Record → toàn bộ vùng bảng chuyển thành danh sách `Field | Value` của đúng row đó. Không mở tab Query hay dialog mới. Không giữ grid bên cạnh như inspector hiện tại. Độ rộng là đề xuất, cần xác nhận qua capture native.

Ví dụ với row 4 trong ảnh người dùng:

| Field | Value |
|---|---|
| id · INTEGER · PK | 4485 |
| first_name · TEXT | Alex |
| last_name · TEXT | Morgan |
| email · TEXT | user_1278@example.com |
| created_at · DATETIME | Hiển thị đầy đủ giá trị từ result; ảnh đang cắt nên không suy đoán phần bị khuất |

Type/PK là metadata phụ cùng ô Field, tránh thêm một cột Type riêng khi không cần. Header nhỏ: `Record · row 4`. Số row phải dùng offset và vị trí đang hiển thị, không dùng raw array index.

## 2. DBeaver thực sự làm gì

- Record view chuyển tên cột thành các hàng và hiển thị giá trị của record hiện tại; chọn cell rồi chuyển Record. Có điều hướng record. [Data view and format](https://dbeaver.com/docs/dbeaver/Data-View-and-Format/).
- Data Editor có left sidebar chứa Grid/Record và các presentation khác; Save/Cancel nằm ở toolbar dưới. [Data Editor](https://dbeaver.com/docs/dbeaver/Data-Editor/).
- Hai trang chưa thống nhất vị trí Record: trang Data Editor ghi left sidebar, trang Data view and format ghi bottom toolbar. Chưa chạy DBeaver để đối chiếu phiên bản; không khẳng định layout pixel hiện tại. Nút neo đáy rail là đề xuất theo yêu cầu người dùng.
- DBeaver có hỗ trợ layout nhiều record, nhưng phạm vi đề xuất DB Pro chỉ là một record.

## 3. Source evidence DB Pro

Mọi đường dẫn/line dưới đây thuộc SHA baseline ở đầu báo cáo.

| Bằng chứng | Hiện trạng và hệ quả |
|---|---|
| `crates/ui/src/result_grid_view.rs:58-74` | Table Data không render result toolbar; toolbar khác có nút Record. Giải thích vì sao ảnh không có nút này. |
| `crates/ui/src/result_grid_toolbar_view.rs:129-138` | Record toggle `record_inspector_open`, tooltip mô tả inspector panel. |
| `crates/ui/src/result_grid_view.rs:74-105` | Inspector chia vùng ngang với grid, bề rộng tối đa 320px/45%. Đây là xem song song, chưa đáp ứng single-record presentation. |
| `crates/ui/src/result_grid_edit.rs:255-289` | Inspector lấy selected_row/selected_cell và row từ UiQueryResult; có thể reuse selection và Inspect action. |
| `crates/ui/src/result_grid_record_surface_view.rs:24-85` | Inspector giới hạn scroll 220px, rút gọn giá trị hơn 80 ký tự, đọc raw row; chưa đủ cho full Record view. |
| `crates/ui/src/result_grid_cell.rs:32-33` | Grid ưu tiên staged_cell_value; Record mới phải dùng cùng effective value để tránh hiển thị khác dữ liệu chưa Save. |
| `crates/ui/src/table_data_state.rs:353-405` | Row/cell selection đã có; selected_rows hỗ trợ nhiều row. Không tạo selection state thứ hai. |
| `crates/ui/src/result_grid_body_view.rs:52-83` | Scroll ngang bao header/body, row virtualization nằm trong scroll dọc. Rail phải nằm ngoài cả hai để luôn cố định. |
| `crates/ui/src/table_data_view.rs:44-104` | Table Data có toolbar/grid/footer và placeholder path riêng. Rail cần có vị trí ổn định cả loading/error/empty. |
| `crates/ui/src/table_events.rs:174-184`, `crates/ui/src/query_result_events.rs:27-40` | Load result có đường reset selection; Table load giữ selection khi staged changes tồn tại. Không được coi index cũ luôn là cùng record sau reload. |
| `docs/architecture/system-overview.md:5-14` | UI native egui, command/event bridge; presentation switch có thể chỉ xử lý local result. |

Đã kiểm tra plan Core Component Style System: không tìm thấy scope dành riêng cho single-record presentation. Chưa kiểm tra remote PR inventory; phải kiểm tra trước khi triển khai/publish nếu cần tránh trùng work.

## 4. Hợp đồng tương tác đề xuất

1. Grid mặc định. Chọn row qua gutter hoặc cell đều hợp lệ; cần đúng một selected row. Khi chưa chọn hoặc chọn nhiều row, Record disabled kèm lý do; không tự chọn một record tùy ý.
2. Record hiển thị đúng row trong result hiện có; không phát LoadTableData mới chỉ để đổi presentation.
3. Quay Grid giữ row/cell selection, column widths, sort/filter và scroll. State dùng chung; không clone toàn bộ result vào Record state.
4. Record bản đầu để xem/copy/Inspect, không thêm inline edit. Giá trị hiển thị lấy staged override như Grid, giữ NULL khác chuỗi rỗng; long text wrap hoặc mở Inspect để xem hết, không cắt im lặng.
5. Nếu đang gõ cell khi bấm Record, dùng đường validation/staging hiện có trước khi chuyển; lỗi validation giữ Grid để người dùng sửa. Không gọi Save hay gửi mutation do chuyển mode. Cần kiểm tra implementation của staging trước khi viết patch.
6. Loading/error/empty và result replacement: không hiển thị row cũ từ index cũ. Nếu selection hết hợp lệ thì quay Grid; chỉ giữ Record khi xác minh được identity của record trong result mới.
7. Hiển thị row number theo vị trí visible sau sort/filter cộng page offset. Không dùng raw zero-based index của inspector hiện tại.
8. Chưa thêm Prev/Next giữa record hay shortcut Tab trong scope đầu; đây là khả năng DBeaver có, không phải yêu cầu hiện tại. Tab vẫn dùng cho focus accessibility.
9. Footer Save/Discard/Refresh và pagination giữ flow hiện tại. Rail có accessible label, tooltip, focus và active state; semantic colors từ DbProTheme.

## 5. Phương án và phạm vi sửa

| Phương án | Đánh giá |
|---|---|
| Chỉ expose nút inspector hiện có | Ít code nhưng vẫn giữ grid và chỉ có preview; không đạt yêu cầu. |
| Dialog record | Làm được nhưng mất tương tác chuyển mode tại grid; không phù hợp yêu cầu dải dọc. |
| Rail + Grid/Record presentation dùng chung state | Khuyến nghị; đúng hành vi và reuse dữ liệu/selection/value formatting hiện có. |

Điểm sửa dự kiến: composition `table_data_surface_view.rs`/`table_data_view.rs`, presentation state và routing trong `result_grid_view.rs`, record renderer hiện có hoặc renderer tách đúng vai trò; reuse staged_cell_value và cell inspector. Không biến `record_inspector_open` thành hai nghĩa. Chưa chốt số file hoặc tạo abstraction mới trước khi implementation review.

Severity: yêu cầu presentation thiếu là P2 UX. Các điểm raw/staged mismatch và selection sau reload là rủi ro cần chặn trong implementation; chưa chứng minh wrong database mutation nên không gán P1 cho pattern chưa kiểm chứng.

## 6. Provider và proof boundary

| Provider | Đề xuất support | Source/automated | Live UI/provider |
|---|---|---|---|
| SQLite | Có: đọc result đã load, không cần PK để xem | Source path chung; implementation chưa có | Pending |
| PostgreSQL | Có: đọc result đã load, không cần PK để xem | Source path chung; implementation chưa có | Pending |

Tests/build chưa chạy vì chưa thay đổi code. Ảnh người dùng là visual reference, không chứng minh native interaction của feature mới. Sau triển khai cần kiểm tra sort/filter/offset, một/nhiều/không selected row, giá trị dài/NULL/JSON/BLOB, staged changes và result replacement. Capture native ở 1280×800, 1440×900, 1920×1080, normal/loading/error/empty. Provider evidence phải riêng cho SQLite/PostgreSQL.

## 7. Learning pass

Bài học reusable: Record presentation và side inspector là hai vai trò khác nhau; chuyển presentation phải dùng cùng selection và effective staged value như Grid. Ghi tại report dự án này; không ghi global memory vì chưa có yêu cầu cập nhật memory.

## 8. Tổng kết bằng tiếng Việt

Đã xác minh DBeaver và code DB Pro. Đề xuất rail bên trái cột `#`, Record neo đáy và thay vùng grid bằng một record. Có nền tảng selection/inspector để reuse nhưng không thể chỉ bật inspector hiện có. Scope này sau đó được owner cho triển khai; xem cập nhật §9 và VERIFICATION.md.


## 9. Implementation and owner simplification

Baseline SHA: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`; implementation is an uncommitted main patch. Exact patch and artifact hashes are recorded in evidence/manifest.json.

- Fixed vertical rail beside the index column; Record at the bottom, enabled for one selected row.
- Owner requested a simple shared Table. Final Record uses exactly two columns, Field and Value. No custom heading, type sublabel, or Inspect column. Earlier research recommendations above are historical, superseded by this instruction.
- Click field name copies the name; click value copies the full effective value, including staged overrides. NULL copies NULL; empty text reports an intentional no-op because the existing egui-winit clipboard bridge ignores empty text.
- Mode switch stages a valid active edit locally; an invalid editor stays open. It sends no SQL and does not save changes. Grid selection is preserved; hidden or reloaded selections exit Record.
- Both PostgreSQL and SQLite share this presentation over UiQueryResult. Native captures use deterministic fixtures; live provider verification remains pending.
- Full-height rail uses horizontal_top; a normal horizontal container initially clipped content. Reuse Table rather than custom compound record rows for this simple field/value presentation.
- Focused headless click tests establish egui clipboard output, not physical OS clipboard behavior.
- Two workspace PostgreSQL connection-form tests fail on both the patch and exported baseline: new_postgresql_connection_defaults_to_tls_require and explicit_disable_selection_is_preserved_on_submit. Baseline run: 182 pass / 2 fail.
- Native perf invocation also has an inherited libtest --quick incompatibility; quick scan runs its supported checks. No performance improvement claimed.

Learning pass: reusable local lessons recorded here. Global memory not modified, as no memory write was authorized.

## 10. Footer balance follow-up

Owner supplied active Save/Discard/Refresh screenshot. Baseline SHA remains `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`, plus uncommitted implementation patch. Refresh used a legacy 24px button; Save/Discard used shared Sm buttons (28px) with different icon/text metrics. Refresh now uses the same shared Sm tokens; Discard uses Secondary and Save remains primary. Footer action spacing is SPACE_XS (4px). Action dispatch, enabled conditions and database mutation policies are unchanged. Added only a deterministic changed-data capture state. No new tests for this reversible styling change; existing behavior tests retained. Reusable lesson: toolbar siblings should use the same component/size rather than mixing legacy and shared buttons. Global memory not modified.


## Source: `VERIFICATION.md`

# Verification

Baseline/current HEAD: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`, main; new implementation is uncommitted. Source patch and native artifact hashes: evidence/manifest.json.

Final presentation: shared Table, exactly Field and Value columns. Native fixture screenshots are in evidence/. Native capture is real eframe framebuffer output; no live provider or OS clipboard claim.

Validation logs are stored in evidence/. Six focused tests cover copying field/value, staged edits, NULL/empty, single selection and invalid editor, filtered selection, result reset, and full-height content. Workspace tests previously ran with 1656 pass / 2 fail / 41 ignored; both failures independently reproduced at baseline (182 pass / 2 fail), so they are inherited. Final simplification: six focused tests passed again; fmt, workspace check/clippy, clean-code ratchet, locked native release and capture builds passed. Exact command logs are recorded in final-* logs.

Prior checks: fmt, workspace check/clippy, standard locked release build, capture build passed. Quick performance scan passed four executed checks; native benchmark attempt failed due unchanged libtest --quick handling. Benchmark/provider measurements were skipped; no performance claim.

Captures requested 1280×800, 1440×900, 1920×1080. Host caps larger window heights at 838 logical pixels; actual image dimensions in manifest. Normal/Grid/loading/error/empty fixture states captured. Exact 900/1080 height acceptance remains pending.

Remaining: live PostgreSQL and SQLite interaction, physical clipboard/accessibility verification, exact larger viewport heights, independent review, inherited workspace test fixes. Feature remains RUNTIME_VERIFY; source inspection and fixtures do not satisfy all runtime gates. No commit, PR, CI run, merge, or push.

Footer follow-up: cargo fmt --all -- --check, workspace check/clippy, locked release and capture builds passed. No new tests or full workspace rerun for styling-only changes. Six fresh native captures cover changed and unchanged data at three widths; same host height cap applies. Direct rustfmt check of include-based toolbar surface reports existing unrelated formatting; left untouched to avoid scope expansion. Latest footer/source artifact hashes in manifest, per-image binary provenance retained.

## Audit evidence update — 2026-10-13

`docs/plans/_audit-2026-10-13/03-data-grid-table-editor.md` verdict: **EVIDENCE_GAP**
(code fully verified; live/interactive evidence still missing). Confirmed at HEAD:
40px fixed rail left of `#`, Record bottom-anchored, exactly-one-row gate, staged
effective values, click-to-copy via `OutputCommand::CopyText`; all 7 record tests
pass (`cargo test -p db-pro-ui --lib record`).

**Stale-fact correction:** the "2 inherited workspace failures" above are now fixed —
`new_postgresql_connection_defaults_to_tls_require` and
`explicit_disable_selection_is_preserved_on_submit` (`app_tests.rs:372,467`) both
**PASS at HEAD**; the 1656/992 "2 fail" totals are stale on that fact.

Fresh captures: `docs/plans/_audit-2026-10-13/captures/table-record_{1280x800,1440x900,1920x1080}.png` —
fixed left rail + Field/Value single-record layout confirmed visually (larger heights
capped at 838). Remaining per earlier record: live PG/SQLite record flow, physical
OS clipboard + accessibility pass, exact 900/1080 heights, independent review. State
stays RUNTIME_VERIFY.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Data Grid Record View

## 1. Claim

Agent: Codex root. Task state: Review. Feature state: RUNTIME_VERIFY. Baseline SHA: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`. Branch main; PR n/a. Implementation is an uncommitted patch, exact hashes in evidence/manifest.json.

Scope: fixed mode rail, single selected record rendered with shared two-column Table, click field/value to copy. User superseded the initial custom layout with this simple table. No new database command, migration, dependency, or persisted contract.

## 2. Progress checkpoint

Implemented selection guards, staged effective values, edit validation, reload/filter reset, and deterministic capture fixture. Six focused tests and native captures collected; details and proof limits in VERIFICATION.md. Remaining runtime gates: both live providers, OS clipboard/accessibility, exact taller viewports and independent review.

## 3. Implementation handoff / review request

Changed native UI surface, editing/clipboard state, runtime result reset, capture driver and tests. Reused shared Table and existing value/selection paths. Invalid edits cannot switch mode; switching/copying does not submit database mutations. PostgreSQL and SQLite presentation supported through common UiQueryResult; live provider evidence pending independently.

Validation: see evidence logs and VERIFICATION.md. Existing .DS_Store preserved. No commit or PR. Research sources retained in FINDINGS.md; final product behavior supersedes historical design recommendations.

## 4. Review outcome

Self-review only; independent approval n/a. Verdict: ACCEPT WITH P2 for source handoff, pending runtime acceptance. Introduced P0/P1: 0 identified. Inherited validation issues: two baseline connection-form test failures, plus native benchmark runner --quick incompatibility. No unrelated fixes. CI skipped; exact gates and proof gaps documented, not reported as passing.

## 5. Research / audit handoff

Official DBeaver research and baseline source evidence: FINDINGS.md. Initial placement discrepancy resolved using owner's bottom Record button instruction. Owner's later feedback resolved presentation to shared Field/Value Table. Source date: 2026-10-06. Remote duplicate PR check skipped. Lessons recorded locally in FINDINGS.md; global memory unchanged.

Footer follow-up: standardized Save/Discard/Refresh to shared Sm size and SPACE_XS gaps; Discard Secondary. No action/enable semantics changed. fmt/check/clippy and release builds passed; active/inactive native captures collected at three widths, exact heights still capped. Source remains baseline plus uncommitted patch; manifest identifies final patch and per-image binaries. Styling-only change: no additional tests. Lessons in FINDINGS §10.

## 6. Tổng kết (Vietnamese summary)

Record đã đổi sang Table dùng chung chỉ hai cột Field / Value. Click tên trường copy tên, click giá trị copy nội dung đầy đủ và phản ánh staged edit. Rail giữ cố định cạnh index. Có headless tests và native fixture screenshots; chưa xác nhận clipboard hệ điều hành, database thật, hoặc đủ chiều cao 900/1080. Giữ trạng thái RUNTIME_VERIFY.

Footer đã chỉnh ba nút cùng size, icon/type và khoảng cách đều; Save chính, Discard nền nhẹ. Có ảnh native ở trạng thái có/không có staged change.

