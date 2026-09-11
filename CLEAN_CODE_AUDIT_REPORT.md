# 🛡️ BÁO CÁO AUDIT CLEAN CODE & CLEAN ARCHITECTURE
**Dự án:** `db-pro` (Database Client Native / Tauri Rust)
**Thời gian thực hiện:** 2026-09-11
**Phạm vi Audit:** Toàn bộ Workspace (`crates/core`, `crates/infrastructure`, `crates/runtime`, `crates/ui`, `crates/native-app`, `crates/tauri-app`)
**Tiêu chuẩn đánh giá:** Clean Architecture (Hexagonal/Ports & Adapters), SOLID, Clean Code Standards (Naming, Functions, Error Handling, Cohesion & Coupling, Testability).

## 🔁 FOLLOW-UP 2026-09-11

### Native UI redesign follow-up

- Shared native presentation now uses a dark-first token hierarchy, quiet surfaces, borderless tabs, compact activity rail, context-driven Explorer actions, and a focused empty workspace. Theme storage uses `native-redesign-v4`, migrating stale pre-redesign light-mode state to the dark default once while preserving later user choices.
- `agent.rs` responder branches and ER diagram canvas/node selection are split into focused helpers; behavior-preserving UI tests remain green.
- Current native code sizes after the split: `app.rs` 753 lines, `agent_view.rs` 303, `diagram_view.rs` 567, `query_view.rs` 681, `navigation_view.rs` 686, `app_state.rs` 209.
- Verification: workspace check/clippy, native build, UI 52/52 tests and `git diff --check` pass. Clean-code heuristic retains one constructor warning for `app_state::default`; no behavior-neutral abstraction was added solely to satisfy the heuristic.
- Runtime note: the current native binary is running through X11 at 1280×800; full keyboard-only outcome remains pending. No Orca/computer-use path was used.

- (Historical snapshot) `crates/ui/src/app.rs` đã tách phần khởi tạo/persistence state sang `app_state.rs`; composition root khi đó giảm từ **845 xuống 677 dòng**.
- `crates/infrastructure/src/postgres/introspect.rs` đã bỏ `get_mut(...).unwrap()` ở luồng gom composite foreign key, dùng `HashMap::entry(...).or_insert_with(...)`.
- `crates/native-app/src/translate.rs` đã tách event translation khỏi một hàm 193 dòng; mapping schema/connections/query folders/saved queries có helper riêng.
- Tauri command boundary đã chuyển sang `DbProRuntime` và các facade typed (`ConnectionApi`, `QueryApi`, `SchemaApi`, `TableDataApi`, `ExportApi`, `BackupApi`, `UserApi`, `DataDiffApi`, `PostgresApi`); không còn `State<Arc<...Service>>` trong command modules.
- (Historical snapshot) Đã chạy: `cargo test -p db-pro-ui --offline` (**36 passed**), `cargo clippy -p db-pro-ui --offline --all-targets -- -D warnings` (**pass**), `cargo test -p db-pro-infrastructure --offline` (**37 unit + 25 SQLite/integration passed**), isolated PostgreSQL fixture (**10/10 passed**) và clippy infrastructure (**pass**).
- Workspace gate sau facade migration: `cargo fmt --all -- --check`, `cargo test --workspace --offline` và `cargo clippy --workspace --offline --all-targets -- -D warnings` đều PASS; native launch smoke sống 8 giây không output/crash.
- Còn giữ các hàm render dài trong các view egui vì đây là các boundary UI; chỉ tách tiếp khi có thay đổi hành vi hoặc test chứng minh cần thiết.
- Clean-code scan hiện không còn blocking translation-function finding; cảnh báo còn lại là các boundary render egui/Tauri bootstrap và heuristic `unwrap_or_default`, đã ghi rõ để không biến thành refactor không có hành vi.
- PostgreSQL runtime đã được verify trong container fixture tạm (10/10 tests pass); database BSN `127.0.0.1:15433` chỉ được probe read-only. Numeric mapper đã sửa để tôn trọng `dscale` của PostgreSQL và giữ exact decimal text.

---

## 🔁 FOLLOW-UP #2 — 2026-09-11 · Quét lại trên trạng thái đã merge

