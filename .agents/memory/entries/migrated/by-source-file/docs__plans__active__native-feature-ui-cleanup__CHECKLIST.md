---
schema_version: 1
id: "MEM-SRC-1bc0acec35d9cbbd"
title: "Source mirror: docs/plans/active/native-feature-ui-cleanup/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/native-feature-ui-cleanup/CHECKLIST.md"
source_sha256: "1bc0acec35d9cbbd51c7bb95f9ceade902d60608acfbc6c70fc56b73423eba4f"
mirror_body_sha256: "1bc0acec35d9cbbd51c7bb95f9ceade902d60608acfbc6c70fc56b73423eba4f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

- [x] Load ui-ux-pro-max and native project UI rules; verify search fit.
- [x] Research native feature families and distinguish observed defects from source-only risks.
- [ ] Implement shared quiet frame hierarchy and remove obsolete local overrides where safe.
- [ ] Repair evidence-backed feature layout defects without changing actions or database logic.
- [ ] Run affected regression tests and final native build/quality checks.
- [ ] Capture/inspect Light/Dark normal and applicable empty/loading/error states at requested widths.
- [x] Provisioned loopback-only PostgreSQL demo (`fullstack_starter`: 18 base tables, 2 views, 81,200 high-volume rows) and saved a dedicated DB Pro demo profile.
- [ ] Capture and inspect the actual PostgreSQL-connected Table/Data, Query results and Schema views; current native screenshot starts on the SQLite sample and is not PG view evidence.
- [ ] Record exact source snapshot, limitations and independent-review status.
