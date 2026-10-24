---
schema_version: 1
id: "MEM-SRC-a405f7ebd056af5e"
title: "Source mirror: docs/plans/active/table-mutation-failure-index-bounds-safety/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/table-mutation-failure-index-bounds-safety/FINDINGS.md"
source_sha256: "a405f7ebd056af5e98acc4f00a23b28fefa5aa3793f5f76c59ae1128fc519407"
mirror_body_sha256: "a405f7ebd056af5e98acc4f00a23b28fefa5aa3793f5f76c59ae1128fc519407"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings: Table Mutation Transaction Failure Index Bounds Safety

1. Pre-execution failures (`Validation`, `Begin`) and post-execution failures (`Commit`) report statement indices `0` or `statements.len()`.
2. Remapping `statement_index` to original mutation input index is only valid during `TransactionFailurePhase::Statement` when `statement_index < indexed_mutations.len()`.
3. `TableMutationExecution::execute` properly enforces both conditions, preventing out-of-bounds indexing or misattribution.