**Bối cảnh.** Toàn bộ 6 nhánh local (native redesign, sidebar DBeaver/codex, checkpoint WIP `3bd8eef`…) đã được merge trực tiếp vào `truongnat/main`; React frontend đã archive vào `_archive/frontend/`. Phần dưới là kết quả **quét thực tế trên cây đã merge**, thay thế các số liệu lấy từ nhánh redesign ở phần trên.

**Mốc so sánh:** `origin/main` = `bd64ddf6` → `HEAD` = `c9a40a99`, **33 commit**, 580 file, **+6.589 / −1.795** dòng.
Phân bố file thay đổi: `_archive/` 500 · `docs/` 33 · `crates/ui/` 19 · `.skills/` 7 · `plans/` 6 · `crates/native-app/` 2 · `crates/tauri-app/` 2 · `.github/` 2 · còn lại là file gốc.

### ⚠️ Đính chính các số liệu đã lệch

| Mục | Báo cáo cũ (nhánh redesign) | Thực tế sau merge |
| :--- | :--- | :--- |
| Theme mặc định | dark-first, storage `native-redesign-v4` | **light-first**, `THEME_STORAGE_VERSION = "light-first-v1"` (`app_state.rs`) |
| `crates/ui/src/app.rs` | 753 dòng | **825 dòng** |
| `crates/ui/src/query_view.rs` | 676 dòng | **766 dòng** |
| `crates/ui/src/navigation_view.rs` | 682 dòng | **708 dòng** |
| `crates/ui/src/components.rs` | không nêu | **803 dòng** (trước merge: 341) |
| `crates/ui/src/explorer_view.rs` | "tách riêng boundary" | **1.471 dòng** (trước merge: 637) |
| `db-pro-ui` tests | 52/52 | **64/64** |

Lý do lệch: merge ưu tiên nhánh sidebar/DBeaver (light-first + navigator kiểu DBeaver), nên test `stale_theme_storage_resets_to_dark_first_default` đã được đổi thành `..._to_light_first_default`. Đây là **xung đột ngữ nghĩa** mà git không phát hiện được.

### 📊 Kết quả quét (clean-code scan)

| Phạm vi | pass | warn | fail |
| :--- | :---: | :---: | :---: |
| **Diff** (`origin/main...HEAD`, 19 file production) | **14** | 3 | **2** |
| **Baseline** (toàn repo, 120 file production) | 11 | 8 | 0 |

Hai gate chặn còn lại trong scope diff — **cả hai đều là gate kích thước**:

1. **`fn` dài > 100 dòng: 20 hàm** (43 hàm vượt ngưỡng cảnh báo 50 dòng).
2. **File dài: 1 file > 1.200 dòng** — `crates/ui/src/explorer_view.rs` (1.471 dòng). Cảnh báo thêm: `table_editor_view.rs` 991, `app.rs` 825, `components.rs` 803 (> 800).

Cảnh báo (không chặn): `let _ = <fallible>` không comment lý do — **32 chỗ**; `> 15 .clone()` — `events.rs` (25), `explorer_view.rs` (18), `table_editor_view.rs` (27).

### 🧭 Phân loại 20 hàm vượt ngưỡng chặn

Đối chiếu với bản `origin/main` để tách "nợ cũ" khỏi "nợ do merge tạo ra":

**NEW — 7 hàm, do merge đưa vào (100% dòng là mới):**

| Hàm | Vị trí | Dòng |
| :--- | :--- | ---: |
| `draw_dbeaver_table_item` | `explorer_view.rs:720` | 324 |
| `draw_codex_tree_row` | `explorer_view.rs:23` | 148 |
| `draw_dbeaver_connections_tree` | `explorer_view.rs:304` | 145 |
| `draw_dbeaver_schema_objects` | `explorer_view.rs:600` | 118 |
| `draw_dbeaver_functions_folder` | `explorer_view.rs:1159` | 117 |
| `draw_dbeaver_views_folder` | `explorer_view.rs:1046` | 111 |
| `draw_query_actions_menu` | `query_view.rs:248` | 141 |

**MODIFIED — 6 hàm có dòng bị sửa (hàm cũ, bị phình thêm):**
`app_state.rs:88 default` (142, Δ7) · `query_view.rs:4 draw_query` (243, Δ57) · `result_grid_view.rs:4 draw_result_grid` (236, Δ152) · `table_view.rs:4 draw_welcome` (104, Δ84) · `table_view.rs:180 draw_table_metadata_view` (114, Δ21) · `workspace_view.rs:16 draw_workspace_tabs` (176, Δ3).

