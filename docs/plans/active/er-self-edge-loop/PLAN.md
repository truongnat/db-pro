# #512 — Self-referencing FK edge loops outside the node

## Goal
Self-FK (vd. `employees.manager_id → employees.id`) phải vẽ thành vòng loop bên ngoài card thay vì đường ngang xuyên thân node (bị node che mất, user không thấy relationship).

## Scope
- `crates/app/src/diagram_surface/diagram_view.rs::paint_scene_edges` — branch `edge.source == edge.target`:
  - cả hai anchor cùng phía phải (`right_side: true`);
  - `bend_x` nằm ngoài cạnh phải node + fan-out dương theo `edge.id`;
  - crow's-foot `dir = +1` (chĩa ra ngoài, thay vì chĩa vào thân node).
- Tách pure helper `edge_route_bend_x(from_x, to_x, self_edge, edge_id, zoom)` để unit-test được geometry mà không cần egui render.
- Không đổi route của edge thường (giữ nguyên công thức midpoint + lane).

## Non-goals
- Arc loop trên đỉnh node (alternative trong issue) — chọn same-side polyline, ít code hơn, nhất quán với style orthogonal 3-segment hiện tại.
- Auto chọn phía trái/phải theo viewport — luôn loop bên phải cho đơn giản, đúng gợi ý trong issue.

## Architecture
- `ErNode::column_anchor(col, ErAnchorSpec{right_side,..})` (`crates/ui/src/diagram/model.rs:56`): `right_side=true` → `world_rect.right()`.
- `paint_scene_edges` đã có `edge.source/target/id`, `viewport.world_to_screen_pos`, `zoom`.
- Helper thuần số học trên screen coords → test được không cần painter.

## Acceptance
- Self-edge: `bend_x > max(from.x, to.x)` với mọi `edge_id`/`zoom` (route ngoài node).
- Self-edge id khác nhau → `bend_x` khác nhau (fan-out, không đè nhau).
- Edge thường: công thức cũ giữ nguyên (midpoint + lane, kể cả lane âm).
- Crow's-foot của self-edge chĩa +x (ra ngoài).
