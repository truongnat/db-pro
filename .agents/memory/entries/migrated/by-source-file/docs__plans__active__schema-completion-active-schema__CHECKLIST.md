---
schema_version: 1
id: "MEM-SRC-27a05113b9ff4979"
title: "Source mirror: docs/plans/active/schema-completion-active-schema/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/schema-completion-active-schema/CHECKLIST.md"
source_sha256: "27a05113b9ff49798d445226e673b1c69326443916c0e9857af0ce9a162e34c4"
mirror_body_sha256: "27a05113b9ff49798d445226e673b1c69326443916c0e9857af0ce9a162e34c4"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #511 Checklist

- [x] PLAN/FINDINGS/CHECKLIST/VERIFICATION scaffold
- [x] Repro test (fail-before): active=tenant_b nhưng gợi ý cột tenant_a
- [x] Helper `find_completion_table` + 3 call-site
- [x] Tests: prefer-active, fallback, ai-context
- [x] Focused tests + fmt + clippy + cc-scan PASS
- [ ] Comment/close issue (chờ owner quyết định)
