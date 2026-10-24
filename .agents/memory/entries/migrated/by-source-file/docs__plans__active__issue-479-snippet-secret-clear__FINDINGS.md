---
schema_version: 1
id: "MEM-SRC-de7bc59a9b37ae19"
title: "Source mirror: docs/plans/active/issue-479-snippet-secret-clear/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-479-snippet-secret-clear/FINDINGS.md"
source_sha256: "de7bc59a9b37ae1967c08701f4986bd1cbeb3e5d503c47eef9d81794371da7c6"
mirror_body_sha256: "de7bc59a9b37ae1967c08701f4986bd1cbeb3e5d503c47eef9d81794371da7c6"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #479 Findings

- `apply_connection_snippet` (`crates/app/src/connection/mapper.rs`) copied the
  URI's password into `draft.password` but left the full URI — including the
  plaintext password — in `draft.cloud_snippet`.
- `advanced_panels.rs:108-128` binds `cloud_snippet` to an unmasked `Input`
  (`.password(false)` semantics — plain text field), so the secret stays
  visible in the Cloud Presets panel after import.
- Grep audit of `cloud_snippet` across the workspace:
  - written to `String::new()` at draft construction sites
    (`mapper.rs:98`, `workspace_actions.rs:378`, runtime `Default`)
  - read only by the paste `Input` and cloned for the Import button call in
    `advanced_panels.rs`
  - never serialized to disk by `draft_to_domain_config` (no such field on
    `ConnectionConfig`), but the live draft keeps the secret for the dialog's
    lifetime — clearing is still required by the issue.
- Conclusion: nothing depends on the snippet after a successful import;
  `draft.cloud_snippet.clear()` inside `apply_connection_snippet` is the
  minimal safe fix. Done in the mapper so all callers get the guarantee; a
  failed parse keeps the user's input for correction.
