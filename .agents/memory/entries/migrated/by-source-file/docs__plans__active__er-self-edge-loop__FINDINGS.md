---
schema_version: 1
id: "MEM-SRC-a6126b3a7854a87a"
title: "Source mirror: docs/plans/active/er-self-edge-loop/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/er-self-edge-loop/FINDINGS.md"
source_sha256: "a6126b3a7854a87a3fdaf63e9578a8fb6514840df88e70091dc0ca59a5f4d0f1"
mirror_body_sha256: "a6126b3a7854a87a3fdaf63e9578a8fb6514840df88e70091dc0ca59a5f4d0f1"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
