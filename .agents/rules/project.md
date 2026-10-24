# Bối cảnh riêng của project

> DB Pro — desktop Database IDE / Agent IDE. Cập nhật khi stack hoặc lệnh chuẩn thay đổi.

## Project

- **Tên / mục tiêu:** DB Pro — IDE cơ sở dữ liệu native, hỗ trợ truy vấn, schema, chỉnh sửa dữ liệu có kiểm soát.
- **Loại ứng dụng:** Desktop native (Rust + egui/eframe), không WebView/Node cho UI sản phẩm.
- **Thành phần chính:** `crates/core`, `crates/infrastructure`, `crates/runtime`, `crates/ui`, `crates/native-app` (`db-pro-native`).
- **Nguồn tài liệu kiến trúc:** `AGENTS.md`, `RULES.md`, `docs/10-egui-native-migration-plan.md`, `docs/plans/`.

## Stack và phiên bản

- **Runtime/framework:** Rust workspace; UI `eframe` + `egui`; host Tauri 2 (`crates/tauri-app`) chỉ transitional.
- **Ngôn ngữ/toolchain:** Rust; fmt/clippy theo `AGENTS.md`; release qualification matrix yêu cầu pin compiler (xem memory `environment` / plan RQM).
- **Database:** PostgreSQL, SQLite (provider/runtime trong workspace).
- **Dependency quan trọng:** Khóa bản qua `Cargo.lock`; build release dùng `--locked` khi qualify release.

## Kiến trúc và quy ước

- **Entry point:** `crates/native-app` → binary `db-pro-native`; shell/state trong `crates/ui`.
- **Luồng dữ liệu:** Runtime bootstrap/services → UI `AppState`/reducer/task bridge; không gọi egui widget thô từ view (xem `RULES.md`).
- **Theme:** `Primitive → Semantic → Component`; view dùng `DbProTheme` / token, không hardcode màu/spacing lẻ.
- **Test layout:** Unit test **không** inline trong file nguồn — `#[path = "tests/<stem>_tests.rs"]` cạnh module (chi tiết `AGENTS.md`).
- **Archive:** `_archive/frontend/` và bench React — tham chiếu only, không build trong CI.

## Lệnh (đối chiếu CI / AGENTS.md)

- **Cài đặt / deps:** `cargo fetch` (workspace).
- **Chạy dev:** `cargo run -p db-pro-native` (hoặc flow dev team).
- **Unit / workspace test:** `cargo test --workspace`
- **Lint/format:** `cargo +nightly fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`
- **Type check:** `cargo check --workspace`
- **Build release (sản phẩm):** `cargo build --release --locked -p db-pro-native`
- **Perf scan:** `bash .skills/perf-audit/scripts/perf-scan.sh`
- **Clean code scan:** `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`
- **Tìm memory:** `.agents/scripts/search-memory.sh "<từ khóa>"`

## Ràng buộc

- **Tương thích:** Native UI là nguồn sự thật; không khôi phục frontend React làm fallback.
- **Bảo mật:** Không commit secret; database safety theo `AGENTS.md` (không suy diễn hành vi provider từ test provider khác).
- **Hiệu năng:** Virtualized grid, ER diagram, query path — budgets trong perf-audit skill.
- **Release:** Release Qualification Matrix (`docs/plans/active/release-qualification-matrix/`) — evidence schema, compiler pin, Orca GUI lanes.

## Ngôn ngữ tài liệu

- **Mặc định:** Tiếng Việt cho memory `.agents/`, plan nội bộ, bàn giao agent (theo `rules/language.md`).
- **Ngoại lệ:** `AGENTS.md`, `RULES.md`, comment code và identifier kỹ thuật giữ tiếng Anh như hiện tại.

## Bảng thuật ngữ nghiệp vụ

| Thuật ngữ | Ý nghĩa | Nguồn chuẩn |
|---|---|---|
| Component-first | View chỉ dùng widget trong `crates/ui/src/components` | `RULES.md` |
| DbProTheme | Facade theme + semantic tokens light/dark | `crates/ui/src/theme/` |
| RQM | Release Qualification Requirements Matrix | `docs/plans/active/release-qualification-matrix/` |
| Analyst-first | Ưu tiên evidence/invariant trước khi đổi hành vi | `AGENTS.md` |
