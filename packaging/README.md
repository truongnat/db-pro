# DB Pro application branding

The source logo is `crates/native-app/assets/db-pro-logo.png`.

| Platform | Asset | Runtime/packaging use |
|---|---|---|
| macOS | `crates/native-app/assets/db-pro.icns` | Native bundle icon metadata |
| Windows | `crates/native-app/assets/db-pro.ico` | Executable/installer icon metadata |
| Linux | `packaging/linux/db-pro.png` + `db-pro.desktop` | Desktop entry and app launcher |
| Native window | `db-pro-logo.png` | Embedded through `egui::ViewportBuilder::with_icon` on all supported native backends |

The repository currently ships the native binary rather than an installer generator. The platform
bundle assets are kept ready for the packaging pipeline while the embedded PNG covers the running
window/taskbar icon.
