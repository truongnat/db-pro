---
schema_version: 1
id: "MEM-PLAN-issue-506-grapheme-word-select-f33005be1eaa"
title: "Plan archive: Issue 506 Grapheme Word Select"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-506-grapheme-word-select]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "f33005be1eaa452a7be82a62e55249a8476c6f37c22144fbf862771141cba8fd"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-506-grapheme-word-select/PLAN.md"
  - "docs/plans/active/issue-506-grapheme-word-select/FINDINGS.md"
  - "docs/plans/active/issue-506-grapheme-word-select/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-506-grapheme-word-select/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# #506 — select_word_at splits words at combining marks

## Goal
Double-click word selection must treat grapheme clusters as indivisible so
combining marks (U+0301, Vietnamese combining tones U+0302+U+0301) no longer
split words: `café` (decomposed) selects the whole word.

## Root cause
`select_word_at` classed each **char** with `is_alphanumeric() || '_'`.
Combining marks are non-alphanumeric → clicking/expanding across them broke
the run, and clicking the mark itself selected just the mark.

## Fix
`crates/ui/src/editor/selection.rs` — expand by grapheme cluster
(`unicode-segmentation::grapheme_indices(true)`, new workspace dep): the
clicked cluster is classified by its base char (`alphanumeric || '_'`);
expansion walks cluster boundaries and breaks on whitespace or a class
change — identical semantics to the old classifier, with cluster atomicity
added. Offset at end-of-buffer selects the last cluster (old `prev_char`
behavior).

## Dependency
`unicode-segmentation = "1"` added to `[workspace.dependencies]` and
`db-pro-ui` (justified per issue; already in the lock file transitively).

## Tests
`editor/tests/selection_tests.rs`: decomposed `café` (click on mark and on
base char), decomposed Vietnamese `ấ`, end-of-buffer last-word selection.


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

# #506 Verification

- `cargo check -p db-pro-ui` → Finished, no errors (also `db-pro-app` for #510 caller).
- `cargo test -p db-pro-ui` → 687 passed, 0 failed.
- `cargo clippy -p db-pro-ui` → no warnings in touched files.
- `cargo fmt -p db-pro-ui -p db-pro-app -- --check` → clean.
- `bash scripts/check-ui-architecture.sh` → PASS.

## New regression tests
- `select_word_keeps_combining_marks_inside_the_word`
- `select_word_handles_decomposed_vietnamese_tone`
- `select_word_at_end_of_buffer_selects_last_word`

