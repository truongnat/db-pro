---
schema_version: 1
id: "MEM-PLAN-issue-509-search-unicode-offsets-712167833bc4"
title: "Plan archive: Issue 509 Search Unicode Offsets"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-509-search-unicode-offsets]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "712167833bc429766ab13c4cec6bb34348ed8fe59e514b9fe840fcefdadbbf89"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-509-search-unicode-offsets/PLAN.md"
  - "docs/plans/active/issue-509-search-unicode-offsets/FINDINGS.md"
  - "docs/plans/active/issue-509-search-unicode-offsets/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-509-search-unicode-offsets/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# #509 — Search highlight offsets corrupt on Unicode case expansion

## Goal
Case-insensitive search must produce `(start, end)` byte ranges in the
ORIGINAL text — never offsets into a lowercased copy.

## Root cause
`EditorSearchState::update_matches` (and a duplicated loop in
`editor/renderer.rs:444-481`) ran `match_indices` on `text.to_lowercase()`.
Case folding changes byte width (`İ` U+0130 → "i\u{307}" 2→4 bytes; Kelvin
`K` U+212A → "k" 3→1 byte), so every range after such a char shifted off the
original text → wrong highlighting / mid-char slice panic.

## Fix
- New `crates/ui/src/editor/search.rs`:
  `case_insensitive_matches(text, query)` iterates `char_indices` on the
  original text and compares per-char `to_lowercase()` iterators, yielding
  non-overlapping `(start, end)` ranges on original char boundaries.
  Documented contract: expansion chars like `İ` compare as their expansion
  and do NOT equal a single query char — boring, predictable find behavior.
- `query/query_document.rs::update_matches` and
  `editor/renderer.rs::paint_search_and_execution` now share the helper —
  search state and painted highlights can no longer diverge.

## Tests
`editor/tests/search_tests.rs` (ascii, empty/overlong queries,
non-overlap, `İ`-expansion offset shift, expansion-char no-match, Kelvin
`K`, char-boundary ends) +
`query/tests/query_document_tests.rs::search_match_ranges_are_original_
text_byte_offsets`.


## Source: `FINDINGS.md`

# #509 Findings

## Evidence
- `query/query_document.rs:61-65` (`update_matches`) and the duplicated
  paint loop `editor/renderer.rs:444-447` both ran
  `text.to_lowercase().match_indices(&query.to_lowercase())` and then used
  those indices on the original text.
- Unicode full case folding changes byte width per char:
  `İ`(U+0130,2B)→`i\u{307}`(4B), Kelvin `K`(U+212A,3B)→`k`(1B). Every match
  after such a char is shifted → wrong highlight; a shifted end can land
  mid-char → `&text[s..e]` panics (renderer `range_to_screen_rects` slices).
- The two sites had already diverged in behavior risk; now share
  `editor::search::case_insensitive_matches`.

## Decision
Per-char lowercase comparison (`char::to_lowercase` iterators) keeps
original byte offsets and non-overlap semantics of `match_indices`.
Documented limitation: expansion chars compare as their expansion, so `İ`
never equals query 'i' — matches the issue's "boring correct behavior" note;
full case-folded position mapping is not needed for a find UI.

## Severity
P2 — panic/mis-highlight whenever the buffer contains expansion-fold chars.


## Source: `VERIFICATION.md`

# #509 Verification

- `cargo check -p db-pro-ui` → Finished, no errors (also `db-pro-app` for #510 caller).
- `cargo test -p db-pro-ui` → 687 passed, 0 failed.
- `cargo clippy -p db-pro-ui` → no warnings in touched files.
- `cargo fmt -p db-pro-ui -p db-pro-app -- --check` → clean.
- `bash scripts/check-ui-architecture.sh` → PASS.

## New regression tests
- `editor::search::tests` (7 tests incl. İ expansion + Kelvin K offsets)
- `search_match_ranges_are_original_text_byte_offsets`

