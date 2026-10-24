---
schema_version: 1
id: "MEM-SRC-024819d166e7e12f"
title: "Source mirror: docs/plans/active/issue-506-grapheme-word-select/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-506-grapheme-word-select/FINDINGS.md"
source_sha256: "024819d166e7e12fd2df6b0066670f3f417ada855cdff0c0f323c4018cd4675b"
mirror_body_sha256: "024819d166e7e12fd2df6b0066670f3f417ada855cdff0c0f323c4018cd4675b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #506 Findings

## Evidence
- `editor/selection.rs:74-104` classified each scalar char with
  `is_alphanumeric() || '_'`. Combining marks (e.g. U+0301 combining acute)
  are `Mark, Nonspacing` → non-word → they ended the word run.
- Repro: text `cafe\u{301}`; double-click on 'e' selected `cafe` (missing
  the mark) and clicking the mark selected only the mark.
- Vietnamese `ấ` decomposed (`a` + U+0302 + U+0301) split the word at three
  interior points.

## Decision
Grapheme-cluster expansion (`grapheme_indices(true)`) over
`unicode_word_indices`: UAX#29 word segmentation handles the word case
identically but would change non-word selection semantics (whitespace runs,
punctuation) that existing behavior deliberately exposes. Grapheme
expansion preserves the char-class contract and adds cluster atomicity.

## Severity
P2 — wrong selection target on double-click in non-ASCII identifiers.