**UNTOUCHED — 7 hàm, nợ cũ hoàn toàn (0 dòng bị sửa, chỉ nằm trong file có thay đổi):**
`events.rs:4 apply_runtime_events` (401) · `table_editor_view.rs:4 draw_table_data` (231) · `table_editor_view.rs:475 draw_table_ddl` (118) · `table_view.rs:295 draw_table_structure` (163) · `navigation_view.rs:458 draw_history` (140) · `navigation_view.rs:599 draw_settings` (109) · `query_view.rs:549 sql_layouter` (116).

➡️ **Kết luận:** 7 hàm là **nợ mới**, 6 hàm bị phình thêm, 7 hàm là **nợ cũ** chỉ bị "lộ ra" vì nằm cùng file. Nguồn nợ mới tập trung ở **một chỗ duy nhất**: navigator kiểu DBeaver trong `explorer_view.rs` (637 → 1.471 dòng).

### 🐞 Hai bug của chính script scan (đã sửa)

1. **`awk -v` xử lý escape sequence** → regex `pub(\([a-z]+\))?` bị biến thành `pub(([a-z]+))?`, nên mọi `pub(super) fn` / `pub(crate) fn` **không bao giờ khớp**. Script báo **13** hàm > 100 dòng trong khi thực tế là **20**. Sửa: truyền regex qua `ENVIRON["START_RE"]` thay vì `-v`.
2. **`prod_only()` không loại file `_tests.rs`** → `crates/ui/src/app_tests.rs` (1.223 dòng, 50 `#[test]`) bị tính là *production*. Hệ quả: gate "file dài" báo FAIL oan, và phát sinh 26 `unwrap/expect` + 10 `let _ =` báo nhầm. Sửa: thêm `_tests?\.rs$|_bench\.rs$` vào mẫu loại trừ.

Sau 2 fix: file production 20 → **19**; gate file dài 5/2 → **4/1**; tổng kết diff 13/4/2 → **14/3/2**.

### ✅ Cổng kiểm chứng (đã chạy lại trên cây đã merge)

| Gate | Kết quả |
| :--- | :--- |
| `cargo fmt --all -- --check` | ✅ sạch |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ **0 error, 0 warning** |
| `cargo test --workspace --offline` | ✅ **348 passed / 0 failed / 10 ignored** |

Chi tiết test: `db-pro-core` 186 · `db-pro-infrastructure` 72 · `db-pro-ui` **64** · `db-pro-tauri` 21 · `db-pro-runtime` 4 · `db-pro-native` 1. 10 test `ignored` là bộ **PostgreSQL fixture** cần DB sống — chưa chạy trong lần này.

### 📋 Action items bổ sung

| Mức độ | Hạng mục | Vị trí | Đề xuất |
| :---: | :--- | :--- | :--- |
| 🔴 **Cao (Must)** | `explorer_view.rs` 1.471 dòng — vượt ngưỡng chặn | `crates/ui/src/explorer_view.rs` | Tách navigator DBeaver thành `views/explorer/` (mỗi folder/schema/table một module) |
| 🔴 **Cao (Must)** | 6 hàm `draw_dbeaver_*` > 100 dòng | `crates/ui/src/explorer_view.rs` | Trích `draw_dbeaver_table_item` (324 dòng) thành các helper theo hành vi |
| 🟡 **Trung bình** | `components.rs` +462 dòng trong 1 lần merge | `crates/ui/src/components.rs` | Tách theo nhóm widget (`feedback/`, `controls/`, `data/`) |
| 🟡 **Trung bình** | 32 `let _ = <fallible>` không comment | `native-app/main.rs`, `app.rs`, `connection_view.rs`, `events.rs` | Thêm comment lý do + `tracing::debug` (`error-handling.md` §3) |
| 🔵 **Thấp** | `.clone()` cao ở `table_editor_view.rs` (27) | `crates/ui/src/table_editor_view.rs` | Kiểm tra borrow / `Arc` / `Cow` (`functions.md` §10) |
| 🔵 **Thấp** | 7 hàm nợ cũ > 100 dòng | `events.rs`, `table_editor_view.rs`, `navigation_view.rs` | Không bắt buộc trong lần này — chỉ tách khi có thay đổi hành vi |

