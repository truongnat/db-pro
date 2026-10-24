---
schema_version: 1
id: "MEM-SRC-14b2183023169bcc"
title: "Source mirror: docs/plans/active/bsn-real-data-native-audit/dod.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/bsn-real-data-native-audit/dod.md"
source_sha256: "14b2183023169bccd408e0ccd03eb6d21c4812f3975eec41cf171d2a55dbc27b"
mirror_body_sha256: "14b2183023169bccd408e0ccd03eb6d21c4812f3975eec41cf171d2a55dbc27b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Definition of Done — BSN Real-Data Findings Remediation

- Current source triage distinguishes confirmed defects from historical or unverified visual/focus candidates.
- Confirmed correctness defects have consumer-visible failure-injection regression tests and isolated provider verification.
- Native i18n catalogs load all tested entries across five supported languages, retain both diagram key groups, and preserve interpolation placeholders.
- Confirmed missing high-impact translations and malformed Korean export wording are corrected.
- Any visual fixes use current native capture evidence; synthetic fixtures are labeled and never represented as BSN output.
- BSN remains read-only; no customer values, credentials, endpoint data, or raw query contents enter artifacts.
- Focused tests and relevant build/quality gates are executed and reported exactly.
- Native BSN screenshot/focus limitations remain explicit; plan is not moved to COMPLETED while those runtime gates are open.
- Evidence handoff follows the canonical template and ends with a Vietnamese summary.
