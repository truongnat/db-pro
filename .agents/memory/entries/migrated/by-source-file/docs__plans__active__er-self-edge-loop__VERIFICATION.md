---
schema_version: 1
id: "MEM-SRC-88bd6460eaf457e7"
title: "Source mirror: docs/plans/active/er-self-edge-loop/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/er-self-edge-loop/VERIFICATION.md"
source_sha256: "88bd6460eaf457e77938545e3ecf8e58cb7349f7e01c175e1d4f6f337b255f67"
mirror_body_sha256: "88bd6460eaf457e77938545e3ecf8e58cb7349f7e01c175e1d4f6f337b255f67"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
