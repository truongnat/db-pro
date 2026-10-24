---
schema_version: 1
id: "MEM-PLAN-native-app-branding-3e54ac142a3a"
title: "Plan archive: Native App Branding"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, native-app-branding]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "3e54ac142a3a742966f14de069e5062ec12571e4d072b13714d86aac95b269ed"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/native-app-branding/PLAN.md"
  - "docs/plans/active/native-app-branding/FINDINGS.md"
  - "docs/plans/active/native-app-branding/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/native-app-branding/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Native App Branding

## State
- **State:** IMPLEMENTING
- **Branch:** `main`
- **Baseline SHA:** `dc87da1d`
- **Surface:** `crates/native-app` window icon and platform packaging assets.

## Goal
Replace the default native window icon with the supplied DB Pro logo and provide platform-ready icon formats for macOS, Windows and Linux without changing application behavior.

## Scope
- Embed the canonical 256×256 RGBA PNG in the native eframe viewport.
- Ship `.icns`, `.ico`, Linux PNG and `.desktop` metadata beside the native package.
- Add a decode/shape regression test for the embedded icon.
- Build and launch the native app after the change.

## Non-goals
- No installer generator is introduced; the repository currently ships the native binary.
- No changes to database/runtime behavior.
- No logo redraw or color editing beyond producing platform formats from the supplied source.

## Platform matrix
| Platform | Runtime/packaging path | Status |
|---|---|---|
| macOS | `db-pro.icns` + embedded PNG | source/runtime wiring implemented; bundle integration pending packaging pipeline |
| Windows | `db-pro.ico` + embedded PNG | source/runtime wiring implemented; installer integration pending packaging pipeline |
| Linux | `db-pro.png` + `.desktop` + embedded PNG | source/runtime wiring implemented; distro install integration pending packaging pipeline |

## Acceptance criteria
- The native viewport receives the logo icon through `ViewportBuilder::with_icon`.
- Icon asset decodes as square RGBA data at 256×256.
- Native tests, fmt, workspace checks/clippy/tests and release build pass.
- App launches after rebuild; runtime log is captured.

## Tổng kết bằng tiếng Việt
Đã thay icon mặc định của eframe bằng logo DB Pro mới, chuẩn bị PNG/ICO/ICNS/desktop metadata cho macOS, Windows và Linux. Phần installer/bundle pipeline chưa có trong repo nên chỉ ghi nhận asset readiness.


## Source: `FINDINGS.md`

# Native App Branding — Findings

## Baseline
- The native eframe viewport had no application icon configured, so eframe's default icon was used.
- The workspace has no installer/bundle generator or existing icon asset directory.
- The supplied logo is a 1254×1254 RGBA PNG; a 256×256 canonical copy is appropriate for egui/winit window icons.

## Decisions
- Use the supplied image unchanged as the source of truth.
- Embed the PNG in the native binary because eframe/winit can use it consistently across macOS, Windows and Linux window managers.
- Keep `.icns`, `.ico`, Linux PNG and `.desktop` assets ready for a future packaging pipeline.

## Risks / limitations
- An installer or OS bundle may require additional manifest wiring outside this repository's current native-binary-only setup.
- No runtime screenshot can prove OS taskbar/bundle metadata on every platform from this macOS session.

## Tổng kết bằng tiếng Việt
Baseline không có icon wiring nên eframe dùng icon mặc định. Đã chọn PNG 256×256 làm asset canonical nhúng runtime và tạo thêm định dạng platform; giới hạn còn lại là installer/bundle pipeline chưa tồn tại.


## Source: `VERIFICATION.md`

# Native App Branding — Verification

## Current checkpoint
- Baseline: `dc87da1d`.
- Supplied logo copied and resized to `crates/native-app/assets/db-pro-logo.png` (256×256 RGBA).
- Platform assets generated and validated: `db-pro.icns`, `db-pro.ico`, Linux `db-pro.png` and `db-pro.desktop`.
- Native integration: `ViewportBuilder::with_icon` embeds the PNG for macOS, Windows and Linux window/taskbar surfaces.
- `cargo fmt --all -- --check`: PASS.
- `cargo test -p db-pro-native`: PASS (22 passed; 9 evidence-manifest tests also passed).
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace --quiet`: PASS; 914 UI tests and 22 native tests passed, expected provider/SSH fixture cases ignored.
- `cargo build --release --locked -p db-pro-native`: PASS.
- Clean-code scan: PASS (14 pass, 2 warning categories, 0 fail); warnings are inherited theme numeric casts and long theme methods.
- Runtime launch: PASS; rebuilt app is running as background job `bash-84`, and SQLite demo connection/schema introspection completed.
- Packaging limitation: no installer/bundle generator exists in this workspace, so `.icns`/`.ico`/desktop metadata are asset-ready rather than installer-integrated.

## Tổng kết bằng tiếng Việt
Logo mới đã được nhúng vào native window và chuẩn bị đủ PNG/ICO/ICNS/Linux desktop asset. Các gate đều PASS, app release mới đang chạy; phần installer/bundle thật sự còn chờ packaging pipeline của repo.