### 🎯 Đánh giá sau merge

Các tiêu chí 1–4 và 6 ở bảng Executive Summary **giữ nguyên**. Riêng **tiêu chí 5 (Độ phức tạp hàm & Code Smells)** nên hạ từ **8.5 → 7.5/10**: số hàm > 100 dòng tăng từ ~13 (đếm sai) lên **20**, trong đó **7 hàm mới** đến từ navigator DBeaver. Đây là nợ kỹ thuật **có chủ đích và khu trú** (một file, một tính năng), không phải suy giảm kiến trúc — các gate `fmt` / `clippy -D warnings` / test vẫn sạch tuyệt đối.

> **Lưu ý về bằng chứng runtime:** chưa có ảnh chụp UI native ở các độ phân giải 1280×800 / 1440×900 / 1920×1080 và chưa chạy phiên PostgreSQL + SQLite sống. Vì vậy phần UI của lần merge này **chưa thể đánh dấu `COMPLETED`** theo `docs/plans/FEATURE_LIFECYCLE.md` (thiếu bằng chứng mức 4 — UI runtime).

---

## 📊 TỔNG QUAN ĐÁNH GIÁ (EXECUTIVE SUMMARY)

| Tiêu chí | Điểm đánh giá | Trạng thái | Nhận xét |
| :--- | :---: | :---: | :--- |
| **1. Kiến trúc phân tầng (Clean Architecture)** | **9.5/10** | 🟢 Xuất sắc | Phân tách ranh giới Core/Domain/Ports/Infra/UI cực kỳ nghiêm ngặt. Zero leak dependency. |
| **2. Nguyên lý SOLID & Modularity** | **9.0/10** | 🟢 Tốt | SRP và DIP tuân thủ tốt; vừa refactor thành công module UI monolithic (`app.rs`). |
| **3. Quy chuẩn đặt tên & Ngôn ngữ chung (Ubiquitous Language)** | **9.5/10** | 🟢 Xuất sắc | Tên struct, trait, enum mang tính biểu đạt cao theo thuật ngữ CSDL chuẩn. |
| **4. Xử lý lỗi & Độ tin cậy (Error Handling)** | **9.0/10** | 🟢 Tốt | Có hệ thống `DbError` / `ErrorEnvelope` phân lớp, không leak raw driver error. Không có production `panic!`. |
| **5. Độ phức tạp hàm & Code Smells** | **8.5/10** | 🟡 Khá | Còn một số hàm render UI dài (>100 dòng); đây là boundary egui và chưa có thay đổi hành vi buộc phải tách thêm. |
| **6. Chất lượng kiểm thử (Testing & Testability)** | **9.5/10** | 🟢 Xuất sắc | Workspace gates pass; 10/10 PostgreSQL fixture tests pass trong runtime container riêng; clippy pass không warning. |

---

## 🏛️ 1. PHÂN TÍCH KIẾN TRÚC & PHÂN TẦNG (CLEAN ARCHITECTURE)

### 1.1. Luồng phụ thuộc (Dependency Rule)
Dự án áp dụng mô hình **Hexagonal Architecture (Ports & Adapters)** chuẩn mực:
```
[crates/native-app]  [crates/tauri-app]
         \                /
      [crates/runtime]  /
             \        /
         [crates/ui] (giao tiếp qua DTOs / TaskBridge Events)
             |
   [crates/infrastructure] (Adapters: Postgres, SQLite, Keyring, SSH)
             ↓
     [crates/core]
       ├── domain/      (Entities, Value Objects, Error Taxonomy)
       ├── ports/       (Traits: DBConnector, Repositories, SecretStore)
       └── application/ (Use Cases, DDL Builder, SQL Policy, Services)
```

- **Điểm mạnh vượt trội:**
  - `crates/core` độc lập hoàn toàn với framework và thư viện I/O bên ngoài (không phụ thuộc vào `sqlx`, `rusqlite`, `eframe`, `tauri`).
  - `crates/ui` hoàn toàn tách biệt khỏi DB driver, chỉ giao tiếp với backend thông qua DTOs và message passing (`UiCommand`, `UiEvent`, `TaskBridge`). Rất dễ chuyển đổi giữa Native App (`eframe/egui`) và Tauri App.
  - Các Ports (`crates/core/src/ports/`) định nghĩa rõ ràng các Trait cho tầng ngoại vi triển khai (DIP - Dependency Inversion Principle).

