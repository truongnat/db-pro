---
schema_version: 1
id: "MEM-SRC-85840f6ba6a858ca"
title: "Source mirror: docs/plans/active/issue-501-backend-start-guard/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-501-backend-start-guard/CHECKLIST.md"
source_sha256: "85840f6ba6a858ca57840a663fea2277857f47fd68aff026323a7b65e350f676"
mirror_body_sha256: "85840f6ba6a858ca57840a663fea2277857f47fd68aff026323a7b65e350f676"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — #501

- [x] Confirm actions carry `backend_id` + `backend_start` (Option<String>)
- [x] `expected_backend_start` field added: UiCommand → RuntimeCommand → api → service → port
- [x] PG SQL guards `a.backend_start = $2::timestamptz` inside the signal statement
- [x] Missing/empty backend_start refused (app feedback + service Validation)
- [x] Tests: cancel/terminate carry backend_start; None → no dispatch; service rejects empty
- [x] `cargo check` on all touched crates
- [x] `cargo test` on touched modules
