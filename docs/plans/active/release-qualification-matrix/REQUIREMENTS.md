# Requirements traceability index

Source baseline: `dd2bf5e4090e6d509d34c0c8c58ca95015d91640`. Plan state **PLANNING**. Đây là yêu cầu cần chứng minh, **không phải tuyên bố các tính năng đã PASS**. Canonical chi tiết duy nhất là [CASES.json](CASES.json); bảng dưới không lặp test steps.

| Requirement | Phạm vi / acceptance | Case IDs |
|---|---|---|
| REQ-ENV — Candidate/fixture/desktop identity | Mỗi evidence phải gắn đúng source, binary, OS, provider và sandbox; không thao tác user state. | ENV-01, ENV-02, ENV-03 |
| REQ-REL — Build/package/distribution | Pinned gates, archive/license/trust và CI provenance đúng candidate; không dùng historical green để qualify. | REL-01, REL-02, REL-03, REL-04 |
| REQ-LIF — Launch/quit/restart/upgrade | Native release hoạt động, đóng bình thường, bảo toàn state theo contract và recovery upgrade thực. | LIF-01, LIF-02, LIF-03, LIF-04, LIF-05, LIF-06 |
| REQ-CON — Connection/validation/transport | Tạo/test/connect/edit/disconnect đúng endpoint; validation/auth/TLS/SSH thất bại minh bạch và recover được. | CON-01, CON-02, CON-03, CON-04, CON-05, CON-06, CON-07, CON-08, CON-09 |
| REQ-SEC — Secrets/security evidence | Default OS-store lifecycle và redaction được chứng minh; dependency advisories có disposition. | SEC-01, SEC-02, SEC-03, SEC-04 |
| REQ-EXP — Explorer/catalog identity | Danh mục/object/refresh/quoted names thuộc đúng connection/schema; không click-through. | EXP-01, EXP-02, EXP-03, EXP-04, EXP-05 |
| REQ-QRY — SQL editor/run/context | Actual buffer, parameters, limit, explain và document target tạo đúng kết quả, không stale context. | QRY-01, QRY-02, QRY-03, QRY-04, QRY-05, QRY-06, QRY-07, QRY-08, QRY-09, QRY-10, QRY-11 |
| REQ-EXE — Execution timing/state/transaction | Cancel/timeout/late events đúng request; commit/rollback có independent DB oracle. | EXE-01, EXE-02, EXE-03, EXE-04 |
| REQ-SAFE — Backend safety | Read-only/destructive consent được enforce thực, canceled action không thay DB. | SAFE-01, SAFE-02 |
| REQ-GRID — Read/copy/value fidelity | Sort/filter/page/copy đúng identity; precision/null/truncation không mất nghĩa. | GRID-01, GRID-02, GRID-03, GRID-04 |
| REQ-EDIT — Mutation/conflict/retry | Staging chưa ghi DB; Save đúng row/full key; atomic failure, conflict và targeted retry không mất edit khác. | EDIT-01, EDIT-02, EDIT-03, EDIT-04, EDIT-05, EDIT-06, EDIT-07, EDIT-08 |
| REQ-SCM — Schema inspect/migrate/routine | Catalog chính xác; DDL supported theo provider, preview/confirmation/target fingerprint đúng. | SCM-01, SCM-02, SCM-03, SCM-04, SCM-05 |
| REQ-ER — ER runtime/layout/performance | Graph/relationship/navigation đúng schema; interaction thật, persistence và measurements không suy từ ảnh. | ER-01, ER-02, ER-03 |
| REQ-XPT — Export/diff/transfer | File/SQL chính xác, overwrite consent scoped, real DB copy atomicity theo contract; không tráo harness. | XPT-01, XPT-02, XPT-03, XPT-04, XPT-05, XPT-06, XPT-07 |
| REQ-BAK — Backup/restore/recovery | Backup độc lập đọc được, restore đúng disposable target, WAL giữ committed data; PG partial failure trung thực. | BAK-01, BAK-02, BAK-03, BAK-04 |
| REQ-NAV — Tabs/palette/history | Tab được activate nhìn/đóng được, manual scroll giữ nguyên, palette/history không stale target. | NAV-01, NAV-02 |
| REQ-FIL — Saved queries/files/workspace | Tên/nội dung/save/reload đúng file; path confinement và dirty state không silent loss. | FIL-01, FIL-02, FIL-03 |
| REQ-SET — Settings/locale/session | Có đường thoát; preferences/locales/keybinding/session có effect và persistence thực. | SET-01, SET-02, SET-03 |
| REQ-TOOLS — Tools/admin/tasks/builder | Không chỉ mở trang: sessions/roles/RLS/tasks/builder/ER-design thực thi đúng hoặc gated rõ. | TOOL-01, TOOL-02, TOOL-03, TOOL-04, TOOL-05, TOOL-06, TOOL-07, TOOL-08, TOOL-09 |
| REQ-AI — Agent preview/provider/safety | No-key và consent/egress rõ; real provider evidence riêng, patch/cancel/target đúng scope. | AI-01, AI-02, AI-03, AI-04 |
| REQ-UX — Keyboard/pointer/accessibility | Thao tác keyboard/pointer thật, focus/labels/menu hữu dụng; không biến AXPress thành keyboard proof. | A11Y-01, A11Y-02, A11Y-03 |
| REQ-PERF — Measured responsiveness | Có raw sample method/hardware/budgets; không tuyên bố smooth/60fps/idle thấp từ screenshot. | PERF-01 |
| REQ-CAP — Unsupported feature contract | Positive unsupported có approved N_A; negative action phải có clear reason và no unsupported SQL. | CAP-01 |
| REQ-VIS — Rendered geometry/state/theme | Mỗi named surface/state đúng light/dark và requested dimensions; clamp/ảnh sai surface không PASS. | VIS-01, VIS-02, VIS-03, VIS-04, VIS-05, VIS-06, VIS-07, VIS-08, VIS-09, VIS-10, VIS-11, VIS-12, VIS-13 |