---

## 🔍 2. CHI TIẾT CÁC PHÁT HIỆN & ĐÁNH GIÁ THEO TỪNG TIÊU CHÍ

### 2.1. Đặt tên & Ngôn ngữ nghiệp vụ (Naming & Domain Ubiquitous Language)
- **Đánh giá:** 🟢 **Xuất sắc (9.5/10)**
- **Điểm sáng:**
  - Danh từ miền nghiệp vụ rõ ràng: `ConnectionConfig`, `DatabaseCapabilities`, `CellValue`, `SqlPolicy`, `ExecutionState`, `SchemaSummary`.
  - Không có các tên mơ hồ kiểu `Data`, `Info`, `Manager`, `Util` ở tầng domain (trừ các tên chuyên biệt như `UserManager` được đặt theo role nghiệp vụ).
  - Biến và hàm có tiền tố/hậu tố rõ nghĩa: `build_create_table`, `is_terminal`, `redact_sensitive`.

---

### 2.2. Xử lý lỗi & Tính phòng vệ (Error Handling & Fail-Loud)
- **Đánh giá:** 🟢 **Tốt (9.0/10)**
- **Điểm sáng:**
  - Định nghĩa thống nhất `DbError` (sử dụng `thiserror`) với các biến thể lỗi rõ ràng: `ConnectionFailed`, `QueryFailed`, `ConstraintViolation`, `SchemaIntrospectionFailed`, `SerializationError`, `NotFound`.
  - Tầng Domain chuẩn hóa `ErrorEnvelope` để truyền tải lỗi ra frontend/UI với mã lỗi (`code`), tính năng retry (`retryable`), và `operation_id`.
  - **Zero production `panic!`**: Không có `panic!` nào trong production code path (tất cả `panic!` đều nằm trong module `#[cfg(test)]` để assert).

- **Follow-up đã xử lý:** các `.unwrap()` trong SQL classification/quote scanning và HashMap introspection đã được thay bằng nhánh an toàn hoặc `entry` API; numeric wire decoding cũng đã có test exact-scale.

---

### 2.3. Cấu trúc Module & Tách biệt trách nhiệm (SRP & Modularity)
- **Đánh giá:** 🟢 **Tốt (9.0/10)**
- **Điểm sáng:**
  - Vừa qua file monolithic `crates/ui/src/app.rs` (~5,000 dòng) đã được bóc tách thành các view riêng biệt; sau follow-up, phần state khởi tạo nằm trong `app_state.rs` và composition root còn 677 dòng:
  - `agent_view.rs` (296 dòng)
  - `connection_view.rs` (332 dòng)
  - `diagram_view.rs` (539 dòng)
  - `events.rs` (478 dòng)
  - `navigation_view.rs` (682 dòng)
  - `palette_view.rs` (382 dòng)
  - `query_view.rs` (676 dòng)
  - `schema_object_view.rs` (140 dòng)
  - `table_editor_view.rs` (990 dòng)
  - `table_view.rs` (425 dòng)
  - `workspace_view.rs` (200 dòng)
  - `agent_state.rs`, `explorer_view.rs`, `result_grid_view.rs` tách riêng các boundary tương ứng.
  - File gốc `app.rs` hiện 753 dòng, chủ yếu giữ state contract và điều phối frame/UI.

- **Khuyến nghị tổ chức file (Rust Idiomatic Refactoring):**
  - Hiện tại `app.rs` đang dùng directive `#[path = "agent_view.rs"] mod agent_view;`.
  - **Khuyến nghị:** Gom các view vào thư mục `crates/ui/src/views/` (hoặc `crates/ui/src/app/`) và khai báo module chuẩn (`mod views;` hoặc `pub mod views`) thay vì dùng explicit `#[path]`. Điều này giúp cây thư mục `src/` của `crates/ui` gọn gàng và tuân theo quy chuẩn chung của hệ sinh thái Rust.

---

