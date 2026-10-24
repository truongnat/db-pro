---
schema_version: 1
id: "MEM-SRC-955465a54bf97221"
title: "Source mirror: docs/plans/active/license-public-release/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/license-public-release/PLAN.md"
source_sha256: "955465a54bf972213dc0472828707ed45b0773b2de9065b45da3f93534aefb48"
mirror_body_sha256: "955465a54bf972213dc0472828707ed45b0773b2de9065b45da3f93534aefb48"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# DB Pro license and public-release policy

## Goal

Record the owner-approved MIT policy and remove the undefined-license state
from the current release documentation and Rust package metadata.

## Scope

- add the MIT license text;
- declare MIT metadata for every workspace package;
- record the DB Pro trademark boundary;
- add the third-party notice inventory index;
- reconcile current release/readiness documents.

## Non-goals

- renaming source identifiers, binaries, bundle IDs, or persisted paths;
- changing release runtime support, signing, or GUI-smoke decisions;
- legal clearance of the DB Pro trademark;
- generating a final platform-specific binary notice bundle.

## Acceptance

- `LICENSE` exists and is the selected MIT policy;
- all workspace packages inherit `license = "MIT"`;
- current release documents no longer say the license is undecided;
- remaining runtime/platform blockers remain accurately open;
- tests/gates actually executed are recorded in `VERIFICATION.md`.
