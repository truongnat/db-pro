---
schema_version: 1
id: "MEM-SRC-53fc871e0d7e85ab"
title: "Source mirror: docs/plans/active/issue-483-pgpass-leak/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-483-pgpass-leak/CHECKLIST.md"
source_sha256: "53fc871e0d7e85ab314ecc7677575e6cbabb0073519a457109715d7bac39c35c"
mirror_body_sha256: "53fc871e0d7e85ab314ecc7677575e6cbabb0073519a457109715d7bac39c35c"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #483 checklist

- [x] `write_pgpass` removes the file when `write_all` fails
- [x] sink dropped before unlink (Windows remove-while-open rule)
- [x] `create` wires the helper before `Ok(Self { path })`
- [x] failing-writer test proves file absent after error
- [x] success-path companion test
- [x] `cargo check -p db-pro-infrastructure` clean
- [x] `cargo test -p db-pro-infrastructure --lib` green: 167 passed / 0 failed
