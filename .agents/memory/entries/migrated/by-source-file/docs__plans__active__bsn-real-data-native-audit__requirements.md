---
schema_version: 1
id: "MEM-SRC-14495fd1204b212d"
title: "Source mirror: docs/plans/active/bsn-real-data-native-audit/requirements.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/bsn-real-data-native-audit/requirements.md"
source_sha256: "14495fd1204b212d79883db1c7e57a5bfc86a0b681fc6ea57d50336df0512c50"
mirror_body_sha256: "14495fd1204b212d79883db1c7e57a5bfc86a0b681fc6ea57d50336df0512c50"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Requirements — BSN Real-Data Findings Remediation

## In scope
- [x] Audit the shipped native Rust/egui product, not archived React/Tauri UI.
- [x] Cover all 15 `Activity` routes, Agent, core feature families, and relevant child workflows in `CHECKLIST.md`.
- [x] Use the owner's saved BSN PostgreSQL profile only for realistic, non-mutating reads.
- [x] Review layout, focus/navigation, accessibility cues, responsive density, behavior/logic, and localization separately.
- [x] Record current findings and evidence provenance; historical fixtures remain non-current.
- [x] After the owner's explicit authorization, verify and fix confirmed in-scope findings with regression coverage.
- [ ] Audit all supported locale catalogs and high-impact native UI text for missing, malformed, or inconsistent localization.

## Out of scope
- No write/DDL/administrative/transfer/restore action against BSN.
- No raw-row, credentials, personal data, or private query text in persisted screenshots/logs.
- No fixes for unverified screenshot-only or interaction-only hypotheses; keep these blocked with evidence gaps.
- No archived frontend, external UI-framework dependency, or unrelated feature expansion.

## Acceptance
- Confirmed findings are resolved with a regression test or current-runtime evidence; unconfirmed findings are explicitly blocked rather than silently closed.
- Locale keys and placeholders are coherent across `en`, `vi`, `zh`, `ja`, and `ko`; localized UI strings resolve in every supported language.
- Provider failure/rollback tests use isolated PostgreSQL/SQLite targets, never BSN.
- No behavior is claimed fixed, broken, or passed without direct source/test/runtime evidence.
- `VERIFICATION.md`, `FINDINGS.md`, and `AGENT_EVIDENCE.md` identify source revision, commands/outcomes, screenshots/provenance, blockers, and exact limitations.
