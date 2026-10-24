---
schema_version: 1
id: "MEM-SRC-692248de69f87682"
title: "Source mirror: docs/plans/active/bsn-real-data-native-audit/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/bsn-real-data-native-audit/PLAN.md"
source_sha256: "692248de69f876829a089793f768603c67d6de3ac7f85aa8bbf819345d26e44f"
mirror_body_sha256: "692248de69f876829a089793f768603c67d6de3ac7f85aa8bbf819345d26e44f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# BSN Real-Data Native Product Audit

State: `IMPLEMENTING` — prior source/test remediation is recorded below. This continuation is a read-only QA audit; safe BSN-native captures now cover additional real feature surfaces, but coverage remains partial and no new product source change is authorized by this request.

Baseline source SHA: `cb6e00d633772de538498384a8b14d32d25f7915`.

## Goal

Audit and repair concrete native DB Pro defects found while reviewing the product against the owner's BSN PostgreSQL use case. Prioritize UI layout and localization, and follow confirmed logic defects through the app/provider boundary. Safe BSN reads are allowed; all data mutations and provider failure tests must use isolated fixtures.

## Scope

- Existing 25-row native feature audit in `CHECKLIST.md`; verify each recorded finding against current source and available runtime evidence.
- Fix confirmed correctness and user-visible defects, including cross-provider copy transaction failure atomicity, catalog defects, and incomplete localization of audited high-impact surfaces.
- Revalidate historical screenshot-only clipping candidates against current source/captures; fix only those reproduced on the current implementation.
- Audit supported `en`, `vi`, `zh`, `ja`, and `ko` catalogs for key structure, interpolation placeholders, and native view coverage.
- Preserve DB Pro UI components, theme tokens, actions, provider capability boundaries, and BSN data safety.
- Keep all claims separated into current native/runtime evidence, isolated-provider tests, source-only evidence, and historical synthetic captures.

## Safety and privacy boundary

- The saved profile is named `BSN`, uses PostgreSQL, and has the DB Pro `readonly` flag **false**. The app-side flag is not a safety boundary for this audit.
- Audit actions against BSN are read-only: metadata inspection and bounded `SELECT`/non-`ANALYZE` `EXPLAIN` only. No `INSERT`, `UPDATE`, `DELETE`, DDL, role/privilege changes, session termination, settings changes, scheduled-task execution, restore, or transfer apply.
- Do not export, backup, or persist raw BSN rows. Screenshots and logs must not contain credentials, customer identifiers, private addresses, or sensitive query text; use aggregate/sanitized results or redact before storing evidence.
- Any mutation behavior that cannot be proven without a write remains `NOT VERIFIED`; do not infer it from a fixture or a unit test.

## Evidence rules

- Reuse prior audits only for the exact feature/path/state they prove. The existing `ux-friendliness-audit`, `native-feature-ui-cleanup`, `native-ui-focus-consistency`, and RC1 QA are references, not fresh BSN runtime proof.
- The native capture driver's `DB_PRO_CAPTURE_*` openers seed presentation fixtures. Such screenshots are explicitly synthetic and never count as BSN data evidence.
- Capture each activity in Light and Dark at 1280×800 where the runtime permits. For the data/query/schema/ER/monitoring/security critical paths, include wide-layout checks at requested 1440×900 and 1920×1080 and report actual framebuffer sizes. Capture relevant loading/empty/error/focus states where safely reproducible.
- Static screenshots prove only visible layout. Keyboard focus, state transitions, query behavior, and command-to-database round trips require runtime interaction/log/test evidence.

## Out of scope

- Writes, DDL, administrative actions, exports, backups, transfers, restores, or preference/profile changes against BSN.
- Treating historical fixtures as current BSN UI proof; completing unobservable runtime checks by inference.
- Archived React/Tauri frontend changes, external UI-framework dependencies, or wholesale view/theme rewrites.

## Acceptance

1. Each confirmed in-scope defect has a regression test or direct reproduction and a minimal root-cause fix.
2. Transfer `Replace` and multi-statement copy writes execute atomically on the target using the provider transaction contract; failure leaves prior target data intact.
3. All five locale catalogs resolve the tested keys without duplicate-key loss or placeholder mismatch; confirmed high-impact native strings are localized and Korean export copy describes exporting correctly.
4. Historical layout candidates are either reproduced and fixed on the current implementation or retained as explicitly unverified; no screenshot-only inference is reported as a current defect.
5. Safe BSN runtime reads remain read-only; provider mutation tests use disposable local fixtures only.
6. Focused tests, package checks, native build/capture where available, source SHA, exact limitations, and Vietnamese handoff are recorded. This plan remains active until all required live screenshot, feature, focus and viewport gates are evidenced or explicitly dispositioned; safe captures are now available, but the complete matrix remains incomplete.
