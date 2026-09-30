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