### 2.4. Độ phức tạp hàm & Code Smells
- **Đánh giá:** 🟡 **Khá (8.5/10)**
- **Quan sát:**
  - Một số hàm vẽ UI trong `table_editor_view.rs` và `query_view.rs` có độ dài tương đối lớn (>80 dòng) do tính chất của declarative UI framework (`egui`).
  - **Khuyến nghị:** Tiếp tục trích xuất các sub-component nhỏ (ví dụ: `draw_table_columns_tab`, `draw_table_indexes_tab`, `draw_table_foreign_keys_tab`) thành các helper function độc lập.
  - Các hằng số ma thuật (Magic numbers) đã được định nghĩa tốt dưới dạng hằng số `const` ở đầu `app.rs` (`AGENT_SIDEBAR_COLLAPSE_WIDTH`, `ER_NODE_WIDTH`, `ER_HEADER_HEIGHT`, v.v.). Có thể cân nhắc đóng gói thành `struct LayoutMetrics` hoặc theme token.

---

### 2.5. Chất lượng kiểm thử (Testing & Verification)
- **Đánh giá:** 🟢 **Xuất sắc (9.5/10)**
- **Kết quả thực tế:**
  - `db-pro-core`: **186/186 tests passed** (Bao phủ toàn diện services, DDL, SQL Builder, Safety, Execution, Capabilities, Diagnostics).
  - `db-pro-infrastructure`: **37/37 unit tests passed** + **25 SQLite integration tests passed** + **10/10 PostgreSQL fixture tests passed** trong container tạm.
  - `db-pro-runtime`: **4/4 tests passed**.
  - `db-pro-tauri-lib`: **21/21 tests passed** (Execution lifecycle, Cancellation, Int64 boundary DTOs).
  - `db-pro-ui`: **52/52 tests passed** (State management, Query dispatch, Schema refresh, Theme, Table filters/sorts, Agent provider routing).
  - **Clippy check:** `0 errors, 0 warnings` trên toàn bộ `--all-targets`.

---

## 📋 3. BẢNG TỔNG HỢP HÀNH ĐỘNG CẢI TIẾN (ACTION ITEMS)

| Mức độ | Hạng mục | Vị trí | Giải pháp đề xuất |
| :---: | :--- | :--- | :--- |
| ✅ **Đã xử lý** | Loại bỏ `.unwrap()` trong string matching | `crates/core/src/application/query_service.rs:395` | Đã thay bằng `.chars().next().is_some_and(...)` |
| ✅ **Đã xử lý** | Pattern matching an toàn cho quote parsing | `crates/core/src/application/sql_policy.rs:29` | Đã thay `chars.next().unwrap()` bằng pattern an toàn |
| ✅ **Đã xử lý** | HashMap grouping an toàn trong PostgreSQL introspection | `crates/infrastructure/src/postgres/introspect.rs:418` | Đã dùng `entry(...).or_insert_with(...)` |
| ✅ **Đã xử lý** | PostgreSQL NUMERIC giữ đúng display scale | `crates/infrastructure/src/postgres/query_mapper.rs` | Đọc `dscale` từ binary wire value; không còn thêm zero giả; có unit + live fixture coverage |
| 🟡 **Trung bình (Should)** | Tối ưu hóa cấu trúc thư mục UI views | `crates/ui/src/` | Chuyển 11 file view vào `crates/ui/src/views/` và dùng module hierarchy chuẩn |
| 🟡 **Trung bình (Should)** | Tách nhỏ hàm render trong Table Editor & Query View | `crates/ui/src/table_editor_view.rs`, `query_view.rs` | Tách các khối tab & panel thành các hàm con chuyên biệt |
| 🔵 **Khuyến khích (Info)** | Bổ sung ví dụ Doc-tests | `crates/core/src/domain/` | Thêm doctests cho các Value Objects cốt lõi (`CellValue`, `DatabaseCapabilities`) |

---

## 🎯 KẾT LUẬN

Codebase **`db-pro`** có chất lượng mã nguồn rất cao, tuân thủ xuất sắc các nguyên tắc **Clean Architecture**, **SOLID** và **Clean Code**. Việc phân chia ranh giới giữa Core (Domain/Ports/Application) và Ngoại vi (Infrastructure/UI) được thực hiện triệt để, hệ thống kiểm thử tự động vô cùng vững chắc với hơn 300 test cases phủ kín các kịch bản biên. Các cải tiến đề xuất chủ yếu nhằm nâng cao tính thành ngữ (idiomatic Rust) và độ tinh gọn của mã nguồn.
