# UI Runtime Verification Plan

**SHA**: bae4765e chore: ignore local agent tooling state directories (.omh, .hermes, .kilo, .jules)
**Crate**: `crates/ui` + `crates/infrastructure`
**Date**: 2026-09-16
**Status**: DRAFT — pending runtime environment availability

---

## Goal

Thu thập runtime evidence cho audit trước đó:
- **Performance**: fps steady-state, scroll latency, sort latency, selection lookup rebuild cost khi column order thay đổi, editor highlight time khi file dài, grid frame time.
- **UX**: keyboard navigation cho tree/sidebar/gallery, focus ring visibility, resize divider hit target feel.

---

## Build constraint (ghi nhận)

- **Build `db-pro-native` thất bại** trong môi trường CLI này (exit code 1).
- Lỗi: bindgen khi compile 의존성 `sqlite`=0.32.0 (C header binding cần Rust C binder / cmake / pkg-config).
- Egui app cần display/GPU để chạy — terminal CLI không có GUI.

**Plan alternative**:
- Fix build environment: cài Rust `bindgen` crate, `cmake`, `pkg-config`, `libsqlite3-dev` (hoặc macOS `sqlite3` native).
- Hoặc build trên machine khác có GUI và đo lường ở那里.
- Hoặc accept code-path audit đã có (benchmarks trong repo, code analysis) và không làm runtime verification thực tế.

---

## Nếu build được — benchmark plan

### 1. Result grid performance (dựa trên `crates/ui/benches/result_grid_benchmarks.rs`)

| Test case | Mục tiêu | Method |
|-----------|----------|--------|
| 1M rows projection (không sort/filter) | Xác nhận `filtered_sorted_indexes` < budget | Chạy benchmark, so sánh với code-path expectation |
| 200k rows x 4 columns — sort temporal column | Xác nhận `GridProjectionCache` giảm rebuild cost | So sánh build time trước/sau cache |
| Selection lookup rebuild (200k rows x 4 columns) | Xác nhận < 36.5ms (baseline issue) | Đo `GridSelectionLookup::new` time mỗi frame khi column order đổi |
| Scroll window materialization (100 visible rows) | Xác nhận fps ổn định khi scroll | Đo frame time khi scroll wheel |

### 2. Editor performance

| Test case | Mục tiêu | Method |
|-----------|----------|--------|
| SQL file dài (>10k lines) highlight | Xác nhận không lag | Đo highlight time per visible range |
| Nhiều diagnostic squiggle + search highlight + IME | Xác nhận draw time ổn định | Đo frame time khi có nhiều overlay |

### 3. UI update path

| Test case | Mục tiêu | Method |
|-----------|----------|--------|
| Query result stream-in | Xác nhận không rebuild projection/selection lookup mỗi chunk | Audit code path, đo nếu có instrumentation |
| Toolbar allocation mỗi frame | Xác nhận không phải bottleneck | Profile allocation rate, đo nếu có instrumentation |

### 4. UX verification

| Test case | Mục tiêu |
|-----------|----------|
| Tree keyboard navigation | Xác nhận có focus ring, tab order, accessible name |
| Sidebar gallery keyboard | Xác nhận có focus ring policy |
| Resize divider hit target | Xác nhận feel (6px vs 8-10px) |

---

## Instrumentation để thêm (nếu làm runtime verification)

- Thêm `egui::Context::request_repaint()` timer hoặc `Instant::now()` measurements quanh:
  - `GridProjectionCache` rebuild
  - `GridSelectionLookup::new`
  - `draw_grid_body` total time
  - `SqlEditor::render` highlight time
- Log ra UI overlay hoặc console để đọc real-time.

---

## Visual acceptance (liên kết)

Xem riêng: `UI_VISUAL_ACCEPTANCE_PLAN_2026-09-16.md`

---

## Tóm tắt bằng tiếng Việt

Đã tạo runtime verification plan. Build db-pro-native thất bại trong môi trường CLI này (lỗi bindgen sqlite, thiếu GUI). Plan ghi nhận constraint và đề xuất alternative: fix build environment, hoặc build trên máy có GUI, hoặc accept code-path audit đã có. Nếu build được, benchmark plan bao gồm result grid (projection, selection lookup, scroll), editor (highlight dài, overlay), UI update path, UX (keyboard tree/sidebar/gallery, resize hit target). Cần thêm instrumentation để đo lường real-time.

---

## Checklist

- [ ] Fix build environment (bindgen, cmake, pkg-config, sqlite3)
- [ ] Build `cargo build --release --locked -p db-pro-native`
- [ ] Chạy db-pro-native trên desktop thật / remote desktop / CI GPU runner
- [ ] Thu thập fps, scroll latency, sort latency, selection lookup rebuild cost
- [ ] Thu thập editor highlight time khi file dài
- [ ] Thu thập UX keyboard navigation evidence
- [ ] Tạo screenshot visual acceptance evidence (xem plan riêng)

---

*Hermes Agent session — runtime verification plan drafted (build constraint noted).*
