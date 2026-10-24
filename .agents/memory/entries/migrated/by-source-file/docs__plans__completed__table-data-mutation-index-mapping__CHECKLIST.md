---
schema_version: 1
id: "MEM-SRC-25b726a96b147e89"
title: "Source mirror: docs/plans/completed/table-data-mutation-index-mapping/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/table-data-mutation-index-mapping/CHECKLIST.md"
source_sha256: "25b726a96b147e893a39f743cad1d9770900b7550ae3539dc4c269ff8d7846e5"
mirror_body_sha256: "25b726a96b147e893a39f743cad1d9770900b7550ae3539dc4c269ff8d7846e5"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — Table Data Mutation Failure Index Mapping

- [x] Reproduce & identify defect in `TableMutationExecution` statement index remapping and `validation_failure`
- [x] Gate statement index remapping on `failure.phase == TransactionFailurePhase::Statement`
- [x] Set `statement_index: 0` in `validation_failure`
- [x] Add unit tests in `table_data_service.rs` for `Validation`, `Begin`, and `Statement` failure index mapping
- [x] Add unit test `apply_mutations_detailed_maps_statement_index_complex_reordering` for `[Insert, Update, Delete, Insert]`
- [x] Run `cargo test -p db-pro-core` to verify all tests pass
- [x] Complete pre-commit checks (`cargo fmt`, `cargo check`, `cargo clippy`)
- [x] Publish Pull Request
