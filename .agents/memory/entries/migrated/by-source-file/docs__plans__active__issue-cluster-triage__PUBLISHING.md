---
schema_version: 1
id: "MEM-SRC-4fd5c5deba210e6d"
title: "Source mirror: docs/plans/active/issue-cluster-triage/PUBLISHING.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-cluster-triage/PUBLISHING.md"
source_sha256: "4fd5c5deba210e6d8da4da18a0ba3c076640eaefe57626e60c656879368148c1"
mirror_body_sha256: "4fd5c5deba210e6d8da4da18a0ba3c076640eaefe57626e60c656879368148c1"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Published issue clusters — 2026-10-16

Owner authorized direct-main commit/push and closure of individually verified issues. Every push below succeeded; remote reported a PR/status-rule bypass, not a green CI status. Local verification and independent source-review limits remain in each plan.

| Cluster | Exact published commit | Closed issues | Evidence |
|---|---|---|---|
| Agent replay budget | 76b120724a7542de8b587a002811ddacad28156a | #448 | Three pre-fix failing regressions, 18 focused PASS, bounded public-orchestrator smoke |
| Schedule enable/backlog | 58d7a47b263aa99a65b69b40b0c7febd3d1a2476 | #504, #462 | Baseline four public-store failures; candidate 21/21 PASS; independent source review ACCEPT |
| Typed data diff/key identity | 4943511bcf687778ca1a61df10d77b4b1f6080e8 | #457, #485 | 11 core PASS; baseline duplicate collision; standalone actual SQLite and PostgreSQL SQL apply/recompare seven equal rows |
| SQLite maintenance/constraint taxonomy | 90524ccb6cc9737647090ccb08f9ff8495d58001 | #491, #498 | Actual baseline/candidate engine errors/operations; corrected attached-schema suite 4/4 PASS; independent re-review ACCEPT |
| SQLite FK-default investigation | 90524ccb6cc9737647090ccb08f9ff8495d58001 | #461 (not planned / does not reproduce on bundled build) | Baseline and candidate fresh connector FK=1, orphan rejected, cascade effective; bundled compile option corroborates; regression added, actor unchanged |
| SSH transport lifecycle/route intent | 5ed80424311f9a32dce703416f01a96dee53932e | #435, #451, #475 | Before/after process environment and real PostgreSQL query drain over local relay; 16 focused PASS; no real bastion-auth claim |
| Provider-generated SQL | 899dbc2003eebd5d838d9949201dd0bd1237c0fb | #442 | MySQL default-mode baseline1064; real MySQL/SQLite/PostgreSQL PK+DDL probes PASS; MySQL cross-db FK enforcement PASS; independent re-review ACCEPT |
| PostgreSQL typed description prerequisite | 6f94372dd5caba22e525fbc1aff918b936e0491d | #533 | Original public INTEGER-PK probe failing before/PASS after; five real-PG cases, 16 mapper tests PASS; final independent source re-review ACCEPT |

## Intentionally open

- #441: code published but named live SQLServer PK/DDL gate not verified; actual SQLServer server/authorized fixture missing. A comment records code/evidence without closure.
- #400: truthful deferred preflight published; missing structured handshake/bind/TCP/auth diagnostics and real bastion validation. A comment records partial result without closure.
- Remaining backlog: inventory only, not silently marked fixed. Open draft PR ownership was preserved. Source-only old safety issue audit did not close any issue.

## Gate transparency

- Final integrated workspace tests: 2,003 passed / 0 failed / 50 ignored (`artifact://160`), exit0; fmt PASS, core/infra/runtime all-target Clippy PASS. Five actual PG typed-query cases and 16 mapper cases also PASS (`artifact://159`).
- Workspace Clippy still fails on five unchanged app diagnostics (two unnecessary_to_owned macro diagnostics plus three test initializer warnings). Introduced clone-on-Copy test warnings were corrected without suppression.
- Clean-code ratchet ran: 12 pass / 4 warnings / 0 failures. Full scanner reported inherited debt plus two newly enlarged functions; introduced query/catalog complexity was removed by meaningful helper/SQL-constant extraction. No baseline regeneration or new blanket suppression.
- `cargo build --release --locked -p db-pro-native` PASS; actual Cargo target is `/data/cargo-target`, final binary `/data/cargo-target/release/db-pro-native` observed 46,674,944 bytes. No performance-improvement or budget claim from a size observation.
- `bash .skills/perf-audit/scripts/perf-scan.sh` FAIL exit1: 2 pass / 1 warning / 1 fail / 4 sections not executed. Release build and workspace check succeeded but scanner looked for missing `target/release/db-pro-native`; actual target directory differs. No benchmark measurement was obtained and no UI/provider performance claim is made.
- Disposable PostgreSQL18/MySQL8.4.11 fixtures stopped/auto-removed; temporary standalone projects, baseline export, relay scripts and generated-SQL state files removed after final proof. Existing application databases/containers untouched.
