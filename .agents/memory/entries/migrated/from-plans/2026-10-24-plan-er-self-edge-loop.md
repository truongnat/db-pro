---
schema_version: 1
id: "MEM-PLAN-er-self-edge-loop-31728b3d3f04"
title: "Plan archive: Er Self Edge Loop"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, er-self-edge-loop]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "31728b3d3f04ffa459ae2112ce9f85d718156841ebb1b053809c9b8be2577d56"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/er-self-edge-loop/PLAN.md"
  - "docs/plans/active/er-self-edge-loop/FINDINGS.md"
  - "docs/plans/active/er-self-edge-loop/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/er-self-edge-loop/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

# #512 Verification (append-only)

## Fail-before (analytic, code tại `55dc35a4` sau #513-push)
- `paint_scene_edges` cũ: self-edge `source_on_right=false` → `from`=left edge,
  `to`=right edge, `bend=(from+to)/2+lane`, `|lane|<=27*zoom` < nửa card
  (`ER_NODE_WIDTH=280`, nửa=140) → bend **luôn** trong `[left,right]`.
- Test `old_route_bend_stays_inside_same_node_512` (copy công thức cũ, card
  100→380, 14 edge id) assert bend nào cũng trong card → PASS, tức bug tồn tại
  với mọi `edge_id`.

## Pass-after (sau fix)
- `cargo test -p db-pro-app --lib self_edge` → **4/4 PASS**:
  `old_route_bend_stays_inside_same_node_512` (record fail-before),
  `self_edge_bend_loops_outside_node_512` (bend > right edge, zoom 0.5/1/1.5 × 14 id),
  `self_edge_bend_fans_out_per_edge_512` (step 8px @zoom1),
  `normal_edge_route_keeps_midpoint_lane_formula_512` (14 id × 3 zoom).
- `cargo test -p db-pro-app --lib diagram` → **22 passed / 0 failed**.
- `cargo +nightly fmt --all -- --check` → exit 0.
- `cargo clippy -p db-pro-app --all-targets` → không warning nào trong `diagram*`.
- `python3 tools/cc-scan.py diagram_view.rs diagram_view_self_edge_tests.rs --fail-on error` → **100/100, error=0** (đã gom 5 params thành `EdgeBendSpec` sau 1 warning ban đầu).
- `cargo check -p db-pro-app` → Finished OK.

## Chưa làm (khai báo trung thực)
- Không có pixel/screenshot evidence của loop sau fix — cần owner verify trên build thật.
- Không chạy full `cargo test --workspace`.
- Issue #512 để OPEN chờ owner quyết định.