## Release-risk traceability

| Risk từ assessment | Cases bắt buộc / disposition |
|---|---|
| R-01 native close crash | LIF-02, LIF-03, LIF-04; giữ historical failure riêng, chạy lại chỉ trên new candidate/được giao regression run |
| R-02 notices/license wording | REL-02, SEC-03 |
| R-03 RC tag/version mismatch | REL-04 |
| R-04 stale release claims/evidence | ENV-01, REL-04, CAP-01, aggregate manifest identity |
| R-05 harness guard-return false confidence | REL-01: parsed executed/ignored/guard-skip counts + provider-specific evidence; CON-09 và CON-07 cần fixture thật |
| UI persistence/storage risk discovered while planning | LIF-04/06, SET-01/03, FIL-01, ER-02; inspect actual eframe feature/storage lifecycle, không giả DB_PRO_DATA_DIR là GUI sandbox |

## Matrix expansion — một hàng kết quả cho một instance

Functional: mỗi case × platform × provider × declared state × named subvariant; canonical Dark/1280×800. Visual: thêm Light/Dark × 1280×800,1440×900,1920×1080. Không nhân mọi functional case thêm6 visual cells. Thông tin axes nằm trong CASES.json; instance ID deterministic từ các axes, không chứa attempt.

Các action sequence/boundary values trong một definition là required assertions: worker phải ghi đủ `oracle-01..N` và subchecks cho từng input được nêu. Nếu chỉ làm99 mà chưa làm100/101 thì QRY-05 không PASS. Nếu cần dispatch nhỏ hơn, lead tách assignment theo check IDs nhưng chỉ đóng parent instance khi đủ toàn bộ check; không tự giảm acceptance.

Mọi candidate mới freeze lại expected instances/capabilities tại run intake. Tất cả current cases initial `NOT_TESTED`; old audit kết quả không seed PASS. Worker không xóa provider/platform/state khỏi denominator vì host/fixture thiếu.

## Source basis, không coi source là runtime

Tất cả paths sau @ `dd2bf5e4090e6d509d34c0c8c58ca95015d91640`:
- `crates/core/src/domain/capabilities/capability_presets.rs:15–345` — provider capability boundaries.
- `crates/app/src/table/table_editor_context.rs:10–69`, `table_conflict_dialog_surface.rs:28–126`, `table_mutation_actions.rs:409–567` — native editability/Save/conflicts/retry.
- `crates/app/src/query_workbench/query_parameters_view.rs:13–94`, `query_context_view.rs:147–185`, `query_session.rs:49–69` — actual parameter panel/cap/context.
- `crates/app/src/schema/schema_compare_view.rs:382–461` — Apply migration SQL vs data sync preview-only.
- `crates/app/src/tools/transfer_activity_surface_view.rs:390–526`, `saved_tasks_surface_view.rs:57–302` — real Copy tables vs harness; actual task actions.
- `crates/app/src/settings/settings_backup_view.rs:22–170` — actual Backup / Restore UI.
- `crates/app/src/visual_query_builder/visual_query_builder_surface_view.rs:24–329` — Apply SQL → editor, Run separately.
- `crates/native-app/src/main.rs:585–607`, `crates/app/src/app/app_lifecycle.rs:44–138` — eframe storage wiring, not persistence proof.
- `fixtures/{sqlite,postgres,mysql,mssql,smoke}/` — existing seed/verify assets; planned extensions defined in FIXTURES.md.

## Scope exception register

Core providers/platforms mặc định giữ cả SQLite/PostgreSQL/MySQL/SQL Server và macOS ARM64/Windows x64/Linux x64 vì current selectable drivers/distribution matrix. Giảm scope chỉ qua owner decision trong run manifest, không qua worker N_A. Mỗi exception gồm requirement/instances, reason, owner, accepted risk, channel, expiry và negative-gating evidence.

AI live external calls, positive TLS/SSH/default OS-keyring denial và admin flows có prerequisite/authorization riêng; thiếu prerequisite là BLOCKED cho advertised scope. Preview label không tự miễn kiểm tra safety. Auto-updater/installer formats ngoài portable contract, mobile/web/Tauri archived không thuộc matrix; không tạo giả PASS cho chúng.
