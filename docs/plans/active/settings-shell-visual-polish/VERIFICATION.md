# Verification

## Source baseline

- Base SHA: `bd0535fb`.
- Working branch: local `main` per explicit user workflow override.
- Implementation commit: recorded in the follow-up verification commit after
  the source change is committed.

## Runtime evidence

| Requested viewport | Captured framebuffer | Theme | Artifact |
|---|---:|---|---|
| 1280×800 | 2560×1600 (Retina 2×) | Dark | `screenshots/settings-dark-1280x800.png` |
| 1280×800 | 2560×1600 (Retina 2×) | Light | `screenshots/settings-light-1280x800.png` |
| 1440×900 | 2880×1676 (Retina 2×, host-capped height) | Dark | `screenshots/settings-dark-1440x900-capped.png` |
| 1920×1080 | 3840×1676 (Retina 2×, host-capped height) | Dark | `screenshots/settings-dark-1920x1080-capped.png` |

The captures were visually inspected. They show the dedicated Settings surface,
grouped navigation, active General pill, layered background, and existing
General/Diagnostics content. The host capture path produces a 2× Retina
framebuffer and caps logical height at approximately 838px, so the 1440×900
and 1920×1080 artifacts are valid width checks but not exact-height evidence.

## Automated gates

- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS — 928 UI tests passed; all workspace suites
  passed, with only the repository's expected provider/fixture tests ignored.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  PASS — 15 checks passed, 1 inherited size warning, 0 failures. The warning
  is limited to pre-existing long functions in `settings_model.rs`,
  `workspace_actions.rs`, and `workspace_view.rs`.

## Provider matrix

| Provider | Supported behavior changed | Automated evidence | Live/runtime evidence | Capability gate |
|---|---|---|---|---|
| PostgreSQL | No | N/A | N/A | Existing capability behavior unchanged |
| SQLite | No | N/A | N/A | Existing capability behavior unchanged |

## Tổng kết bằng tiếng Việt

Settings đã được tách thành một workspace native riêng với nền phân lớp, rail
navigation theo nhóm và active pill; pipeline action/state cũ được giữ nguyên.
Dark/light capture 1280×800 và các capture width 1440/1920 đã kiểm tra trực
quan. Toàn bộ gate tự động đã chạy đạt; độ cao 1440/1920 bị giới hạn bởi host,
và independent review vẫn chưa thực hiện.
