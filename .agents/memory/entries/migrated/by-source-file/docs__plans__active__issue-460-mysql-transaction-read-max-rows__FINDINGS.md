---
schema_version: 1
id: "MEM-SRC-31fce3365182438e"
title: "Source mirror: docs/plans/active/issue-460-mysql-transaction-read-max-rows/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-460-mysql-transaction-read-max-rows/FINDINGS.md"
source_sha256: "31fce3365182438eda5dc87d82ad96273988ca7ff160c6005a858a82217addb8"
mirror_body_sha256: "31fce3365182438eda5dc87d82ad96273988ca7ff160c6005a858a82217addb8"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #460 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/460
- Fix: execute_transaction_joined streams read queries and truncates at max_rows instead of using fetch_all unconditionally.
