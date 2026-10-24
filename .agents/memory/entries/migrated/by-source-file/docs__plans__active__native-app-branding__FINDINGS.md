---
schema_version: 1
id: "MEM-SRC-2937c6ccb0763d4a"
title: "Source mirror: docs/plans/active/native-app-branding/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/native-app-branding/FINDINGS.md"
source_sha256: "2937c6ccb0763d4a975acd63908499c507318c152adcdf364de055e9e0c4c679"
mirror_body_sha256: "2937c6ccb0763d4a975acd63908499c507318c152adcdf364de055e9e0c4c679"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Native App Branding — Findings

## Baseline
- The native eframe viewport had no application icon configured, so eframe's default icon was used.
- The workspace has no installer/bundle generator or existing icon asset directory.
- The supplied logo is a 1254×1254 RGBA PNG; a 256×256 canonical copy is appropriate for egui/winit window icons.

## Decisions
- Use the supplied image unchanged as the source of truth.
- Embed the PNG in the native binary because eframe/winit can use it consistently across macOS, Windows and Linux window managers.
- Keep `.icns`, `.ico`, Linux PNG and `.desktop` assets ready for a future packaging pipeline.

## Risks / limitations
- An installer or OS bundle may require additional manifest wiring outside this repository's current native-binary-only setup.
- No runtime screenshot can prove OS taskbar/bundle metadata on every platform from this macOS session.

## Tổng kết bằng tiếng Việt
Baseline không có icon wiring nên eframe dùng icon mặc định. Đã chọn PNG 256×256 làm asset canonical nhúng runtime và tạo thêm định dạng platform; giới hạn còn lại là installer/bundle pipeline chưa tồn tại.
