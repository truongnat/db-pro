---
schema_version: 1
id: "MEM-SRC-9d33075d3f6956d8"
title: "Source mirror: docs/notes/UI_TICKET_SCOPE.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/notes/UI_TICKET_SCOPE.md"
source_sha256: "9d33075d3f6956d81a2db1677e427178598c6bf81c32bbcbe8de5563b66c1e79"
mirror_body_sha256: "9d33075d3f6956d81a2db1677e427178598c6bf81c32bbcbe8de5563b66c1e79"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Native UI Ticket Scope

All UI upgrade issues created from the 2026-09-16 audit should follow these rules:

- start from latest main;
- one focused issue/branch at a time;
- reuse canonical native components and theme tokens;
- no new ad-hoc styling system;
- no business/provider logic in renderer code;
- actual native runtime evidence is mandatory;
- verify dark/light and 1280x800 / 1440x900 / 1920x1080 where applicable;
- verify empty/loading/error/disabled/long-content states;
- verify keyboard focus and mouse hover/pressed states;
- merge accepted issue to main before beginning the next dependent UI issue.
