---
schema_version: 1
id: "MEM-SRC-9253fc441cfec0ee"
title: "Source mirror: RULES.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "RULES.md"
source_sha256: "9253fc441cfec0eed28ecfd54c92aad310f72a1dc76f518c64c54cc8f43a84bc"
mirror_body_sha256: "9253fc441cfec0eed28ecfd54c92aad310f72a1dc76f518c64c54cc8f43a84bc"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---

# Mirror: `RULES.md`

# UI Rules — DB Pro

Bắt buộc cho toàn bộ `crates/ui`. Mọi widget mà người dùng thấy/phải tương tác
**MUST** đi qua core components trong `crates/ui/src/components`. Không gọi trực
tiếp widget nguyên bản của egui trong view code.

## 1. Component-first

| Cần | Dùng | KHÔNG dùng |
|---|---|---|
| Button / icon button | `components::button::Button` | `ui.button`, `egui::Button` |
| Checkbox | `components::selection::Checkbox` | `ui.checkbox`, `egui::Checkbox` |
| Switch / toggle | `components::selection::Switch` | `ui.toggle_value`, `egui::widgets::...` |
| Radio | `components::selection::Radio` | `ui.radio`, `egui::RadioButton` |
| Slider | `components::selection::Slider` | `ui.add(egui::Slider)` |
| Single-line input | `components::input::Input` | `ui.text_edit_singleline`, `egui::TextEdit` |
| Password | `components::input::PasswordInput` | `TextEdit::password` |
| Multi-line text | `components::input::Textarea` | `TextEdit::multiline` |
| Select / dropdown | `components::select::Select` | `egui::ComboBox`, `selectable_value` list |
| Tabs / segmented | `components::tabs::*` (`SegmentedTabs`) | `selectable_label` rows tự chế |
| Scroll container | `components::scroll_area::ScrollArea` | `egui::ScrollArea` trong view mới |
| Dialog / modal / sheet | `components::dialog::*` | `egui::Window`, `egui::Area` |
| Card / panel | `components::card::*` | `egui::Frame` với fill/stroke tự chế |
| Badge / status pill | `components::badge::*` | `ui.label` + `Frame` tự chế |
| Alert / notice | `components::alert::*` | label màu + separator tự chế |
| Tooltip / hover card | `components::hover_card::*` | `response.on_hover_text` khi cần card |
| Separator | `components::separator::Separator` | `ui.separator` (chỉ khi separator mặc định đủ) |
| Tree / nav tree | `components::tree::*` | `CollapsingHeader` tự style |
| Form field + label | `components::form::*` | label + widget ghép tay lặp lại |

Quy tắc bao trùm: nếu core component tồn tại cho pattern đó thì MUST dùng nó.
Nếu chưa tồn tại, MUST thêm vào `crates/ui/src/components` rồi dùng — không được
inline một widget tự chế ở view.

## 2. Layout primitives cho phép

Được phép dùng trực tiếp ở mọi nơi (đây là layout, không phải widget):

- `ui.horizontal`, `ui.vertical`, `ui.columns`, `ui.group` (không border tự chế)
- `ui.label` / `RichText` cho text thuần — **text color MUST đi qua token theme**
- `ui.add_space`, `ui.allocate_*` cho spacing/layout
- `ui.painter()` cho canvas custom (ER diagram, charts, editor)

## 3. Theme tokens

- Mọi màu MUST lấy từ `DbProTheme` / `crate::tokens::*`. Cấm literal `Color32::...`
  trong view (trừ `Color32::TRANSPARENT`, `PLACEHOLDER`).
- Không hard-code font size/spacing; dùng `SPACE_*`, `RADIUS_*`, `font_*()`,
  `DbProTheme::ui_*_font`.
- Widget mới trong `components/` MUST nhận `DbProTheme` và render được cả light
  lẫn dark mà không if-else theo theme ở caller.

## 4. Trạng thái

Mọi interactive component MUST render đủ: default / hover / active / focus /
disabled. Disabled dùng `text_disabled`/`surface_*` tương ứng — không để egui
fallback quyết định.

## 5. Ngoại lệ

Chỉ cho phép egui nguyên bản khi:

- Code bên trong chính `components/` (components bọc egui là nhiệm vụ của nó).
- Dev-tools/debug surface trong `dev_tools/` (đánh dấu comment `// dev-only`).
- `egui::ScrollArea` bọc bởi `components::scroll_area` — view không gọi trực tiếp.
- Text inside editor/renderer (custom canvas paint — ngoài scope widget).

Mỗi ngoại lệ khác MUST có comment `// ui-rules: <lý do>` ngay tại dòng đó.

## 6. Kiểm tra

Trước khi merge UI diff:

```bash
grep -rn 'ui\.checkbox\|ui\.button(\|ui\.radio\|ui\.toggle_value\|egui::ComboBox\|egui::Slider\|ui\.text_edit_' crates/ui/src --include='*.rs' | grep -v 'components/\|dev_tools/\|ui-rules:'
```

Output phải rỗng. Thêm pattern mới vào lệnh này khi rule phình ra.
