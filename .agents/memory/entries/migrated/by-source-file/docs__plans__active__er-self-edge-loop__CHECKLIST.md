---
schema_version: 1
id: "MEM-SRC-efe7178d282a5447"
title: "Source mirror: docs/plans/active/er-self-edge-loop/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/er-self-edge-loop/CHECKLIST.md"
source_sha256: "efe7178d282a54473b3c42fbee30e1e1b5802e3f00fcdb2d087ceb8cedfc2fa0"
mirror_body_sha256: "efe7178d282a54473b3c42fbee30e1e1b5802e3f00fcdb2d087ceb8cedfc2fa0"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #512 Checklist

- [x] PLAN/FINDINGS/CHECKLIST/VERIFICATION scaffold
- [x] Repro (fail-before): self-edge bend_x nằm giữa from/to (xuyên node)
- [x] Helper `edge_route_bend_x` + branch anchor/dir trong `paint_scene_edges`
- [x] Unit tests: loop ngoài node, fan-out theo id, edge thường giữ công thức cũ
- [x] Focused tests + fmt + clippy + cc-scan PASS
- [ ] Comment/close issue (chờ owner quyết định)
