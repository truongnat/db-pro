---
schema_version: 1
id: "MEM-SRC-89e67a3b6344c062"
title: "Source mirror: docs/plans/completed/repository-folder-mapping/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/repository-folder-mapping/FINDINGS.md"
source_sha256: "89e67a3b6344c062bd0065a7e1ceb8351c8b3190b91bbaa52497b9c8078310bc"
mirror_body_sha256: "89e67a3b6344c062bd0065a7e1ceb8351c8b3190b91bbaa52497b9c8078310bc"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Repository Folder Mapping Findings

Baseline: be2e431b07a8fcfbf39bf16fa1deefd600273633.

## Evidence and decisions
- Root plans/ contains eight interlinked documents. Its index calls 00, 01 and 07 current release references; do not label the entire collection obsolete. Move together to docs/plans/project, not archived/completed.
- Root tasks/plan.md and tasks/todo.md both describe core-component-standardization, but have inconsistent historical acceptance states. Move together under docs/plans/project/core-component-standardization without merging them into legacy-component-sunset or declaring completion.
- QA_AUDIT_REPORT.md belongs to the active bsn-real-data-native-audit plan. Retain its findings and scope as a historical revision-pinned report.
- open-api-style.md is a UI design specification despite its filename; open-ai-refer.md is a component reference. Both belong under docs/design/reference.
- docs/stitch_screens/inventory.json embeds repository-root file paths; update all 19 entries when moving exports under docs/design.
- Loose query-workspace-zed-shell screenshots have an existing completed-plan owner and non-colliding filenames; move there without deleting existing captures.
- Keep docs/10-egui-native-migration-plan.md and docs/DESIGN.md at their established entry-point paths. Preserve raw historical command transcripts and SHA-qualified old-source provenance; only repair live navigation/references.
- Rust language server: configured rust-analyzer failed to start (server exited code 0) on capabilities request. Mechanical path moves must use explicit module/include path inspection and compiler/test verification; do not rename logical symbols.

## Severity
P0/P1: none identified in the organization task. Existing product findings are out of scope and are neither resolved nor downgraded by moving reports.

## Known inherited navigation debt
The plan registry refers to active/query-workspace-zed-shell although its directory is completed/query-workspace-zed-shell. This move does not change lifecycle state; navigation should point to the actual directory without asserting new runtime evidence.
