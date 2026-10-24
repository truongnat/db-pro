---
schema_version: 1
id: "MEM-SRC-f67b3dee11c68526"
title: "Source mirror: docs/plans/active/backend-four-issue-wave/CSV_FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/backend-four-issue-wave/CSV_FINDINGS.md"
source_sha256: "f67b3dee11c68526a3546b08b77c5c548191e499e23b055e08d3e5f826b81f88"
mirror_body_sha256: "f67b3dee11c68526a3546b08b77c5c548191e499e23b055e08d3e5f826b81f88"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #486 CSV BOM findings

Exact baseline: bceb735727f81180dad678defee1000bc547d97b. Production delimited_transfer.rs is unchanged in this wave.

- [verified/source] Cargo.lock pins csv1.4.0 and csv-core0.1.13. csv-core reader.rs540/597 calls strip_utf8_bom before parsing a field/record;610–619 removes EFBBBF only at first read, retaining embedded U+FEFF. The high-level reader used by DelimitedFileSource inherits this behavior. No second strip should be added based solely on the ticket's header-extraction snippet.
- [verified/automated] New adjacent consumer regression `application::delimited_transfer::tests::bom_import_streams_matching_headers_and_preserves_embedded_bom` PASS1/0/0 on unchanged source. It opens real CSV/TSV with BOM, asserts name-based id mapping and preview contents, runs the actual bounded TransferService/DelimitedFileTarget at batch size1, rereads full output and checks embedded BOM content/EOF for header/no-header cases.
- [verified/runtime] Exact-baseline independent public source→preview→streamed file transfer→DbPro SQLite import executable prints PASS8 cases (CSV/TSV × BOM/noBOM × header/noheader), exit0. Exact query results preserve both records/embedded U+FEFF; first id header maps to index0. Ticket does not reproduce on the shipped parser.

Disposition: close #486 as not planned / not reproducible with the pinned parser after publishing the consumer regression/evidence. No runtime-code fix or double strip. UTF-16 and deliberately altered parser buffering are outside this UTF-8 file contract; SQLite checks do not prove another provider's import behavior.
