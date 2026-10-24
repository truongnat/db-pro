---
schema_version: 1
id: "MEM-PLAN-export-transfer-preservation-e69c70a4b70b"
title: "Plan archive: Export Transfer Preservation"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, export-transfer-preservation]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "e69c70a4b70bc13d1e4f222d06f65bb537eb774802b660d8a63caec561fc0344"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/export-transfer-preservation/PLAN.md"
  - "docs/plans/active/export-transfer-preservation/FINDINGS.md"
  - "docs/plans/active/export-transfer-preservation/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/export-transfer-preservation/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Plan — Safe Transfer Cleanup and Empty CSV Headers

## Objective

Fix related export/transfer issues #436 and #437 without deleting a prior successful export or publishing a schema-less empty CSV/TSV.

## Target Branch

`fix/export-transfer-preserves-destination`

## Severity

P1 data-loss regression (#436) and P2 empty delimited export schema loss (#437).

## Scope

- Restrict cleanup of CSV/TSV, JSONL, and Excel targets to the current run's temporary output.
- Preserve an existing final destination on cancel/error.
- Write delimited headers during finish when no rows were written.
- Keep JSONL's standard record-only representation: an empty export has zero lines; it has no header-record convention.


## Source: `FINDINGS.md`

# Findings — Export Transfer Preservation

## P1 — Existing export could be deleted by a failed replacement (#436)

- **Evidence:** `FileTarget::cleanup_partial` previously removed both the run's temporary output and its final destination. `TransferService` calls cleanup on cancellation and source/target errors, so a failed transfer could delete an older successful export.
- **Disposition:** Fixed. CSV/TSV, JSONL, and Excel cleanup now removes only the current run's `.partial` file. Regression coverage verifies the prior destination bytes survive cancellation and the partial is removed.

## P2 — Empty CSV/TSV export omitted its schema (#437)

- **Evidence:** Header emission happened only in `write_batch`; an empty source never calls that method, and `finish` previously published a zero-byte file.
- **Disposition:** Fixed. Delimited `finish` emits the header once if no batch has written it. Regression coverage verifies headers for empty CSV/TSV. JSONL remains record-only: an empty export correctly has zero lines, not a pseudo-header.


## Source: `VERIFICATION.md`

# Verification — Export Transfer Preservation

- `cargo test -p db-pro-core --lib delimited_transfer`: 6 passed, 0 failed.
- `cargo test -p db-pro-core --lib json_excel_transfer`: 3 passed, 0 failed.
- `cargo check -p db-pro-core`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- Regression tests prove cancelled export preserves prior final bytes, removes its partial output, empty CSV writes headers, and JSONL/Excel cleanup preserves existing destinations.

