---
schema_version: 1
id: "MEM-SRC-e1b61e33edbfc506"
title: "Source mirror: docs/plans/active/issue-cluster-triage/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-cluster-triage/PLAN.md"
source_sha256: "e1b61e33edbfc506b4857f323da9815bdde7f083b359d203d8e4b12335ed424a"
mirror_body_sha256: "e1b61e33edbfc506b4857f323da9815bdde7f083b359d203d8e4b12335ed424a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Continuous issue clusters — 2026-10-16

Lifecycle: REVIEW (selected verified wave published; inherited global gates and provider-blocked work remain explicit).
Baseline: `76b120724a7542de8b587a002811ddacad28156a`.
Owner authorization: commit/push directly on main, group related issues, continue without per-ticket approval, close each fully verified issue. Preserve user changes and existing PR ownership. Never close from title/source inspection alone.

## Requirements and acceptance

- [x] Publish #448 and close with exact commit/runtime evidence.
- [x] Classify all 156 issues in the starting snapshot by shared implementation/verification boundaries.
- [x] Implement selected uncovered coherent clusters, not one large mixed fix; published receipts in PUBLISHING.md.
- [x] Execute regression/runtime evidence for every claimed fix, account for PostgreSQL/SQLite independently and other providers when relevant.
- [x] Commit/push each verified slice; close only its satisfied acceptance, keeping #441/#400 partial/provider-blocked items open.

## Cluster map

This is an inventory, not a claim that every old issue still reproduces or that all will be implemented together. Some have open draft PRs or already-present fixes. The snapshot includes #448, closed after its verified fix was published.

| Cluster | Issues | Current handling |
|---|---|---|
| Data diff identity | #457 #485 | Typed-value/key fix published; both verified and closed |
| Provider-generated SQL | #441 #442 #499 #508 #365 #373 #443 | #441/#442 share dialect correctness verification; remaining items require their own evidence |
| SSH / connection transport | #435 #451 #475 #400 #452 #478 #479 #483 | #435/#451/#475 published/closed; #400 partial remains open; #479/#483 draft PR ownership preserved |
| Scheduled tasks | #504 #462 #375 | #504/#462 published/closed; #375 tracking scope separate |
| SQLite integrity / maintenance | #461 #498 #491 #403 #493 #494 | #491/#498 published/closed; #461 closed not planned after actual bundled-default proof; #403 source guard is not closure proof; #493/#494 owned by PR #527 |
| Query editor text / analysis | #511 #510 #509 #506 #488 #474 #472 #471 #470 #445 #446 | Search/grapheme overlaps PR #524/#523; analyze remaining state bugs separately |
| Query execution / safety | #477 #456 #438 #410 #374 #356 #351 #383 #382 #387 #507 | Respect PR #519/#518; confirmation/session ownership and execution cancellation are separate subclusters |
| Grid selection / clipboard / editing | #505 #497 #496 #487 #480 #463 #447 #444 #413 #412 #408 #389 #388 #369 #352 | Respect PR #526/#525; verify stale issues against main before new fixes |
| Schema workbench / DDL safety | #411 #402 #398 #390 #385 #381 #380 #371 #370 #368 #364 #361 #357 #354 #353 #350 #348 #347 #346 | Several baseline fixes exist; source alone does not justify closure |
| Connection-bound state / monitoring | #501 #500 #490 #489 #482 #481 #473 #467 #454 #440 #439 #407 #406 #399 #386 | Respect PR #520/#521; group by session identity/request ownership |
| Files / Git workspace | #513 #469 #468 #405 #393 #391 #377 | Filesystem/Git process lifecycle and stale-file state |
| Export / import / masking | #503 #502 #486 #484 #404 #396 #395 #384 | Respect PR #515; distinguish format fidelity from export snapshot/file safety |
| Agent orchestration / UI | #448 #409 #397 #394 #360 #358 #337 | #448 published/closed; remaining controller/UI acceptance separate |
| ER canvas | #512 #455 #450 #449 | Respect PR #522 for duplicate self-loop reports; spatial validation separate |
| General UI interaction / layout | #495 #464 #453 #401 #392 #379 #378 #376 #372 #367 #366 #363 #362 #355 #349 #345 #344 #343 #342 #341 #340 #339 #338 #336 #335 #334 | Require actual native visual/input evidence; no source-only closure |
| Provider execution / roadmap | #460 #459 #458 #359 #260 #252 #182 | Provider-specific runtime prerequisites and broad roadmap acceptance; not safe to close as a narrow bug batch |

## Execution / ownership

Parent owns integration, shared changelog/status, publish/closure, SQLite maintenance/error mapping and verification infrastructure. Independent implementers own data diff, generated SQL, scheduler, and SSH transport files. No mid-flight gates in child tasks; parent runs checks after integration.

Each coding cluster owns exactly one separate active plan with requirements/context/DoD/tasks and evidence. Do not overwrite historical plans. New fixes must not inherit unsupported closure claims from old plans.

## Verification and stop conditions

Existing adjacent regression tests plus actual public-service/provider smoke were executed. Disposable PostgreSQL18 and MySQL8.4.11 loopback fixtures and independent SQLite databases were used; existing application databases were untouched. SQLServer and actual bastion-auth evidence remain absent and are not inferred from other providers or a relay. Owned temporary fixtures/projects were removed after proof. Full gate/closure receipts: PUBLISHING.md and VERIFICATION.md.
