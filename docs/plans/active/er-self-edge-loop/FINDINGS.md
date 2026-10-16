# #512 Findings

## Evidence (source)
- `crates/app/src/diagram_surface/diagram_view.rs:630-644` (`paint_scene_edges`):
  `source_on_right = src.center.x < tgt.center.x` → self-edge luôn `false` →
  `from` = left edge, `to` = right edge (`!false = true`).
- `:651` `bend_x = (from.x + to.x)/2 + lane` → giữa node → polyline xuyên thân.
- Edge vẽ trước node → đoạn giữa bị che → self-FK vô hình.
- Crow's-foot `:672` `dir = -1` (vì `source_on_right=false`) → apex chĩa vào thân node.
- `crates/ui/src/diagram/model.rs:56-80` (`column_anchor`): `right_side=true` → `world_rect.right()`.

## Failure scenario
Bảng `employees` có `manager_id → employees.id`: hai anchors ở hai cạnh đối diện,
bend ở giữa card; line bị card đè lên → user không thấy relationship. Phổ biến ở
adjacency-list (`categories.parent_id → categories.id`).

## Severity
P2 (theo issue): không mất/sai dữ liệu; ER thiếu self-reference nhưng DB và các edge khác đúng.

## Scope decision
Same-side (phải) polyline loop — theo đúng gợi ý đầu trong issue; không làm arc trên đỉnh.

## Provider impact
n/a — ER render từ introspected schema, provider-agnostic.

## Runtime testability
Geometry helper thuần số học → unit test trực tiếp. Render пиксеl (egui painter)
không test được headless → visual screenshot để owner verify, khai báo thiếu.
